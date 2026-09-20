#!/usr/bin/env bash
# seed-demo/seed.sh — idempotent registrar seeding of the 4663 demo state
# (step-9 plan task 1). The polished successor of e2e/scripts/seed-scratch.sh;
# same two-sided settle plumbing, signed by the DEMO keys, not the e2e ones.
#
#   REGISTRAR_KEY=0x… OPERATOR_KEY=0x… bash scripts/seed-demo/seed.sh
#   # optional: BUYER_KEY (default: OPERATOR_KEY — the funded demo-operator
#   #           wallet the runbook's bridge step already funds; NO anvil
#   #           default here, that key is dead weight on 4663),
#   #           DRY_RUN=1 (print the plan, send nothing), DEPLOYMENTS_FILE
#   #           (default deployments/4663.json; rehearsal override only —
#   #           never point it at a hand-forged file in the repo)
#
# PREREQUISITE (step-9 plan task 6 / Revised 2026-09-21 note 1): the funded-run
# runbook in .agent-workbench/step-7-contract-hardening/findings.md has
# executed — deployments/4663.json exists (registry + guard + the step-6
# replicas field) and the worker holds the post-handoff registrar key.
#
# Keys (Revised 2026-09-21 note 2): verify/revoke are signed by REGISTRAR_KEY
# — the NEW `cast wallet new` key the runbook's transfer_registrar handed the
# registry to; writes under the old deployer key revert after the handoff.
# OPERATOR_KEY is the funded demo operator (replica admin: mint/pause/
# setBlocked/upgradeTo) and the MM counterparty for the settle.
#
# What it seeds (the demo arc's standing state — red flags stay OFF, the beats
# drive them live per demo/runbook.md):
#   1. buyer gas (only when the buyer is a separate wallet)
#   2. registry: verify(replica proxy, impl = the beacon's CURRENT
#      implementation) — the record the upgrade beat later drifts + revokes
#   3. registry: verify(mockTokens.tokenB) then revoke(tokenB, …) — the
#      standing REVOKED record the registry page / REVOKED scan state shows;
#      the replica itself stays VERIFIED until the live upgrade beat
#   4. twins stay UNREGISTERED — the impostor refusal is GUARD_NO_RECORD
#   5. settle plumbing: buyer tokenA balance+allowance → guard, MM replica
#      balance+allowance → guard (ported from seed-scratch.sh)
# Every step is idempotent: state is read first, sends are skipped when the
# on-chain state already matches.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
CHAIN="4663"
DEP="${DEPLOYMENTS_FILE:-$ROOT/deployments/$CHAIN.json}"
RPC="https://rpc.mainnet.chain.robinhood.com"   # wire.md chains table
DRY_RUN="${DRY_RUN:-0}"

command -v cast >/dev/null || { echo "foundry's cast is required (scripts/toolchain.sh)" >&2; exit 1; }
command -v jq >/dev/null || { echo "jq is required" >&2; exit 1; }
command -v awk >/dev/null || { echo "awk is required" >&2; exit 1; }

: "${REGISTRAR_KEY:?set REGISTRAR_KEY (the NEW post-handoff registrar key — funded-run runbook step 2)}"
: "${OPERATOR_KEY:?set OPERATOR_KEY (the funded demo operator — replica admin, MM, funder)}"
BUYER_KEY="${BUYER_KEY:-$OPERATOR_KEY}"

[ -f "$DEP" ] || { echo "missing $DEP — the funded-run runbook (step-7 findings) has not executed yet; this seeder is task-6-gated" >&2; exit 1; }
STATUS=$(jq -r '.status' "$DEP"); [ "$STATUS" = "deployed" ] || { echo "$DEP status=$STATUS — not a funded deploy" >&2; exit 1; }

