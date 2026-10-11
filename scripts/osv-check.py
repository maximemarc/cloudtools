#!/usr/bin/env python3
"""List known vulnerabilities in every tracked Cargo.lock and pinned requirements file.

Queries OSV (osv.dev) and keeps the advisories that have a GitHub advisory id (GHSA): those are
what Dependabot reports. RustSec-only notices (unmaintained or unsound crates) are counted, not
listed; `cargo audit` shows them.

Usage:
    python3 -I scripts/osv-check.py          # exit status 1 when something is found
"""
import collections
import json
import re
import subprocess
import sys
import urllib.request

OSV = "https://api.osv.dev/v1"


def post(url: str, body: dict) -> dict:
    req = urllib.request.Request(url, data=json.dumps(body).encode(), headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=120) as res:
        return json.load(res)


def get(url: str) -> dict:
    with urllib.request.urlopen(url, timeout=60) as res:
        return json.load(res)


def packages() -> dict:
    files = subprocess.run(["git", "ls-files"], capture_output=True, text=True, check=True).stdout.split()
    found = collections.defaultdict(set)
    for f in files:
        if f.endswith("Cargo.lock"):
            text = open(f, encoding="utf-8").read()
            for name, version in re.findall(r'\[\[package\]\]\nname = "([^"]+)"\nversion = "([^"]+)"', text):
                found[("crates.io", name, version)].add(f)
        elif re.search(r"requirements[^/]*\.txt$", f):
            for line in open(f, encoding="utf-8"):
                m = re.match(r"^\s*([A-Za-z0-9_.-]+)\s*==\s*([^\s;#]+)", line)
                if m:
                    found[("PyPI", m.group(1), m.group(2))].add(f)
    return found


def main() -> int:
    found = packages()
    keys = list(found)
    hits = collections.defaultdict(set)
    for i in range(0, len(keys), 1000):
        batch = keys[i : i + 1000]
        queries = [{"package": {"ecosystem": e, "name": n}, "version": v} for e, n, v in batch]
        for key, result in zip(batch, post(f"{OSV}/querybatch", {"queries": queries})["results"]):
            for vuln in result.get("vulns") or []:
                hits[key].add(vuln["id"])
    rows = []
    for key, ids in sorted(hits.items()):
        for vid in sorted(ids):
            detail = get(f"{OSV}/vulns/{vid}")
            ghsa = sorted({a for a in detail.get("aliases", []) + [vid] if a.startswith("GHSA")})
            rows.append((key, vid, ghsa, detail.get("summary", "")[:90]))
    rustsec_only = {vid for _, vid, ghsa, _ in rows if not ghsa}
    rows = [r for r in rows if r[2]]
    seen = set()
    for (eco, name, version), vid, ghsa, summary in rows:
        tag = (name, version, tuple(ghsa))
        if tag in seen:
            continue
        seen.add(tag)
        files = ", ".join(sorted(found[(eco, name, version)]))
        print(f"{name}@{version}  {' '.join(ghsa)}  {summary}\n    {files}")
    print(f"{len(seen)} GitHub advisories in {len(keys)} packages ({len(rustsec_only)} RustSec-only notices not listed)")
    return 1 if seen else 0


if __name__ == "__main__":
    sys.exit(main())
