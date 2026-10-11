#!/usr/bin/env python3
"""Inventory telemetry SDKs, network primitives and external hosts in upstream/*.

Static scan only (no build, no execution). It locates candidates; whether a call is
actually reachable and enabled is assessed by hand in docs/artcraft/EXTERNAL-CALLS.md.

Usage:
    python3 -I scripts/audit-external-calls.py [upstream_dir] [out_json]
"""
import collections
import json
import os
import re
import subprocess
import sys

CODE_EXT = {".rs", ".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs", ".py", ".html", ".sh", ".swift", ".m", ".mm"}
SKIP_PARTS = {".git", "node_modules", "target", "dist", ".next"}
MAX_EXAMPLES = 15

# Applied case-insensitively on non-comment lines.
TELEMETRY = {
    "posthog": r"posthog",
    "sentry": r"@sentry/|\bsentry::|sentry\.init|ingest\.(us\.)?sentry\.io",
    "google-analytics": r"\bgtag\(|googletagmanager\.com|google-analytics\.com",
    "ms-clarity": r"clarity\.ms",
    "amplitude": r"@amplitude/|api\.amplitude\.com|amplitude\.(init|track|getInstance)\(",
    "mixpanel": r"mixpanel",
    "segment": r"@segment/|cdn\.segment\.com",
    "datadog": r"datadoghq\.|dd-trace|@datadog/|datadog_client",
    "opentelemetry": r"opentelemetry_otlp|opentelemetry-otlp|otlp",
    "newrelic": r"newrelic",
    "statsig": r"statsig",
    "intercom": r"widget\.intercom\.io|intercomSettings",
    "ad-pixels": r"fbq\(|connect\.facebook\.net|analytics\.tiktok\.com|static\.ads-twitter|snap\.licdn\.com|redditstatic",
    "discord-presence": r"discord_rich_presence|DiscordIpc",
    "usage-heartbeat": r"/v1/analytics/|log_active_user",
    "crash-upload": r"minidump|crashpad|breakpad",
}
NETWORK = {
    "http-client": r"\b(reqwest|ureq|wreq|isahc|attohttpc|ehttp|gloo_net)::|hyper_util::client",
    "browser-fetch": r"window\.fetch|fetch_with_(str|request)|\bfetch\(\s*['\"`]https?://|XMLHttpRequest|axios\.",
    "websocket": r"new WebSocket\(|tungstenite",
    "raw-tcp-out": r"TcpStream::connect",
    "listen": r"TcpListener::bind|HttpServer::new|axum::serve",
    "spawn-curl": r"Command::new\(\"curl\"\)",
    "update-check": r"releases/latest|tauri_plugin_updater|self_update::",
    "open-browser": r"\bopen::that|webbrowser::open|tauri_plugin_opener|ctx\.open_url",
    "python-http": r"\brequests\.(get|post)\(|urllib\.request|httpx\.",
}
URL_RE = re.compile(r"https?://([A-Za-z0-9.-]+\.[A-Za-z]{2,})")
# Hosts that only appear as identifiers, docs or package metadata, never as runtime endpoints.
IGNORED_HOSTS = re.compile(
    r"(^|\.)(w3\.org|openxmlformats\.org|schemas\.microsoft\.com|ns\.adobe\.com|purl\.org|purl\.oclc\.org|"
    r"xml\.org|oasis-open\.org|iptc\.org|xmlns\.com|docs\.rs|crates\.io|rust-lang\.org|mozilla\.org|"
    r"wikipedia\.org|example\.(com|org|net)|[a-z0-9-]*\.example|[a-z0-9-]*\.invalid|[a-z0-9-]*\.test|spdx\.org|"
    r"opensource\.org|creativecommons\.org|apache\.org|unicode\.org|ietf\.org|iso\.org|schema\.org|"
    r"json-schema\.org|ecma-international\.org|khronos\.org|registry\.npmjs\.org|opencollective\.com|"
    r"tidelift\.com|feross\.org|github\.com|githubusercontent\.com|x\.org|a\.org|b\.org|y\.com)$"
)
TEST_PATH = re.compile(r"/(tests?|test_utils|testkit|testing|fixtures?|benches|examples|__tests__|test_data|golden)/|_tests?\.|\.test\.|\.spec\.|/tests?\.rs$|tests_[a-z_]+\.rs$")
DEV_PATH = re.compile(r"^/(xtask|scripts?|tools|_tools|perf|scorecard|packaging|book|docs|_docs|_metrics|build|\.github)/|/xtask/|/build\.rs$|/scripts?/|/tools/")