REGISTRY=$(jq -r '.registry // empty' "$DEP")
GUARD=$(jq -r '.guard // empty' "$DEP")
REPLICA=$(jq -r '.replicas.proxy // empty' "$DEP")
REPLICA_BEACON=$(jq -r '.replicas.beacon // empty' "$DEP")
IMPL_V2=$(jq -r '.replicas.implV2 // empty' "$DEP")
TWIN1=$(jq -r '.replicas.twin1 // empty' "$DEP")
TOKEN_A=$(jq -r '.mockTokens.tokenA // empty' "$DEP")   # settle tokenIn
TOKEN_B=$(jq -r '.mockTokens.tokenB // empty' "$DEP")   # the standing REVOKED record
[ -n "$REGISTRY" ] && [ -n "$GUARD" ] || { echo "$DEP has no core deploy (registry/guard) — runbook steps 3-4 first" >&2; exit 1; }
[ -n "$REPLICA" ] && [ -n "$REPLICA_BEACON" ] || { echo "$DEP has no .replicas.proxy/.replicas.beacon — 4663 replicas deploy first (demo/runbook.md)" >&2; exit 1; }

REGISTRAR=$(cast wallet address --private-key "$REGISTRAR_KEY")
OPERATOR=$(cast wallet address --private-key "$OPERATOR_KEY")
BUYER=$(cast wallet address --private-key "$BUYER_KEY")
MAX_UINT=115792089237316195423570985008687907853269984665640564039457584007913129639935
MINT=1000000000000000000000   # 1000 units, 18 decimals

log() { printf '\n== %s\n' "$*"; }

# uint256-safe "a < b" (bash arithmetic is 64-bit — never $(( )) these values)
lt() { awk -v a="$1" -v b="$2" 'BEGIN{exit !(a<b)}'; }

call() { # call WHAT TO SIG ARGS... — read, honoring dry-run ("0" = the
         # harmless default that routes balance checks down the seed path)
  local what="$1" to="$2" sig="$3"; shift 3
  if [ "$DRY_RUN" = "1" ]; then printf '[dry-run] cast call %s "%s" %s\n' "$to" "$sig" "$*" >&2; echo "0"; return 0; fi
  cast call "$to" "$sig" "$@" --rpc-url "$RPC"
}

send() { # send KEY WHAT TO SIG ARGS...  (empty SIG = plain ETH transfer)
  local key="$1" what="$2" to="$3" sig="$4"; shift 4
  if [ "$DRY_RUN" = "1" ]; then
    local from; from=$(cast wallet address --private-key "$key")
    if [ -n "$sig" ]; then
      printf '[dry-run] cast send %s "%s" %s  (from %s)\n' "$to" "$sig" "$*" "$from"
    else
      printf '[dry-run] cast send %s %s  (from %s)\n' "$to" "$*" "$from"
    fi
    return 0
  fi
  if [ -n "$sig" ]; then
    cast send "$to" "$sig" "$@" --private-key "$key" --rpc-url "$RPC" --json >/dev/null
  else
    cast send "$to" "$@" --private-key "$key" --rpc-url "$RPC" --json >/dev/null
  fi
}

record_status() { # token → 0 VERIFIED / 1 REVOKED / "" no record
  if [ "$DRY_RUN" = "1" ]; then echo ""; return 0; fi
  local rec
  rec=$(cast call "$REGISTRY" "getRecord(address)(uint8,uint256,uint64,address,address,uint64,bytes32)" "$1" --rpc-url "$RPC" 2>/dev/null || echo "")
  rec="${rec#\(}"; echo "${rec%%,*}"
}

log "chain $CHAIN — registry $REGISTRY · guard $GUARD"
log "registrar $REGISTRAR · operator $OPERATOR · buyer $BUYER"

# 1. buyer gas (a distinct buyer wallet needs gas for approve + the on-camera swap)
if [ "$BUYER" != "$OPERATOR" ]; then
  log "1. buyer ETH"
  BAL=0
  [ "$DRY_RUN" = "1" ] || BAL=$(cast balance "$BUYER" --rpc-url "$RPC")
  if [ "$BAL" = "0" ]; then
    send "$OPERATOR_KEY" "fund buyer" "$BUYER" "" --value 0.02ether
  fi
fi

# 2. verify(replica) — registrar-signed (the handoff key, Revised note 2).
log "2. registry: verify(replica $REPLICA)"
ST=$(record_status "$REPLICA")
if [ "$ST" != "0" ]; then
  IMPL=$(call "beacon implementation()" "$REPLICA_BEACON" "implementation()(address)")
  send "$REGISTRAR_KEY" "verify(replica)" "$REGISTRY" "verify(address,uint256,address)" "$REPLICA" 0 "$IMPL"
