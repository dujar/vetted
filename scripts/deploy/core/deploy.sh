#!/usr/bin/env bash
# Step-3 core integration deploy (plan task 5) + source verification (task 6).
#
# Deploys the Canonical Registry + Guarded Swap (Stylus crates) and the mock
# pattern tokens (foundry) to ONE chain, runs the five red-flag receipts +
# degraded + clean settle ON-CHAIN, asserts the <= 200k gas gate from the
# broadcast receipts, and writes deployments/<chain>.json (append-only merge).
#
#   DEPLOY_KEY=0x… ./scripts/deploy/core/deploy.sh --chain 421614
#   DEPLOY_KEY=0x… MM_KEY=0x… ETHERSCAN_API_KEY=0x… \
#     ./scripts/deploy/core/deploy.sh --chain 421614 --verify
#   ./scripts/deploy/core/deploy.sh --chain 421614 --skip-receipts   # deploys only
#   ./scripts/deploy/core/deploy.sh --chain 421614 --receipts-only   # re-run receipts
#
# DEPLOY_KEY: the throwaway operator key (spike/.env — never committed). It is
#   the registry registrar, the beacon/token admin and the buyer. Funding is
#   the one manual step (spike/DEPLOY.md runbook) — an unfunded run cannot
#   fabricate receipts.
# MM_KEY: the project MM counterparty key — defaults to DEPLOY_KEY (self-fill
#   receipts; the two-party settle accounting is asserted only with a distinct
#   MM key).
set -euo pipefail

usage() {
  grep -E '^#   ' "$0" | head -6
  exit 1
}

CHAIN=""
VERIFY=0
SKIP_RECEIPTS=0
RECEIPTS_ONLY=0
while [ $# -gt 0 ]; do
  case "$1" in
    --chain) CHAIN="${2:?--chain needs a value}"; shift 2 ;;
    --verify) VERIFY=1; shift ;;
    --skip-receipts) SKIP_RECEIPTS=1; shift ;;
    --receipts-only) RECEIPTS_ONLY=1; shift ;;
    *) usage ;;
  esac
done
[ -n "$CHAIN" ] || usage
case "$CHAIN" in
  # live-probed RPCs (packages/shared/wire.md chains table — do not re-probe)
  421614) RPC="https://sepolia-rollup.arbitrum.io/rpc" ;;
  46630) RPC="https://rpc.testnet.chain.robinhood.com" ;;
  4663) RPC="https://rpc.mainnet.chain.robinhood.com" ;;
  *) echo "unknown chain '$CHAIN' (4663 | 46630 | 421614)" >&2; exit 1 ;;
esac

: "${DEPLOY_KEY:?set DEPLOY_KEY (spike/.env, never commit it)}"
MM_KEY="${MM_KEY:-$DEPLOY_KEY}"
DEPLOYER="$(cast wallet address --private-key "$DEPLOY_KEY")"
MM_ADDR="$(cast wallet address --private-key "$MM_KEY")"
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
MOCK="$ROOT/contracts/core/mock-token"
: "${CARGO_STYLUS_EXTRA:-}"   # e.g. --no-verify for a local (non-docker) build

log() { printf '\n=== %s ===\n' "$*"; }

# The last 0x…40 printed by a stylus deploy is the deployed contract; each
# deploy is then VALIDATED with a cast call against its own getter, which is
# the real check (a wrong parse aborts the script there).
last_addr() { grep -Eo '0x[0-9a-fA-F]{40}' | tail -1; }

if [ "$RECEIPTS_ONLY" -eq 1 ]; then
  : "${REGISTRY_ADDRESS:?set REGISTRY_ADDRESS}" "${GUARD_ADDRESS:?set GUARD_ADDRESS}"
  : "${BEACON_ADDRESS:?set BEACON_ADDRESS}" "${TOKEN_A:?set TOKEN_A}" "${TOKEN_B:?set TOKEN_B}"
  : "${TOKEN_PLAIN:?set TOKEN_PLAIN}" "${IMPL_V1:?set IMPL_V1}" "${IMPL_V2:?set IMPL_V2}"
  REGISTRY="$REGISTRY_ADDRESS"
  GUARD="$GUARD_ADDRESS"
  BEACON="$BEACON_ADDRESS"
  TOKEN_A="$TOKEN_A"
  TOKEN_B="$TOKEN_B"
  TOKEN_PLAIN="$TOKEN_PLAIN"
  IMPL_V1="$IMPL_V1"
  IMPL_V2="$IMPL_V2"
