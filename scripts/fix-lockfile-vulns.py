#!/usr/bin/env python3
"""Bump vulnerable crates to their newest semver-compatible release in Cargo lockfiles.

For each directory given (a Cargo workspace root, or any directory holding a Cargo.lock):
  1. `cargo update --workspace` reconciles the lockfile with patched manifests (minimal changes);
  2. `cargo audit` lists vulnerable packages (ignores come from .cargo/audit.toml);
  3. `cargo update -p name@version` takes each one to its newest compatible release;
  repeated until nothing changes. What remains needs a manifest change (a patch in patches/).

Usage:
    python3 -I scripts/fix-lockfile-vulns.py upstream/photocraft upstream/pdfcraft/vendor/winit
"""
import json
import subprocess
import sys
from pathlib import Path

MAX_ROUNDS = 5


def cargo(args: list[str], cwd: Path) -> subprocess.CompletedProcess:
    return subprocess.run(["cargo", *args], cwd=cwd, capture_output=True, text=True, check=False)


def vulnerable(lock: Path, cwd: Path) -> set[str]:
    out = cargo(["audit", "-n", "-q", "--json", "-f", str(lock)], cwd)
    try:
        report = json.loads(out.stdout)
    except json.JSONDecodeError:
        print(f"  cargo audit failed for {lock}: {out.stderr.strip()[:200]}", file=sys.stderr)
        return set()
    return {f"{v['package']['name']}@{v['package']['version']}" for v in report["vulnerabilities"]["list"]}


def fix(directory: Path, audit_cwd: Path) -> set[str]:
    lock = directory / "Cargo.lock"
    if not lock.is_file():
        return set()
    cargo(["update", "-q", "--workspace"], directory)
    remaining = vulnerable(lock.resolve(), audit_cwd)
    for _ in range(MAX_ROUNDS):
        if not remaining:
            break
        for spec in sorted(remaining):
            cargo(["update", "-q", "-p", spec], directory)
        after = vulnerable(lock.resolve(), audit_cwd)
        if after == remaining:
            break
        remaining = after
    return remaining


def main() -> int:
    repo_root = Path(__file__).resolve().parent.parent
    # Fetch the advisory database once; later calls run offline (-n).
    cargo(["audit", "-q", "-f", str(repo_root / "Cargo.lock")], repo_root)
    status = 0
    for arg in sys.argv[1:]:
        left = fix(Path(arg), repo_root)
        print(f"{arg}: {'clean' if not left else 'still vulnerable: ' + ', '.join(sorted(left))}")
        status |= bool(left)
    return status


if __name__ == "__main__":
    sys.exit(main())
