#!/usr/bin/env bash
# seed-scratch.sh — seeded demo state for the e2e scratch journeys (plan task 5).
#
# Seeded ad hoc here with the registrar key from step 3's integration deploys;
# the polished, idempotent 4663 version lands as step 9's scripts/seed-demo/
# — hand this working version over. GATED (plan Revised 2026-09-20 note 3):
# requires the FUNDED step-3/step-6 scratch deployments (deployments/46630.json
# or 421614.json) — do not hand-forge deployment entries.
#
#   CHAIN=46630 DEPLOY_KEY=0x… bash e2e/scripts/seed-scratch.sh
#   # optional: BUYER_KEY (default: the public anvil well-known e2e key),
#   #           MM_KEY (default: DEPLOY_KEY — the deploy script's own fallback)
#
# What it seeds (journeys + demo arc):
#   1. funds the e2e buyer (ETH)
#   2. registry: verify(replica); verify(tokenB) then revoke(tokenB, …)
#   3. token plumbing for the J2 settle: buyer tokenIn balance+allowance,
#      MM tokenOut balance+allowance (verify loose end 8)
#   4. the red-flag states, in the order the spec replays them — guard order
#      is record → paused → blocklist → impl-match, so IMPL_MISMATCH lands
#      BEFORE PAUSED (once paused, GUARD_PAUSED always preempts):
#      NO_RECORD (twin) → RECORD_REVOKED (tokenB) → IMPL_MISMATCH (shared mock
#      beacon → implV2) → PAUSED (tokenA) → BLOCKLISTED (buyer, last)
#   5. writes e2e/.scratch-state.json — the gated spec's data file
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
CHAIN="${CHAIN:-46630}"
DEP="$ROOT/deployments/$CHAIN.json"

case "$CHAIN" in
  46630) RPC="https://rpc.testnet.chain.robinhood.com" ;;
  421614) RPC="https://sepolia-rollup.arbitrum.io/rpc" ;;
  *) echo "unsupported scratch chain: $CHAIN (wire.md chains table)" >&2; exit 1 ;;
esac

command -v cast >/dev/null || { echo "foundry's cast is required (scripts/toolchain.sh)" >&2; exit 1; }
# NOTE: no apostrophes inside the ${VAR:?…} expansions — bash treats them as
# quote characters there (found by bash -n; the script is funding-gated so it
# had never been parsed on a real run).
: "${DEPLOY_KEY:?set DEPLOY_KEY (the registrar — the step-3 deploy key)}"

[ -f "$DEP" ] || { echo "missing $DEP — run the funded step-3/step-6 deploys first (e2e/README.md)" >&2; exit 1; }
STATUS=$(jq -r '.status' "$DEP"); [ "$STATUS" = "deployed" ] || { echo "$DEP status=$STATUS — not a funded deploy" >&2; exit 1; }

REGISTRY=$(jq -r '.registry' "$DEP")
GUARD=$(jq -r '.guard' "$DEP")
BEACON=$(jq -r '.mockTokens.beacon // empty' "$DEP")
IMPL_V2=$(jq -r '.mockTokens.implV2 // empty' "$DEP")
TOKEN_A=$(jq -r '.mockTokens.tokenA // empty' "$DEP")   # pattern token (pay token / paused / mismatch)
TOKEN_B=$(jq -r '.mockTokens.tokenB // empty' "$DEP")   # pattern token (the pre-revoked record)
TWIN1=$(jq -r '.replicas.twin1 // .replicas.twinPlain // empty' "$DEP")
REPLICA=$(jq -r '.replicas.proxy // empty' "$DEP")
REPLICA_BEACON=$(jq -r '.replicas.beacon // empty' "$DEP")
[ -n "$REPLICA" ] || { echo "no .replicas.proxy in $DEP — run the step-6 replicas deploy first" >&2; exit 1; }
[ -n "$REPLICA_BEACON" ] || { echo "no .replicas.beacon in $DEP — the buyer-blocklist beat needs the replica's beacon" >&2; exit 1; }

REGISTRAR=$(cast wallet address --private-key "$DEPLOY_KEY")
BUYER_KEY="${BUYER_KEY:-0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80}" # anvil well-known — e2e-only
BUYER=$(cast wallet address --private-key "$BUYER_KEY")
MM_KEY="${MM_KEY:-$DEPLOY_KEY}"  # same fallback rule as scripts/deploy/core/deploy.sh
MM=$(cast wallet address --private-key "$MM_KEY")

send() { # send KEY CONTRACT SIG ARGS...
  local key="$1" to="$2" sig="$3"; shift 3
  cast send "$to" "$sig" "$@" --private-key "$key" --rpc-url "$RPC" --json >/dev/null
}

log() { printf '\n== %s\n' "$*"; }

log "chain $CHAIN — registry $REGISTRY · guard $GUARD · buyer $BUYER · MM $MM"

# 1. fund the e2e buyer (gas for the journey)
log "1. buyer ETH"
BAL=$(cast balance "$BUYER" --rpc-url "$RPC")
if [ "$BAL" = "0" ]; then
  cast send "$BUYER" --value 0.1ether --private-key "$DEPLOY_KEY" --rpc-url "$RPC" --json >/dev/null
fi

