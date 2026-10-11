#!/usr/bin/env bash
# Import or refresh the upstream ArtCraft repositories declared in upstream/repos.conf.
#
# Usage:
#   ./scripts/upstream-sync.sh                 # every repo
#   ./scripts/upstream-sync.sh photocraft pdfcraft
#   UPSTREAM_CACHE=/path/to/clones ./scripts/upstream-sync.sh
#
# vendor:    upstream/<name> is replaced by the tracked files of the upstream branch HEAD.
# submodule: upstream/<name> is a git submodule moved to the upstream branch HEAD.
# Pins are written to upstream/UPSTREAM.lock. Nothing is committed.
set -euo pipefail

UPSTREAM_OWNER="storytold"
ROOT="$(git rev-parse --show-toplevel)"
CONF="$ROOT/upstream/repos.conf"
LOCK="$ROOT/upstream/UPSTREAM.lock"
CACHE="${UPSTREAM_CACHE:-${XDG_CACHE_HOME:-$HOME/.cache}/cloudtools-upstream}"

cd "$ROOT"
mkdir -p "$CACHE"
[[ -f "$LOCK" ]] || printf '# name\tmode\tcommit\tsynced_at\turl\n' > "$LOCK"

declare -A WANTED=()
for n in "$@"; do WANTED[$n]=1; done

update_lock() {
  local name=$1 mode=$2 sha=$3 url=$4 tmp
  tmp="$(mktemp)"
  awk -v n="$name" -F'\t' '$1 != n' "$LOCK" > "$tmp"
  printf '%s\t%s\t%s\t%s\t%s\n' "$name" "$mode" "$sha" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$url" >> "$tmp"
  { head -1 "$tmp"; tail -n +2 "$tmp" | sort; } > "$LOCK"
  rm -f "$tmp"
}

sync_vendor() {
  local name=$1 branch=$2 url=$3 clone="$CACHE/$1" dest="upstream/$1" sha
  if git -C "$clone" rev-parse HEAD >/dev/null 2>&1; then
    GIT_LFS_SKIP_SMUDGE=1 git -C "$clone" fetch -q --depth 1 origin "$branch"
    git -C "$clone" reset -q --hard FETCH_HEAD
  else
    GIT_LFS_SKIP_SMUDGE=1 git clone -q --depth 1 --branch "$branch" "$url" "$clone"
  fi
  sha="$(git -C "$clone" rev-parse HEAD)"

  rm -rf -- "$dest"
  mkdir -p "$dest"
  # Tracked files only: no .git, no local build output, symlinks and modes preserved.
  (cd "$clone" && git ls-files -z) | tar -C "$clone" --null -T - -cf - | tar -C "$dest" -xf -
  update_lock "$name" vendor "$sha" "$url"
  echo "vendor     $name @ ${sha:0:12}"
}

sync_submodule() {
  local name=$1 branch=$2 url=$3 dest="upstream/$1" sha
  if [[ ! -e "$dest/.git" ]]; then
    git submodule add -q -b "$branch" --depth 1 "$url" "$dest"
  else
    git submodule update -q --remote --depth 1 -- "$dest"
  fi
  sha="$(git -C "$dest" rev-parse HEAD)"
  update_lock "$name" submodule "$sha" "$url"
  echo "submodule  $name @ ${sha:0:12}"
}

while read -r name mode branch _license <&3; do
  [[ -z "$name" || "$name" == \#* ]] && continue
  [[ "$name" =~ ^[a-z0-9-]+$ ]] || { echo "invalid repo name: $name" >&2; exit 1; }
  (( $# == 0 )) || [[ -n "${WANTED[$name]:-}" ]] || continue

  url="https://github.com/${UPSTREAM_OWNER}/${name}"
  case "$mode" in
    vendor)    sync_vendor "$name" "$branch" "$url" ;;
    submodule) sync_submodule "$name" "$branch" "$url" ;;
    *)         echo "unknown mode '$mode' for $name" >&2; exit 1 ;;
  esac
done 3< "$CONF"