def classify(rel: str) -> str:
    path = "/" + rel.lower()
    if TEST_PATH.search(path):
        return "test"
    if DEV_PATH.search(path):
        return "dev"
    return "prod"


def is_comment(line: str) -> bool:
    s = line.strip()
    return s.startswith(("//", "*", "/*", "<!--", "--")) or (s.startswith("#") and not s.startswith(("#[", "#!")))


def read_lock(root: str) -> dict:
    pins = {}
    try:
        with open(os.path.join(root, "UPSTREAM.lock"), encoding="utf-8") as fh:
            for line in fh:
                cols = line.rstrip("\n").split("\t")
                if len(cols) >= 3 and not line.startswith("#"):
                    pins[cols[0]] = cols[2]
    except OSError:
        pass
    return pins


def scan_repo(path: str, commit: str) -> dict:
    files = subprocess.run(["git", "-C", path, "ls-files"], capture_output=True, text=True, check=False).stdout.splitlines()
    hits = {"telemetry": collections.defaultdict(list), "network": collections.defaultdict(list)}
    counts = {"telemetry": collections.Counter(), "network": collections.Counter()}
    hosts = collections.defaultdict(lambda: {"count": 0, "example": ""})

    for rel in files:
        if any(part in SKIP_PARTS for part in rel.split("/")):
            continue
        if os.path.splitext(rel)[1].lower() not in CODE_EXT or classify(rel) != "prod":
            continue
        full = os.path.join(path, rel)
        try:
            if os.path.getsize(full) > 3_000_000:
                continue
            with open(full, encoding="utf-8", errors="ignore") as fh:
                lines = fh.read().split("\n")
        except OSError:
            continue
        for no, line in enumerate(lines, 1):
            if len(line) > 2000 or is_comment(line):
                continue
            where = f"{rel}:{no}"
            for kind, table in (("telemetry", TELEMETRY), ("network", NETWORK)):
                for name, rx in table.items():
                    if re.search(rx, line, re.I):
                        counts[kind][name] += 1
                        if len(hits[kind][name]) < MAX_EXAMPLES:
                            hits[kind][name].append(where)
            for m in URL_RE.finditer(line):
                host = m.group(1).lower()
                if IGNORED_HOSTS.search(host):
                    continue
                hosts[host]["count"] += 1
                hosts[host]["example"] = hosts[host]["example"] or where

    return {
        "commit": commit,
        "telemetry": {k: {"count": counts["telemetry"][k], "examples": v} for k, v in sorted(hits["telemetry"].items())},
        "network": {k: {"count": counts["network"][k], "examples": v} for k, v in sorted(hits["network"].items())},
        "hosts": dict(sorted(hosts.items(), key=lambda kv: -kv[1]["count"])),
    }


def main() -> None:
    root = sys.argv[1] if len(sys.argv) > 1 else "upstream"
    out = sys.argv[2] if len(sys.argv) > 2 else "docs/artcraft/external-calls.inventory.json"
    repos = sorted(d for d in os.listdir(root) if os.path.isdir(os.path.join(root, d)))
    pins = read_lock(root)
    result = {}
    for repo in repos:
        if repo not in pins:
            continue
        result[repo] = scan_repo(os.path.join(root, repo), pins[repo])
        tel = ", ".join(f"{k}:{v['count']}" for k, v in result[repo]["telemetry"].items()) or "-"
        print(f"{repo:24} telemetry[{tel}] hosts={len(result[repo]['hosts'])}")
    with open(out, "w", encoding="utf-8") as fh:
        json.dump(result, fh, indent=1, ensure_ascii=False)
        fh.write("\n")


if __name__ == "__main__":
    main()