# 2. registry records (registrar = the deploy key)
log "2. registry: verify(replica), verify+revoke(tokenB)"
IMPL=$(cast call "$REPLICA" "implementation()(address)" --rpc-url "$RPC" 2>/dev/null || cast call "$BEACON" "implementation()(address)" --rpc-url "$RPC")
REC=$(cast call "$REGISTRY" "getRecord(address)(uint8,uint256,uint64,address,address,uint64,bytes32)" "$REPLICA" --rpc-url "$RPC" 2>/dev/null || echo "")
if ! echo "$REC" | grep -q "^0, "; then
  send "$DEPLOY_KEY" "$REGISTRY" "verify(address,uint256,address)" "$REPLICA" 0 "$IMPL"
fi
REC_B=$(cast call "$REGISTRY" "getRecord(address)(uint8,uint256,uint64,address,address,uint64,bytes32)" "$TOKEN_B" --rpc-url "$RPC" 2>/dev/null || echo "")
if ! echo "$REC_B" | grep -q "^0, "; then
  send "$DEPLOY_KEY" "$REGISTRY" "verify(address,uint256,address)" "$TOKEN_B" 0 "$IMPL"
fi
if ! echo "$REC_B" | grep -q "^1, "; then
  send "$DEPLOY_KEY" "$REGISTRY" "revoke(address,string)" "$TOKEN_B" "beacon impl changed — record stale"
fi

# 3. J2 settle plumbing: buyer tokenIn (tokenA) balance + allowance to the
#    guard; MM tokenOut (replica) balance + allowance (verify loose end 8).
log "3. token plumbing: buyer tokenA, MM replica, allowances → guard"
send "$DEPLOY_KEY" "$TOKEN_A" "mint(address,uint256)" "$BUYER" 1000000000000000000000     # 1000
send "$BUYER" "$TOKEN_A" "approve(address,uint256)" "$GUARD" 115792089237316195423570985008687907853269984665640564039457584007913129639935
AMOUNT=$(cast call "$REPLICA" "balanceOf(address)(uint256)" "$MM" --rpc-url "$RPC" 2>/dev/null || echo 0)
if [ "$AMOUNT" = "0" ]; then
  send "$DEPLOY_KEY" "$REPLICA" "mint(address,uint256)" "$MM" 1000000000000000000000 || \
    send "$REGISTRAR" "$REPLICA" "transfer(address,uint256)" "$MM" 100000000000000000000
fi
send "$MM" "$REPLICA" "approve(address,uint256)" "$GUARD" 115792089237316195423570985008687907853269984665640564039457584007913129639935

# 4. red-flag states in spec-replay order. Guard order is record → paused →
#    blocklist → impl-match, so: the mismatch must land BEFORE the pause
#    (once paused, GUARD_PAUSED always preempts), and the buyer blocklist —
#    on the REPLICA's own beacon — comes last (it would poison every swap).
log "4. red-flag states"
if [ -n "$TWIN1" ]; then
  true # GUARD_NO_RECORD: twins stay unregistered — nothing to seed
fi
if [ -n "$BEACON" ] && [ -n "$IMPL_V2" ]; then
  CUR_IMPL=$(cast call "$BEACON" "implementation()(address)" --rpc-url "$RPC")
  [ "$CUR_IMPL" != "$IMPL_V2" ] && send "$DEPLOY_KEY" "$BEACON" "upgradeTo(address)" "$IMPL_V2" # tokenA record → stale → GUARD_IMPL_MISMATCH
fi
if [ -n "$TOKEN_A" ]; then
  PAUSED=$(cast call "$TOKEN_A" "paused()(bool)" --rpc-url "$RPC" 2>/dev/null || echo false)
  [ "$PAUSED" = "false" ] && send "$DEPLOY_KEY" "$TOKEN_A" "pause()" # GUARD_PAUSED (checked before impl)
fi
BLOCKED=$(cast call "$REPLICA_BEACON" "isBlocked(address)(bool)" "$BUYER" --rpc-url "$RPC" 2>/dev/null || echo false)
[ "$BLOCKED" = "false" ] && send "$DEPLOY_KEY" "$REPLICA_BEACON" "setBlocked(address,bool)" "$BUYER" true

# 5. the gated spec's data file (gitignored)
log "5. write $ROOT/e2e/.scratch-state.json"
jq -n --arg chainId "$CHAIN" --arg registry "$REGISTRY" --arg guard "$GUARD" --arg buyer "$BUYER" \
  --arg tokenA "$TOKEN_A" --arg tokenB "$TOKEN_B" --arg twin1 "$TWIN1" --arg replica "$REPLICA" \
  '{chainId: ($chainId | tonumber), registry: $registry, guard: $guard, buyer: $buyer,
    entries: [
      {kind: "settle",   tokenIn: $tokenA, tokenOut: $replica},
      {kind: "revert",   reason: "GUARD_NO_RECORD",      tokenOut: $twin1},
      {kind: "revert",   reason: "GUARD_RECORD_REVOKED", tokenOut: $tokenB},
      {kind: "revert",   reason: "GUARD_IMPL_MISMATCH",  tokenOut: $tokenA},
      {kind: "revert",   reason: "GUARD_PAUSED",         tokenOut: $tokenA},
      {kind: "revert",   reason: "GUARD_BLOCKLISTED",    tokenOut: $replica}
    ]}' > "$ROOT/e2e/.scratch-state.json"
cat "$ROOT/e2e/.scratch-state.json"

log "seeded. Fill the worker's scratch var next: REGISTRY_ADDRESS_$CHAIN=$REGISTRY (e2e/README.md step 3)"
