#!/usr/bin/env python3
"""Write the license notices that ship with a Craft Studio build.

Walks the dependency graph `cargo build -p craft-studio` compiles for one target (normal and build
dependencies, no dev-dependencies) and writes THIRD-PARTY-LICENSES.txt: the upstream Craft repos
whose crates are compiled in, then every crates.io and git crate (name, version, license
expression, repository), then the text of all their license files, each distinct text once.

Usage:
    python3 -I scripts/third-party-licenses.py <out-file> [--target <triple>]
"""
import collections
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LICENSE_FILE = re.compile(r"^(licen[cs]e|copying|notice|unlicense|copyright)([-._].*)?$", re.IGNORECASE)


def host_triple() -> str:
    out = subprocess.run(["rustc", "-vV"], capture_output=True, text=True, check=True).stdout
    return next(line.split(":", 1)[1].strip() for line in out.splitlines() if line.startswith("host:"))


def metadata(target: str) -> dict:
    cmd = ["cargo", "metadata", "--format-version", "1", "--locked", "--filter-platform", target]
    out = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, check=True).stdout
    return json.loads(out)


def compiled_packages(meta: dict) -> list[dict]:
    """The packages craft-studio's build compiles: normal and build dependencies, transitively."""
    packages = {p["id"]: p for p in meta["packages"]}
    nodes = {n["id"]: n for n in meta["resolve"]["nodes"]}
    start = next(p["id"] for p in meta["packages"] if p["name"] == "craft-studio")
    seen, todo = {start}, [start]
    while todo:
        for dep in nodes[todo.pop()]["deps"]:
            if dep["pkg"] not in seen and any(k["kind"] in (None, "build") for k in dep["dep_kinds"]):
                seen.add(dep["pkg"])
                todo.append(dep["pkg"])
    return sorted((packages[i] for i in seen), key=lambda p: (p["name"], p["version"]))


def license_files(package: dict) -> list[Path]:
    folder = Path(package["manifest_path"]).parent
    files = {f for f in folder.iterdir() if f.is_file() and LICENSE_FILE.match(f.name)}
    if package.get("license_file"):
        named = folder / package["license_file"]
        if named.is_file():
            files.add(named)
    return sorted(files)


def add_text(texts: dict[str, str], users: dict[str, list[str]], path: Path, label: str) -> None:
    text = path.read_text(encoding="utf-8", errors="replace").strip()
    digest = hashlib.sha256(re.sub(r"\s+", " ", text).encode()).hexdigest()
    texts.setdefault(digest, text)
    users[digest].append(label)


def main() -> int:
    args = sys.argv[1:]
    if not args or args[0].startswith("-"):
        print(__doc__, file=sys.stderr)
        return 2
    out = Path(args[0])
    target = args[args.index("--target") + 1] if "--target" in args else host_triple()
    out.parent.mkdir(parents=True, exist_ok=True)

    texts: dict[str, str] = {}
    users: dict[str, list[str]] = collections.defaultdict(list)
    without_file: list[str] = []
    upstream: set[str] = set()
    crates: list[str] = []
    for package in compiled_packages(metadata(target)):
        label = f"{package['name']} {package['version']}"
        if package["source"] is None:
            relative = Path(package["manifest_path"]).relative_to(ROOT)
            if relative.parts[0] == "upstream":
                upstream.add(relative.parts[1])
            continue
        crates.append(f"{label:<48} {package.get('license') or 'see license files':<40} {package.get('repository') or ''}")
        files = license_files(package)
        if not files:
            without_file.append(label)
        for f in files:
            add_text(texts, users, f, f"{label} ({f.name})")
    for repo in sorted(upstream):
        files = [f for f in sorted((ROOT / "upstream" / repo).iterdir()) if f.is_file() and LICENSE_FILE.match(f.name)]
        if not files:
            print(f"warning: upstream/{repo} has no license file", file=sys.stderr)
        for f in files:
            add_text(texts, users, f, f"upstream {repo} ({f.name})")

    with open(out, "w", encoding="utf-8", newline="\n") as report:
        report.write(f"Craft Studio ({target}) is built from the Craft apps of github.com/storytold:\n\n")
        report.write("".join(f"  {repo}\n" for repo in sorted(upstream)))
        report.write(f"\nand bundles these {len(crates)} crates:\n\n")
        report.write("\n".join(crates) + "\n")
        if without_file:
            report.write("\nThese crates ship no license file; the license expression above applies (each license's\n")
            report.write("text is reproduced below, from the crates that ship it):\n")
            report.write("".join(f"  {label}\n" for label in without_file))
        for digest, text in texts.items():
            report.write("\n" + "=" * 100 + "\n")
            report.write("Used by: " + ", ".join(users[digest]) + "\n\n")
            report.write(text + "\n")

    print(f"{len(crates)} crates ({len(texts)} distinct license texts), {len(upstream)} upstream repos -> {out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
