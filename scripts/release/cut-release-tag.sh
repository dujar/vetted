#!/usr/bin/env bash
# cut-release-tag.sh — cut the release tag on the SUBMITTED tree (step-10 task 1).
#
# Tag timing (plan Revised 2026-09-21, verify LE 4): the tag is cut at the
# END of packaging, on the tree the submission links point at — the merge /
# final commit on main. CI is read off the main push of the tagged commit.
#
# Post-tag rule: a blocking fix NEVER re-cuts the tag — bump the minor
# (v1.0.0 -> v1.0.1) with this script and update the submission links.
#
# Usage: bash scripts/release/cut-release-tag.sh [v1.0.0] [--skip-clean-checkout]
#   Requires: clean tree, on main, tag name unused, CI green for HEAD
#   (via `gh`; or pass --local-suite to run the full suite here instead).
set -euo pipefail

TAG="${1:-v1.0.0}"
SKIP_CLEAN=0; LOCAL_SUITE=0
for a in "$@"; do
  case "$a" in
    --skip-clean-checkout) SKIP_CLEAN=1 ;;
    --local-suite) LOCAL_SUITE=1 ;;
  esac
done

BRANCH="$(git rev-parse --abbrev-ref HEAD)"
[ "$BRANCH" = "main" ] || { echo "REFUSE: not on main (on $BRANCH) — the tag must name the submitted tree on main"; exit 1; }
[ -z "$(git status --porcelain)" ] || { echo "REFUSE: dirty tree — commit or stash first"; exit 1; }
git rev-parse -q --verify "refs/tags/$TAG" >/dev/null && { echo "REFUSE: $TAG already exists (never re-cut a tag — bump to the next version)"; exit 1; }
HEAD_SHA="$(git rev-parse --short HEAD)"
echo "cutting $TAG at main $HEAD_SHA"

# 1) CI green at the tagged commit (the gate: CI green on the main push).
if [ "$LOCAL_SUITE" -eq 0 ]; then
  echo "== CI gate (gh run list at $HEAD_SHA)"
  if command -v gh >/dev/null && gh run list --branch main --commit "$(git rev-parse HEAD)" --json conclusion,displayTitle 2>/dev/null | grep -q '"conclusion":"success"'; then
    echo "   CI green on main at $HEAD_SHA"
  else
    echo "REFUSE: no green CI run found for $HEAD_SHA (push to main and wait, or pass --local-suite)"; exit 1
  fi
else
  echo "== local suite gate (--local-suite)"
  ( cd contracts && cargo test --workspace >/dev/null && cargo build --workspace --target wasm32-unknown-unknown >/dev/null \
    && echo "   contracts: native tests + wasm build OK" )
  ( cd scan-backend && cargo test >/dev/null && echo "   scan-backend: tests OK" )
  ( cd packages/shared && npm test >/dev/null && cargo test >/dev/null && echo "   shared: both round-trips OK" )
  ( cd frontend && npm run build >/dev/null && npm test >/dev/null && echo "   frontend: build + vitest OK" )
fi

# 2) The tagged release builds from a clean checkout (plan check line).
if [ "$SKIP_CLEAN" -eq 0 ]; then
  echo "== clean-checkout build"
  TMPWT="$(mktemp -d)/clean"
  git worktree add --detach "$TMPWT" HEAD >/dev/null
  trap 'git worktree remove --force "$TMPWT"' EXIT
  ( cd "$TMPWT/contracts" && cargo build --workspace --target wasm32-unknown-unknown >/dev/null 2>&1 && echo "   contracts wasm build OK from clean checkout" )
  ( cd "$TMPWT/frontend" && npm ci --silent >/dev/null 2>&1 \
      && npm ci --silent --prefix "$TMPWT/packages/shared" >/dev/null 2>&1 \
      && npm run build --silent >/dev/null 2>&1 && echo "   frontend build OK from clean checkout" )
fi

# 3) The annotated freeze note.
git tag -a "$TAG" -m "v1.0.0 — Vetted submission freeze (Arbitrum Open House Singapore Buildathon; target 2026-10-03 EOD SGT)

Product code frozen except blocking fixes; no contract changes at tag time.
Known guard-test dead-code helpers (contracts/core/guard/src/lib.rs BEACON :655,
mock_record :706, install_beacon_token :733) are dispositioned follow-up
cleanup, deliberately not folded in at freeze — warnings, not failures.
The guard erc20() junk-returndata decode is DISCLOSED (README security note,
submission/qa-prep.md Q1) — funds-safe worst case, fix queued post-freeze.
Blocking fixes after this tag bump to v1.0.1 — never re-cut v1.0.0."

echo "tag $TAG created on $HEAD_SHA"
echo "next: git push origin main --follow-tags   (then record the tag in submission/README.md if links change)"
