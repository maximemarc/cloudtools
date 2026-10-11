#!/usr/bin/env bash
# Fork every ArtCraft repository from github.com/storytold into your account (or an org).
#
# Usage:
#   ./scripts/fork-artcraft.sh               # forks into the authenticated gh user
#   ./scripts/fork-artcraft.sh --org futur-code
#   DRY_RUN=1 ./scripts/fork-artcraft.sh     # print what would be forked
#
# Requires: gh >= 2.40, authenticated (`gh auth login`).
set -euo pipefail

UPSTREAM_OWNER="storytold"

# ArtCraft family only (legacy FakeYou / Unreal / voice repos are excluded on purpose).
REPOS=(
  # ArtCraft AI desktop app + backend
  artcraft
  artcraft-services
  artcraftx
  cloud-worker
  # Craft App Suite (clean-room Rust reimplementations)
  photocraft
  lightcraft
  filmcraft
  pdfcraft
  vectorcraft
  effectcraft
  wordcraft
  designcraft
  gridcraft
  cadcraft
  deckcraft
  soundcraft
  # Shared infra
  craft-libs
  craft-launcher
  craft-fonts
  craft-repo-status-app
  photocraft-corpus
)

ORG=""
if [[ "${1:-}" == "--org" ]]; then
  ORG="${2:?--org requires a value}"
fi

command -v gh >/dev/null || { echo "gh CLI not found" >&2; exit 1; }
gh auth status >/dev/null 2>&1 || { echo "gh is not authenticated: run 'gh auth login'" >&2; exit 1; }

target_owner="${ORG:-$(gh api user --jq .login)}"
failed=()

for repo in "${REPOS[@]}"; do
  if gh repo view "${target_owner}/${repo}" >/dev/null 2>&1; then
    echo "SKIP  ${target_owner}/${repo} (already exists)"
    continue
  fi

  if [[ -n "${DRY_RUN:-}" ]]; then
    echo "DRY   ${UPSTREAM_OWNER}/${repo} -> ${target_owner}/${repo}"
    continue
  fi

  args=(repo fork "${UPSTREAM_OWNER}/${repo}" --clone=false --default-branch-only)
  [[ -n "$ORG" ]] && args+=(--org "$ORG")

  if gh "${args[@]}" >/dev/null 2>&1; then
    echo "OK    ${UPSTREAM_OWNER}/${repo} -> ${target_owner}/${repo}"
  else
    echo "FAIL  ${UPSTREAM_OWNER}/${repo}" >&2
    failed+=("$repo")
  fi
  sleep 1 # stay well under the secondary rate limit on fork creation
done

if ((${#failed[@]})); then
  echo "Failed: ${failed[*]}" >&2
  exit 1
fi
