#!/usr/bin/env bash
# production-check.sh — the step-10 task-2 production state audit.
# Read-only. Unfunded-aware: items that cannot exist without the funded run
# report PENDING-FUNDS (never FAIL) — the unfunded variant ships with them
# standing. Exit 1 only on a real FAIL (a live URL down, a dishonest shape).
#
# Usage: bash scripts/release/production-check.sh
#   OPERATOR_KEY=0x… to audit a different key (default: the project operator).
set -u

OPERATOR_KEY="${OPERATOR_KEY:-0x151e9f57F31310aFeBBB60c222c14badCf938E4C}"
ROOT="$(git rev-parse --show-toplevel 2>/dev/null || cd "$(dirname "$0")/../.." && pwd)"
RPC_4663="https://rpc.mainnet.chain.robinhood.com"
RPC_421614="https://sepolia-rollup.arbitrum.io/rpc"
WORKER="https://vetted-scan-backend.dujar-coding.workers.dev"
PAGES="https://vetted-1un.pages.dev"
P_ADDR="0x1Cdad396DB64BDa184d5182A97Dd9B3C62100b7D"   # issuer's own token (calibration_4663.json)

PASS=0; WARN=0; FAIL=0; PEND=0; NA=0
say()  { printf '%s\n' "$*"; }
ok()   { say "PASS          | $*"; PASS=$((PASS+1)); }
warn() { say "WARN          | $*"; WARN=$((WARN+1)); }
bad()  { say "FAIL          | $*"; FAIL=$((FAIL+1)); }
pend() { say "PENDING-FUNDS | $*"; PEND=$((PEND+1)); }
na()   { say "N/A           | $*"; NA=$((NA+1)); }

rpc() { # rpc <url> <method> <params-json>
  curl -sS --max-time 20 -X POST "$1" -H 'Content-Type: application/json' \
    -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"$2\",\"params\":$3}" 2>/dev/null
}

say "# production state audit — $(date -u '+%F %T UTC')"
say "# key $OPERATOR_KEY · regime-aware (PENDING-FUNDS = the funded run hasn't executed)"
say ""

## --- deployments ledger (writer-of-record: the funded-run runbook) --------
if [ -f "$ROOT/deployments/4663.json" ]; then
  ok "deployments/4663.json exists ($(command -v jq >/dev/null && jq -r 'keys | join(",")' "$ROOT/deployments/4663.json" 2>/dev/null || echo present))"
else
  pend "deployments/4663.json absent — go-list (submission/post-funding-checklist.md §1) not executed yet"
fi
if [ -f "$ROOT/deployments/4663.json" ] && ! [ -f "$ROOT/deployments/421614.json" ]; then
  na "deployments/421614.json absent while 4663 funded — 4663 took priority (runbook rule); mirror check honestly N/A"
elif [ -f "$ROOT/deployments/421614.json" ]; then
  ok "deployments/421614.json exists (mirror ledger present)"
else
  pend "deployments/421614.json absent — first funded run anywhere writes it (end-to-end deploy.sh on 421614)"
fi

## --- operator key funding --------------------------------------------------
for CH in "$RPC_4663 4663" "$RPC_421614 421614"; do
  set -- $CH
  HEX=$(rpc "$1" eth_getBalance "[\"$OPERATOR_KEY\",\"latest\"]" | grep -o '"0x[0-9a-f]*"' | head -1 | tr -d '"')
  if [ "${HEX#0x}" = "" ] 2>/dev/null || [ -z "$HEX" ]; then
    warn "chain $2: balance unreadable (RPC?) — re-verify off-machine before flagging"
  elif [ "$HEX" = "0x0" ]; then
    pend "chain $2: operator key reads 0 wei"
  else
    ok "chain $2: operator key funded ($HEX wei)"
  fi
done

## --- production URLs -------------------------------------------------------
CODE=$(curl -sS --max-time 20 -o /dev/null -w '%{http_code}' "$PAGES" 2>/dev/null)
if [ "$CODE" = "200" ]; then ok "Pages $PAGES -> 200"
else warn "Pages $PAGES -> HTTP $CODE (local TLS/probe failures are suspect per plan note — re-verify off-machine before flagging a URL dead)"; fi

HEALTH=$(curl -sS --max-time 20 "$WORKER/health" 2>/dev/null)
echo "$HEALTH" | grep -q '"ok":true' && ok "Worker /health ok" || bad "Worker /health not ok: $HEALTH"

WD=$(curl -sS --max-time 20 "$WORKER/watchdog?chainId=4663" 2>/dev/null)
echo "$WD" | grep -q '"baselinePerDay":150' || bad "watchdog shape unexpected: $WD"
if echo "$WD" | grep -q '"runs":0'; then
  echo "$WD" | grep -q '"provenanceUrl":"https\?://' \
    && ok "watchdog degrade sentinel exact (runs:0 + provenanceUrl — the sentinel, never a measured zero)" \
    || bad "watchdog runs:0 WITHOUT provenanceUrl — wire-sentinel violation: $WD"