else
  log "stage A: Canonical Registry (Stylus) on $CHAIN"
  cd "$ROOT/contracts/core/registry"
  REGISTRY="$(cargo stylus deploy --endpoint "$RPC" --private-key "$DEPLOY_KEY" \
    --constructor-args "$DEPLOYER" $CARGO_STYLUS_EXTRA 2>&1 | last_addr)"
  [ "$(cast call "$REGISTRY" 'registrar()(address)' --rpc-url "$RPC")" = "$DEPLOYER" ] \
    || { echo "registry deploy validation failed ($REGISTRY)" >&2; exit 1; }
  echo "REGISTRY=$REGISTRY"

  log "stage B: Guarded Swap (Stylus) — registry + MM counterparty"
  cd "$ROOT/contracts/core/guard"
  GUARD="$(cargo stylus deploy --endpoint "$RPC" --private-key "$DEPLOY_KEY" \
    --constructor-args "$REGISTRY" "$MM_ADDR" $CARGO_STYLUS_EXTRA 2>&1 | last_addr)"
  [ "$(cast call "$GUARD" 'registry()(address)' --rpc-url "$RPC")" = "$REGISTRY" ] \
    || { echo "guard deploy validation failed ($GUARD)" >&2; exit 1; }
  echo "GUARD=$GUARD"

  log "stage C: mock pattern tokens (foundry)"
  cd "$MOCK"
  PRIVATE_KEY="$DEPLOY_KEY" MM_KEY="$MM_KEY" \
    REGISTRY_ADDRESS="$REGISTRY" GUARD_ADDRESS="$GUARD" \
    forge script script/DeployMockTokens.s.sol:DeployMockTokens \
    --rpc-url "$RPC" --broadcast --slow 2>&1 | tee "$MOCK/broadcast/deploy-mocks.out"
  BEACON="$(grep -Eo 'MOCK_BEACON:? 0x[0-9a-fA-F]{40}' "$MOCK/broadcast/deploy-mocks.out" | grep -Eo '0x[0-9a-fA-F]{40}' | tail -1)"
  IMPL_V1="$(grep -Eo 'MOCK_IMPL_V1:? 0x[0-9a-fA-F]{40}' "$MOCK/broadcast/deploy-mocks.out" | grep -Eo '0x[0-9a-fA-F]{40}' | tail -1)"
  IMPL_V2="$(grep -Eo 'MOCK_IMPL_V2:? 0x[0-9a-fA-F]{40}' "$MOCK/broadcast/deploy-mocks.out" | grep -Eo '0x[0-9a-fA-F]{40}' | tail -1)"
  TOKEN_A="$(grep -Eo 'MOCK_TOKEN_A:? 0x[0-9a-fA-F]{40}' "$MOCK/broadcast/deploy-mocks.out" | grep -Eo '0x[0-9a-fA-F]{40}' | tail -1)"
  TOKEN_B="$(grep -Eo 'MOCK_TOKEN_B:? 0x[0-9a-fA-F]{40}' "$MOCK/broadcast/deploy-mocks.out" | grep -Eo '0x[0-9a-fA-F]{40}' | tail -1)"
  TOKEN_PLAIN="$(grep -Eo 'MOCK_TOKEN_PLAIN:? 0x[0-9a-fA-F]{40}' "$MOCK/broadcast/deploy-mocks.out" | grep -Eo '0x[0-9a-fA-F]{40}' | tail -1)"
  [ -n "$BEACON" ] && [ -n "$IMPL_V1" ] && [ -n "$IMPL_V2" ] && [ -n "$TOKEN_A" ] \
    && [ -n "$TOKEN_B" ] && [ -n "$TOKEN_PLAIN" ] \
    || { echo "mock token addresses not parsed from forge output" >&2; exit 1; }
  echo "BEACON=$BEACON IMPL_V1=$IMPL_V1 IMPL_V2=$IMPL_V2"
  echo "TOKEN_A=$TOKEN_A TOKEN_B=$TOKEN_B TOKEN_PLAIN=$TOKEN_PLAIN"