else
  echo "replica already VERIFIED — skipping"
fi

# 3. the standing REVOKED record on mock tokenB (the replica is revoked LIVE
#    in beat 6 — never pre-revoke it).
log "3. registry: verify+revoke(tokenB $TOKEN_B) — standing REVOKED record"
ST_B=$(record_status "$TOKEN_B")
if [ "$ST_B" != "0" ] && [ "$ST_B" != "1" ]; then
  if [ "$DRY_RUN" = "1" ]; then IMPL_B="0"; else
    IMPL_B=$(cast call "$TOKEN_B" "implementation()(address)" --rpc-url "$RPC" 2>/dev/null \
      || cast call "$(jq -r '.mockTokens.beacon' "$DEP")" "implementation()(address)" --rpc-url "$RPC")
  fi
  send "$REGISTRAR_KEY" "verify(tokenB)" "$REGISTRY" "verify(address,uint256,address)" "$TOKEN_B" 0 "$IMPL_B"
  ST_B=0
fi
if [ "$ST_B" = "0" ]; then
  send "$REGISTRAR_KEY" "revoke(tokenB)" "$REGISTRY" "revoke(address,string)" "$TOKEN_B" "beacon impl changed — record stale"
else
  echo "tokenB already REVOKED — skipping"
fi
# twins stay unregistered — GUARD_NO_RECORD is the impostor beat's refusal

# 4. settle plumbing: buyer tokenA, MM replica, allowances → guard
#    (ported from e2e/scripts/seed-scratch.sh — Revised 2026-09-21 note 4)
log "4. settle plumbing: buyer tokenA, MM replica, allowances → guard"
BAL_A=$(call "tokenA balanceOf(buyer)" "$TOKEN_A" "balanceOf(address)(uint256)" "$BUYER")
if lt "$BAL_A" "$MINT"; then
  send "$OPERATOR_KEY" "mint tokenA → buyer" "$TOKEN_A" "mint(address,uint256)" "$BUYER" "$MINT"
fi
ALLOW_A=$(call "tokenA allowance(buyer→guard)" "$TOKEN_A" "allowance(address,address)(uint256)" "$BUYER" "$GUARD")
if [ "$ALLOW_A" != "$MAX_UINT" ]; then
  send "$BUYER_KEY" "approve tokenA" "$TOKEN_A" "approve(address,uint256)" "$GUARD" "$MAX_UINT"
fi

BAL_R=$(call "replica balanceOf(MM)" "$REPLICA" "balanceOf(address)(uint256)" "$OPERATOR")
if lt "$BAL_R" "$MINT"; then
  send "$OPERATOR_KEY" "mint replica → MM" "$REPLICA" "mint(address,uint256)" "$OPERATOR" "$MINT"
fi
ALLOW_R=$(call "replica allowance(MM→guard)" "$REPLICA" "allowance(address,address)(uint256)" "$OPERATOR" "$GUARD")
if [ "$ALLOW_R" != "$MAX_UINT" ]; then
  send "$OPERATOR_KEY" "approve replica" "$REPLICA" "approve(address,uint256)" "$GUARD" "$MAX_UINT"
fi

log "5. seeded. beat addresses (full choreography: demo/runbook.md)"
cat <<EOF
  replica (scan VERIFIED, swap settle, upgrade beat): $REPLICA
  replica beacon (setBlocked/pause/upgradeTo, OPERATOR key):  $REPLICA_BEACON
  upgrade target implV2: $IMPL_V2
  twin1 (scan UNVERIFIED, swap GUARD_NO_RECORD): $TWIN1
  tokenB (scan REVOKED, registry red row):       $TOKEN_B
  settle tokenIn (tokenA):                       $TOKEN_A
  worker blocklist-probe buyer (setBlocked THIS address for the scan's PRESENT row):
    0xd8da6bf26964af9d7eed9e03e53415d37aa96045  (the engine's fixed FRESH_BUYER, scan-backend/src/lib.rs:28)
  remaining operator steps: worker DRIFT_EXTRA_TOKENS --var (runbook), demo/serve.sh, then demo/arc.sh
EOF
