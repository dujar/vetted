#!/usr/bin/env bash
# demo/arc.sh — the machine-checked beat driver (step-9 plan tasks 2+3).
#
#   bash demo/arc.sh                      # live 4663 beats (funded regime)
#   bash demo/arc.sh --anvil-rehearsal    # local upgrade-beat rehearsal (unfunded)
#
# Live mode drives the runbook's beats that a script CAN drive — scans with
# spaced retries on the designed RPC_RETRYABLE terminal (plan Revised
# 2026-09-20 note 1: retries are part of the harness; never narrate a
# failure), the upgrade beat end-to-end (beacon upgradeTo → manual
# POST /registry/drift-check → REVOKED scan → guard refusal), the watchdog
# read, and per-beat wall-clock for the runbook §5 table.
#
# The anvil rehearsal (verify loose end 4): the UNFUNDED wall-clock source.
# Deploys the step-6 replica cast locally (the step-6 rehearsal pattern) and
# times the upgrade leg's on-chain transactions. The registry/guard legs are
# Stylus (need ArbWasm — no plain-anvil deploy) and the drift-check needs the
# live worker: their actuals measure at task 6 on the funded chain.
#
# Env:
#   OPERATOR_KEY         funded demo operator (replica admin + MM; required live)
#   BUYER_KEY            default OPERATOR_KEY — signs commit/execute in beat 6
#   DRIFT_ADMIN_SECRET   required for beat 6's manual drift-check
#   BASE_URL             default https://vetted-scan-backend.dujar-coding.workers.dev
#   LIVE_SCAN_ADDR       beat 2 address, default the spike-calibrated Everpure token
#   RETRY_ATTEMPTS/RETRY_BACKOFF_S   scan retry policy (default 6 × 10s)
#   SKIP_UPGRADE=1       run beats 2-5 + 7 only (no chain-state change)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEP="$ROOT/deployments/4663.json"
RPC="https://rpc.mainnet.chain.robinhood.com"
BASE_URL="${BASE_URL:-https://vetted-scan-backend.dujar-coding.workers.dev}"
LIVE_SCAN_ADDR="${LIVE_SCAN_ADDR:-0x1Cdad396DB64BDa184d5182A97Dd9B3C62100b7D}"  # Everpure • Robinhood Token
RETRY_ATTEMPTS="${RETRY_ATTEMPTS:-6}"
RETRY_BACKOFF_S="${RETRY_BACKOFF_S:-10}"
BUDGETS=(30 30 30 45 45 60 30)   # runbook §5, beats 1-7 (beat 1 is human-only)

command -v cast >/dev/null || { echo "cast is required" >&2; exit 1; }
command -v jq >/dev/null || { echo "jq is required" >&2; exit 1; }
command -v curl >/dev/null || { echo "curl is required" >&2; exit 1; }

now() { date +%s.%N; }
elapsed() { awk -v a="$1" -v b="$2" 'BEGIN{printf "%.1f", b-a}'; }
BEAT_T=0
beat() { # beat N NAME — resets the timer, prints the header
  BEAT_N="$1"; BEAT_NAME="$2"; BEAT_T=$(now)
  printf '\n=== beat %s — %s (budget %ss)\n' "$BEAT_N" "$BEAT_NAME" "${BUDGETS[$((BEAT_N-1))]}"
}
beat_done() {
  printf '--- beat %s wall-clock: %ss (budget %ss)\n' "$BEAT_N" "$(elapsed "$BEAT_T" "$(now)")" "${BUDGETS[$((BEAT_N-1))]}"
}
TOTAL_T=$(now)

scan_retry() { # scan_retry ADDR — GET /scan with spaced retries; prints the
               # final JSON; sets SCAN_JSON. Honors RPC_RETRYABLE + degraded.
  local addr="$1" attempt=1 json verdict
  while [ "$attempt" -le "$RETRY_ATTEMPTS" ]; do
    if json=$(curl -fsS "$BASE_URL/scan?chainId=4663&addr=$addr"); then
      verdict=$(printf '%s' "$json" | jq -r '.verdict // .terminalState // "?"')
      case "$verdict" in
        RPC_RETRYABLE)
          printf '  attempt %s/%s: RPC_RETRYABLE (designed transient) — waiting %ss\n' "$attempt" "$RETRY_ATTEMPTS" "$RETRY_BACKOFF_S" ;;
        *)
          SCAN_JSON="$json"; printf '  verdict/terminal: %s (attempt %s)\n' "$verdict" "$attempt"; return 0 ;;
      esac
    else
      printf '  attempt %s/%s: scan request failed — waiting %ss\n' "$attempt" "$RETRY_ATTEMPTS" "$RETRY_BACKOFF_S"
    fi
    attempt=$((attempt+1)); [ "$attempt" -le "$RETRY_ATTEMPTS" ] && sleep "$RETRY_BACKOFF_S"
  done
  echo "scan did not settle in $RETRY_ATTEMPTS attempts — report honestly, never fabricate (runbook beat 2 fallback)" >&2
  return 1
}