fi

RECEIPTS_JSON="{}"
if [ "$SKIP_RECEIPTS" -eq 0 ]; then
  log "stage D: five red-flag receipts + degraded + clean settle (on-chain)"
  cd "$MOCK"
  PRIVATE_KEY="$DEPLOY_KEY" MM_KEY="$MM_KEY" \
    REGISTRY_ADDRESS="$REGISTRY" GUARD_ADDRESS="$GUARD" BEACON_ADDRESS="$BEACON" \
    TOKEN_A="$TOKEN_A" TOKEN_B="$TOKEN_B" TOKEN_PLAIN="$TOKEN_PLAIN" \
    IMPL_V1="$IMPL_V1" IMPL_V2="$IMPL_V2" \
    forge script script/RunReceipts.s.sol:RunReceipts \
    --rpc-url "$RPC" --broadcast --slow 2>&1 | tee "$MOCK/broadcast/receipts.out"

  log "stage E: gas gate — every guard execute() receipt <= 200000 (spec.md:51)"
  RUNJSON="$MOCK/broadcast/RunReceipts.s.sol/$CHAIN/run-latest.json"
  [ -f "$RUNJSON" ] || { echo "missing broadcast artifact $RUNJSON" >&2; exit 1; }
  # The receipts script's guard executes, in broadcast order — 11 of them
  # (5 reverting at indexes 1/3/5/7/9, 6 settling). execute() = 0x61461954.
  # Receipt rows stay HEX ("0x1|0x1dc0a") — bash arithmetic converts below.
  mapfile -t ROWS < <(jq -r --arg guard "$GUARD" '
    . as $run
    | .transactions | to_entries[]
    | select(((.value.transaction.to // "") | ascii_downcase) == ($guard | ascii_downcase))
    | select(((.value.transaction.input // .value.transaction.data // "") | ascii_downcase)
      | startswith("0x61461954"))
    | (.value.hash) as $h
    | ($run.receipts | map(select(.transactionHash == $h)) | first)
    | "\(.status // "0x0")|\(.gasUsed)"' "$RUNJSON")
  [ "${#ROWS[@]}" -eq 11 ] \
    || { echo "expected 11 guard execute() receipts, got ${#ROWS[@]}" >&2; exit 1; }

  NAMES=(GUARD_NO_RECORD DEGRADED_SETTLE GUARD_RECORD_REVOKED REVOKED_CLEANUP
    GUARD_PAUSED PAUSED_CLEANUP GUARD_BLOCKLISTED BLOCKLISTED_CLEANUP
    GUARD_IMPL_MISMATCH MISMATCH_CLEANUP SETTLE)
  ROWS_TSV="$MOCK/broadcast/receipts-rows.tsv"
  : > "$ROWS_TSV"
  for IDX in "${!NAMES[@]}"; do
    STATUS="${ROWS[$IDX]%%|*}"
    GAS="${ROWS[$IDX]##*|}"
    case "$STATUS" in
      1 | 0x1 | true) S=1 ;;
      0 | 0x0 | false) S=0 ;;
      *) S="$STATUS" ;;
    esac
    REVERTED=false
    EXPECT=1
    case "$IDX" in
      1 | 3 | 5 | 7 | 9)
        REVERTED=true
        EXPECT=0
        ;;
    esac
    [ "$S" = "$EXPECT" ] \
      || { echo "RECEIPT MISMATCH: ${NAMES[$IDX]} status $STATUS (expected $([ "$EXPECT" = 0 ] && echo reverted || echo success))" >&2; exit 1; }
    GAS="$((GAS))"   # hex 0x… → decimal (bash arithmetic)
    [ "$GAS" -le 200000 ] \
      || { echo "GAS GATE FAILED: ${NAMES[$IDX]} used $GAS (<= 200000 required)" >&2; exit 1; }
    printf '%s|%s|%s\n' "${NAMES[$IDX]}" "$GAS" "$REVERTED" >> "$ROWS_TSV"
    printf '  %-20s %s gasUsed=%s\n' "${NAMES[$IDX]}" "$([ "$EXPECT" = 0 ] && echo REVERTED || echo ok)" "$GAS"
  done
  RECEIPTS_JSON="$(jq -Rn '
    [inputs | select(length > 0) | split("|")]
    | map({(.[0]): {gasUsed: (.[1] | tonumber), reverted: (.[2] == "true")}})
    | add' < "$ROWS_TSV")"
else
  log "stage D/E skipped (--skip-receipts)"
fi

log "stage F: deployments/$CHAIN.json (append-only merge)"
DEP="$ROOT/deployments/$CHAIN.json"
COMMIT="$(git -C "$ROOT" rev-parse --short HEAD)"
TODAY="$(date +%F)"
jq -n --arg chainId "$CHAIN" --arg registry "$REGISTRY" \
  --arg guard "$GUARD" --arg beacon "$BEACON" --arg implV1 "$IMPL_V1" --arg implV2 "$IMPL_V2" \
  --arg tokenA "$TOKEN_A" --arg tokenB "$TOKEN_B" --arg tokenPlain "$TOKEN_PLAIN" \
  --arg deployedAt "$TODAY" --arg commit "$COMMIT" --argjson receipts "$RECEIPTS_JSON" \
  '{chainId: ($chainId | tonumber), status: "deployed", deployedAt: $deployedAt, commit: $commit,
    registry: $registry, guard: $guard,
    mockTokens: {beacon: $beacon, implV1: $implV1, implV2: $implV2,
                 tokenA: $tokenA, tokenB: $tokenB, tokenPlain: $tokenPlain},
    receipts: $receipts,
    gasGate: (if ($receipts | length) == 0
              then "pending (no receipts run)"
              else "all 11 guard execute() receipts <= 200000 (spec.md:51)" end),
    check: "5 revert receipts + degraded + settle, byte-exact strings asserted via expectRevert (scripts/deploy/core/deploy.sh)"}' \
  > "$MOCK/broadcast/deployments-entry.json"
if [ -f "$DEP" ]; then
  # append-only discipline: never clobber another step's fields (e.g. step 6's
  # replicas block) — shallow-merge ours over what is there
  jq -s '.[0] * .[1]' "$DEP" "$MOCK/broadcast/deployments-entry.json" > "$DEP.tmp"
else
  cp "$MOCK/broadcast/deployments-entry.json" "$DEP.tmp"
fi
mv "$DEP.tmp" "$DEP"
echo "wrote $DEP"
jq . "$DEP"

if [ "$VERIFY" -eq 1 ]; then
  log "stage G: source verification (task 6)"
  cd "$ROOT/contracts/core/registry"
  cargo stylus verify --endpoint "$RPC" "$REGISTRY"
  cd "$ROOT/contracts/core/guard"
  cargo stylus verify --endpoint "$RPC" "$GUARD"
  cd "$MOCK"
  for PAIR in "$BEACON:src/MockBeacon.sol:MockBeacon" "$IMPL_V1:src/MockTokenImpl.sol:MockTokenImpl"; do
    ADDR="${PAIR%%:**}"
    SRC="${PAIR#*:}"
    if [ "$CHAIN" = "421614" ]; then
      : "${ETHERSCAN_API_KEY:?ETHERSCAN_API_KEY required to verify on 421614 (Etherscan API v2)}"
      forge verify-contract "$ADDR" "$SRC" --chain-id 421614
    elif [ "$CHAIN" = "46630" ]; then
      forge verify-contract "$ADDR" "$SRC" --verifier blockscout \
        --verifier-url "https://explorer.testnet.chain.robinhood.com/api"
    else
      : "${BLOCKSCOUT_URL:?set BLOCKSCOUT_URL for 4663 (mainnet explorer API; knowledge/contract-verification.md)}"
      forge verify-contract "$ADDR" "$SRC" --verifier blockscout --verifier-url "$BLOCKSCOUT_URL"
    fi
  done
  echo "verified: registry + guard (cargo stylus verify), MockBeacon + MockTokenImpl (forge)"
else
  log "stage G skipped (--verify not set)"
fi

log "done — $CHAIN"