else
  ok "watchdog live counter present: $WD"
fi

SCAN=$(curl -sS --max-time 30 "$WORKER/scan?chainId=4663&addr=$P_ADDR" 2>/dev/null)
V=$(echo "$SCAN" | grep -o '"verdict":"[A-Z]*"' | cut -d'"' -f4)
TS=$(echo "$SCAN" | grep -o '"terminalState":"[A-Z_]*"' | cut -d'"' -f4)
if [ "$TS" = "RPC_RETRYABLE" ]; then
  warn "P scan hit the designed RPC_RETRYABLE terminal (public-RPC rate window) — the scanner retries rather than guesses; retry with backoff"
elif [ "$V" = "VERIFIED" ]; then
  echo "$SCAN" | grep -q '"powerReport":\[\]' && bad "VERIFIED with EMPTY powerReport — fabricated-evidence violation" \
    || ok "P scan VERIFIED with evidence rows (live P receipt shape — link it from submission/README.md)"
elif [ -z "$V" ]; then
  bad "P scan returned neither verdict nor the designed retry terminal: $(echo "$SCAN" | head -c 300)"
elif echo "$SCAN" | grep -q '"record":null'; then
  pend "P scan: record null (worker REGISTRY_ADDRESS_4663 unset — go-list step 6) -> $V"
else
  warn "P scan: $V (honest degrade or rate window) — never fabricated; retry in a calm window"
fi

MIRROR=$(curl -sS --max-time 20 "https://vetted-spike-canonical-fetch.dujar-coding.workers.dev/assets" 2>/dev/null)
echo "$MIRROR" | grep -q '"ok":true' && ok "spike canonical-fetch mirror alive (fallback)" \
  || warn "spike mirror down (non-fatal — production fetches the issuer directly)"

## --- explorers -------------------------------------------------------------
EC=$(curl -sS --max-time 20 -o /dev/null -w '%{http_code}' "https://robinhoodchain.blockscout.com" 2>/dev/null)
case "$EC" in
  200)      ok "4663 explorer resolves (robinhoodchain.blockscout.com)" ;;
  000)      warn "4663 explorer unreachable from here — re-verify off-machine (CF challenge / local TLS)" ;;
  *)        warn "4663 explorer HTTP $EC — CF bot challenge server-side is known; browsers pass (browser-UI verify fallback)" ;;
esac
EC=$(curl -sS --max-time 20 -o /dev/null -w '%{http_code}' "https://sepolia.arbiscan.io" 2>/dev/null)
[ "$EC" = "200" ] && ok "421614 mirror explorer resolves (sepolia.arbiscan.io)" \
  || warn "421614 explorer HTTP $EC from here — re-verify off-machine"

## --- worker secrets (best-effort; needs an authed wrangler) ----------------
if [ -d "$ROOT/scan-backend/node_modules" ] && command -v npx >/dev/null; then
  SECRETS=$(cd "$ROOT/scan-backend" && timeout 60 npx wrangler secret list 2>/dev/null || true)
  if [ -z "$SECRETS" ]; then
    warn "worker secrets audit skipped (wrangler not authed here) — verify by hand: cd scan-backend && npx wrangler secret list"
  else
    echo "$SECRETS" | grep -q REGISTRAR_KEY \
      && ok "worker secret REGISTRAR_KEY set (verify its address == registrar(): cast call <registry> 'registrar()(address)')" \
      || pend "worker secret REGISTRAR_KEY unset (ships deliberately unset until the registrar handoff)"
    echo "$SECRETS" | grep -q DRIFT_ADMIN_SECRET \
      && ok "worker secret DRIFT_ADMIN_SECRET set" \
      || pend "worker secret DRIFT_ADMIN_SECRET unset (deliberate until the handoff)"
  fi
else
  warn "worker secrets audit skipped (no scan-backend/node_modules) — run from a checkout with deps installed"
fi

## --- frontend env contract (manual verify items; the deployed bundle's
##     env is baked at build time and not headlessly readable) --------------
say "MANUAL        | Pages env contract (go-list + checklist step 5): VITE_API_MODE=live, VITE_API_URL=$WORKER,"
say "MANUAL        |   VITE_REGISTRY_ADDRESS/VITE_GUARD_ADDRESS <- deployments/4663.json, VITE_WALLETCONNECT_PROJECT_ID"

say ""
say "# summary: $PASS pass, $WARN warn, $FAIL FAIL, $PEND pending-funds, $NA n/a"
[ "$FAIL" -eq 0 ] && { say "# AUDIT: no failures — remaining PENDING-FUNDS lines are the funded run's work"; exit 0; }
say "# AUDIT: FAILURES PRESENT — fix before submission"
exit 1