anvil_rehearsal() { # the UNFUNDED wall-clock source (verify loose end 4)
  command -v anvil >/dev/null || { echo "anvil is required (foundry)" >&2; exit 1; }
  command -v forge >/dev/null || { echo "forge is required (foundry)" >&2; exit 1; }
  local REPL_LIB="$ROOT/contracts/replicas/lib"
  if [ ! -d "$REPL_LIB/forge-std" ]; then
    echo "bootstrapping replicas deps (contracts/replicas/README.md clones)…"
    ( cd "$ROOT/contracts/replicas" \
      && git clone -q --depth 1 --branch v1.16.2 https://github.com/foundry-rs/forge-std lib/forge-std \
      && git clone -q --depth 1 --branch v5.4.0 https://github.com/OpenZeppelin/openzeppelin-contracts-upgradeable lib/openzeppelin-contracts-upgradeable \
      && git clone -q --depth 1 --branch v5.4.0 https://github.com/OpenZeppelin/openzeppelin-contracts lib/openzeppelin-contracts-upgradeable/lib/openzeppelin-contracts )
  fi
  echo "starting anvil…"
  anvil --port 8545 >/tmp/arc-anvil.log 2>&1 &
  ANVIL_PID=$!
  trap 'kill "$ANVIL_PID" 2>/dev/null || true' EXIT
  for i in $(seq 1 20); do
    cast chain-id --rpc-url http://127.0.0.1:8545 >/dev/null 2>&1 && break
    sleep 0.5
    [ "$i" = 20 ] && { echo "anvil did not come up (see /tmp/arc-anvil.log)" >&2; exit 1; }
  done
  local ANVIL_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
  # anvil's well-known key is legitimate HERE: this chain exists for seconds
  # and holds nothing (the seed-demo default dropped it for 4663 — LE 3).
  local OUT=/tmp/arc-rehearsal-deploy.log
  ( cd "$ROOT/contracts/replicas" \
    && DEPLOY_KEY="$ANVIL_KEY" forge script script/DeployReplicas.s.sol:DeployReplicas \
       --rpc-url http://127.0.0.1:8545 --broadcast --slow 2>&1 | tee "$OUT" )
  local BEACON IMPL_V2
  BEACON=$(grep -Eo 'REPLICA_BEACON:? 0x[0-9a-fA-F]{40}' "$OUT" | grep -Eo '0x[0-9a-fA-F]{40}' | tail -1)
  IMPL_V2=$(grep -Eo 'REPLICA_IMPL_V2_UPGRADE_TARGET:? 0x[0-9a-fA-F]{40}' "$OUT" | grep -Eo '0x[0-9a-fA-F]{40}' | tail -1)
  [ -n "$BEACON" ] && [ -n "$IMPL_V2" ] || { echo "replica addresses not parsed from forge output" >&2; exit 1; }

  printf '\n=== upgrade-beat rehearsal (on-chain leg only — drift-check/guard legs measure at task 6)\n'
  local T0 T1 RC
  T0=$(now)
  cast send "$BEACON" "upgradeTo(address)" "$IMPL_V2" --private-key "$ANVIL_KEY" --rpc-url http://127.0.0.1:8545 >/dev/null
  T1=$(now)
  RC=$(cast call "$BEACON" "implementation()(address)" --rpc-url http://127.0.0.1:8545)
  [ "$RC" = "$IMPL_V2" ] || { echo "upgrade did not take (impl=$RC)" >&2; exit 1; }
  printf 'upgradeTo broadcast→receipt: %ss; implementation() now %s\n' "$(elapsed "$T0" "$T1")" "$RC"
  printf '\nanvil-rehearsal actual for runbook §5 (upgrade leg): %ss — live beat-6 total = this + drift-check + refusal legs\n' "$(elapsed "$T0" "$T1")"
}

LIVE=0
[ "${1:-}" = "--anvil-rehearsal" ] && { anvil_rehearsal; exit 0; }
LIVE=1

# ---- live preflight -------------------------------------------------------
[ -f "$DEP" ] || { echo "missing $DEP — the funded-run runbook (step-7 findings) has not executed (task-6 gate)" >&2; exit 1; }
[ "$(jq -r '.status' "$DEP")" = "deployed" ] || { echo "$DEP is not a funded deploy" >&2; exit 1; }
REGISTRY=$(jq -r '.registry // empty' "$DEP");  GUARD=$(jq -r '.guard // empty' "$DEP")
REPLICA=$(jq -r '.replicas.proxy // empty' "$DEP");  BEACON=$(jq -r '.replicas.beacon // empty' "$DEP")
IMPL_V2=$(jq -r '.replicas.implV2 // empty' "$DEP"); TWIN1=$(jq -r '.replicas.twin1 // empty' "$DEP")
TOKEN_A=$(jq -r '.mockTokens.tokenA // empty' "$DEP")
[ -n "$REGISTRY" ] && [ -n "$GUARD" ] && [ -n "$REPLICA" ] && [ -n "$BEACON" ] && [ -n "$IMPL_V2" ] \
  || { echo "$DEP lacks core+replicas fields — runbook go-list steps 2-4 first" >&2; exit 1; }
HEALTH=$(curl -fsS "$BASE_URL/health" | jq -r '.ok')
[ "$HEALTH" = "true" ] || { echo "worker /health not ok — beat fallbacks apply (runbook §6)" >&2; exit 1; }
: "${OPERATOR_KEY:?set OPERATOR_KEY (funded demo operator)}"
BUYER_KEY="${BUYER_KEY:-$OPERATOR_KEY}"
BUYER=$(cast wallet address --private-key "$BUYER_KEY")
MINT=1000000000000000000000

echo "arc: registry $REGISTRY · guard $GUARD · replica $REPLICA · buyer $BUYER"
echo "(beat 1 — the docs-page problem — is human-only; start the clock when you open the app)"

# ---- beat 2: live scan, not our deployment --------------------------------
beat 2 "live scan (issuer token, not ours)"
scan_retry "$LIVE_SCAN_ADDR"
printf '%s\n' "$SCAN_JSON" | jq -r '"  verdict: \(.verdict)  rows: \(.powerReport | length)  degraded: \(.degraded)"'
printf '%s\n' "$SCAN_JSON" | jq -e 'if .verdict == "VERIFIED" then (.powerReport | length) > 0 else true end' >/dev/null \
  || { echo "HONESTY: VERIFIED without report rows — blocking, not narratable" >&2; exit 1; }
beat_done

# ---- beat 3: impostor twin vs canonical -----------------------------------
beat 3 "twin compare (expected: UNVERIFIED, never IMPOSTOR — verify LE 2)"
scan_retry "$TWIN1"
TWIN_VERDICT=$(printf '%s' "$SCAN_JSON" | jq -r '.verdict')
[ "$TWIN_VERDICT" != "IMPOSTOR" ] || { echo "HONESTY: twin rendered IMPOSTOR on the live engine — discipline breach (spec.md:27), blocking" >&2; exit 1; }
echo "  twin verdict: $TWIN_VERDICT (docs-page impostor definition is beat-3 narration; refusal is GUARD_NO_RECORD)"
beat_done

# ---- beat 4: power report on the replica ----------------------------------
beat 4 "power report (replica)"
scan_retry "$REPLICA"
printf '%s\n' "$SCAN_JSON" | jq -r '"  verdict: \(.verdict)"'
printf '%s\n' "$SCAN_JSON" | jq -r '.powerReport[] | "  \(.check): \(.result) [\(.severity)]"'
printf '%s\n' "$SCAN_JSON" | jq -r '.powerReport[] | select(.check == "Buyer blocklist in modifier") | .result' \
  | grep -q PRESENT || echo "  NOTE: blocklist row not PRESENT — setBlocked the probe buyer (runbook beat 4 pre-stage) or narrate honestly"
beat_done

# ---- beat 5: settle plumbing read-check (the swaps themselves are human) --
beat 5 "settle plumbing read-check"
ALLOW_A=$(cast call "$TOKEN_A" "allowance(address,address)(uint256)" "$BUYER" "$GUARD" --rpc-url "$RPC")
ALLOW_R=$(cast call "$REPLICA" "allowance(address,address)(uint256)" "$(cast wallet address --private-key "$OPERATOR_KEY")" "$GUARD" --rpc-url "$RPC")
echo "  buyer tokenA→guard allowance: $ALLOW_A · MM replica→guard: $ALLOW_R"
if [ "$ALLOW_A" = "0" ] || [ "$ALLOW_R" = "0" ]; then
  echo "  plumbing missing — re-run scripts/seed-demo/seed.sh"
else
  echo "  plumbing holds; refusal leg (5a) + settle leg (5b) run from the frontend"
fi
beat_done

# ---- beat 6: upgrade → auto-revoke → refusal ------------------------------
if [ "${SKIP_UPGRADE:-0}" = "1" ]; then
  echo "SKIP_UPGRADE=1 — beat 6 not run (no chain-state change)"
else
  beat 6 "upgrade → drift-check → guard refusal"
  T_UP0=$(now)
  cast send "$BEACON" "upgradeTo(address)" "$IMPL_V2" --private-key "$OPERATOR_KEY" --rpc-url "$RPC" >/dev/null
  T_UP1=$(now)
  echo "  upgradeTo broadcast→receipt: $(elapsed "$T_UP0" "$T_UP1")s"
  LIVE_IMPL=$(cast call "$BEACON" "implementation()(address)" --rpc-url "$RPC")
  [ "$LIVE_IMPL" = "$IMPL_V2" ] || { echo "upgrade did not take" >&2; exit 1; }

  : "${DRIFT_ADMIN_SECRET:?set DRIFT_ADMIN_SECRET for the manual drift-check (worker secret, runbook step 6b)}"
  T_DC0=$(now); DC=""; DC_OK=0
  for i in 1 2 3; do
    if DC=$(curl -fsS -X POST "$BASE_URL/registry/drift-check" -H "X-Admin-Secret: $DRIFT_ADMIN_SECRET"); then
      SKIPPED=$(printf '%s' "$DC" | jq -r '.skipped // empty')
      REVOKED_N=$(printf '%s' "$DC" | jq -r --arg r "$REPLICA" \
        '[.revoked[]? | select((.token | ascii_downcase) == ($r | ascii_downcase))] | length')
      if [ "$SKIPPED" = "" ] && [ "$REVOKED_N" != "0" ]; then DC_OK=1; break; fi
      echo "  drift-check attempt $i: skipped=${SKIPPED:-none} replica-revocations=$REVOKED_N"
    else
      echo "  drift-check attempt $i failed (worker/RPC) — the guard still refuses in-tx; retry after the beat"
    fi
    sleep 5
  done
  T_DC1=$(now)
  if [ "$DC_OK" = "1" ]; then
    printf '%s\n' "$DC" | jq -r '.revoked[] | "  auto-revoked \(.token) → tx \(.tx_hash)"'
    echo "  drift-check leg: $(elapsed "$T_DC0" "$T_DC1")s (narration: same walk the */15 cron runs)"
  else
    echo "  drift-check did not confirm the revocation — fallback narration: the guard's own in-tx impl probe (GUARD_IMPL_MISMATCH) covers the poll window; retry off-camera"
  fi

  scan_retry "$REPLICA"
  printf '%s\n' "$SCAN_JSON" | jq -r '"  replica verdict now: \(.verdict)  revocationTx: \(.revocationTx // "not indexed")"'

  # the refusal, on-chain: commit then execute → expect GUARD_RECORD_REVOKED
  T_RF0=$(now)
  cast send "$GUARD" "commit(address,address,uint256,uint256)" "$TOKEN_A" "$REPLICA" "$MINT" "$MINT" \
    --private-key "$BUYER_KEY" --rpc-url "$RPC" >/dev/null
  RF=$(cast send "$GUARD" "execute()" --private-key "$BUYER_KEY" --rpc-url "$RPC" 2>&1 || true)
  T_RF1=$(now)
  if printf '%s' "$RF" | grep -q "GUARD_RECORD_REVOKED"; then
    echo "  guard refused on-chain: GUARD_RECORD_REVOKED (execute leg $(elapsed "$T_RF0" "$T_RF1")s)"
  elif printf '%s' "$RF" | grep -q "GUARD_IMPL_MISMATCH"; then
    echo "  guard refused on-chain: GUARD_IMPL_MISMATCH (drift-check had not landed yet — the in-tx probe)"
  else
    echo "  UNEXPECTED guard outcome — inspect manually; do not narrate a scripted refusal:" >&2
    printf '%s\n' "$RF" | tail -5 >&2
    exit 1
  fi
  beat_done
fi

# ---- beat 7: watchdog (close) ---------------------------------------------
beat 7 "watchdog + close"
WD=$(curl -fsS "$BASE_URL/watchdog?chainId=4663")
printf '%s\n' "$WD" | jq -r '"  runs: \(.runs)  baselinePerDay: \(.baselinePerDay)  provenance: \(.provenanceUrl)"'
printf '%s\n' "$WD" | jq -e 'if .runs == 0 then .provenanceUrl != null else true end' >/dev/null \
  || { echo "HONESTY: runs==0 without provenance — wire.md sentinel violated" >&2; exit 1; }
echo "  narration: measured L1 figures (258,707 batchCount / ~2,630 per day, 2026-09-20) with provenance; widget = baseline/illustrative"
beat_done

printf '\n=== arc total: %ss (budget 300s; check: ≤5:00 twice in a row — human legs incl.)\n' "$(elapsed "$TOTAL_T" "$(now)")"
echo "paste the per-beat wall-clocks into demo/runbook.md §5 (task 6)"
