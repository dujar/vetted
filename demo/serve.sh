#!/usr/bin/env bash
# demo/serve.sh — the DEMO serving mode (verify loose end 1): an env-baked
# local serve of the real frontend bundle, the e2e live-server pattern
# (e2e/playwright.config.ts) applied to the demo. The production Pages bundle
# (vetted-1un.pages.dev) is the product-default MOCK build — the live beats
# need the env baked at serve time.
#
#   VITE_WALLETCONNECT_PROJECT_ID=… bash demo/serve.sh
#   # optional: PORT (default 4885 — clear of e2e's 4883/4884)
#
# Bakes, from deployments/4663.json (the funded-run runbook's artifact):
#   VITE_API_MODE=live            VITE_API_URL=<production worker>
#   VITE_CHAIN_ID=4663            VITE_REGISTRY_ADDRESS / VITE_GUARD_ADDRESS
#   VITE_REGISTRY_TOKENS=<replica,twin1,twin2,tokenA,tokenB>  (named rows in
#                                 the registry table + swap selectors)
# Deliberately NOT set: VITE_E2E_STUB_WALLET — the demo uses a REAL injected
# wallet (import the funded operator key into the browser profile; the runbook
# covers the profile setup).
#
# Wallet note: without VITE_WALLETCONNECT_PROJECT_ID wagmi boots with zero
# connectors (frontend/src/lib/wallet.ts) — scan beats work wallet-free, the
# swap beat needs the connector, so the runbook fails this script without it.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEP="$ROOT/deployments/4663.json"
PORT="${PORT:-4885}"

command -v jq >/dev/null || { echo "jq is required" >&2; exit 1; }
[ -f "$DEP" ] || { echo "missing $DEP — the funded-run runbook (step-7 findings) has not executed yet (task-6 gate)" >&2; exit 1; }
[ "$(jq -r '.status' "$DEP")" = "deployed" ] || { echo "$DEP is not a funded deploy" >&2; exit 1; }

REGISTRY=$(jq -r '.registry // empty' "$DEP")
GUARD=$(jq -r '.guard // empty' "$DEP")
[ -n "$REGISTRY" ] && [ -n "$GUARD" ] || { echo "$DEP lacks registry/guard — runbook steps 3-4 first" >&2; exit 1; }

TOKENS=$(jq -r '[.replicas.proxy, .replicas.twin1, .replicas.twin2,
                 .mockTokens.tokenA, .mockTokens.tokenB]
                | map(select(. != null and . != "")) | join(",")' "$DEP")

if [ -z "${VITE_WALLETCONNECT_PROJECT_ID:-}" ]; then
  echo "WARNING: VITE_WALLETCONNECT_PROJECT_ID unset — no wallet connector; the swap beat cannot connect." >&2
  echo "         (operator action, frontend/src/lib/wallet.ts — the runbook's pre-demo checklist names it)" >&2
fi

echo "demo serve :$PORT — registry $REGISTRY · guard $GUARD"
echo "registry tokens: $TOKENS"
cd "$ROOT/frontend"
exec env \
  VITE_API_MODE=live \
  VITE_API_URL=https://vetted-scan-backend.dujar-coding.workers.dev \
  VITE_CHAIN_ID=4663 \
  VITE_REGISTRY_ADDRESS="$REGISTRY" \
  VITE_GUARD_ADDRESS="$GUARD" \
  VITE_REGISTRY_TOKENS="$TOKENS" \
  npm run dev -- --port "$PORT" --strictPort --host
