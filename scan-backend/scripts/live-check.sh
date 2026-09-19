#!/usr/bin/env bash
# Live integration check (plan task 8) — read-only, against the DEPLOYED worker.
# Usage: BASE_URL=https://vetted-scan-backend.dujar-coding.workers.dev ./scripts/live-check.sh
# Exits non-zero on the first failed expectation.
set -euo pipefail

BASE_URL="${BASE_URL:-https://vetted-scan-backend.dujar-coding.workers.dev}"
# A genuine 4663 stock token (calibration_4663.json) and its uid/id.
P_ADDR="0x1Cdad396DB64BDa184d5182A97Dd9B3C62100b7D"
P_UID="0x0000000000000000000000000000000002c4d1ce31ec4310b2c507c921a52b70"
# An EOA — the NOT_CONTRACT terminal state.
EOA="0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"

say() { echo "[live-check] $*"; }
fail() { echo "[live-check] FAIL: $*" >&2; exit 1; }

command -v jq >/dev/null || fail "jq required"

say "health"
HEALTH=$(curl -fsS "$BASE_URL/health")
echo "$HEALTH" | jq -e '.ok == true' >/dev/null || fail "health not ok: $HEALTH"

say "watchdog shape (chainId 4663)"
WD=$(curl -fsS "$BASE_URL/watchdog?chainId=4663")
echo "$WD" | jq -e '.chainId == 4663 and (.baselinePerDay > 0)' >/dev/null \
  || fail "watchdog shape: $WD"
say "  watchdog: $(echo "$WD" | jq -c .)"

say "genuine 4663 token (P) — expect VERIFIED-shaped"
SCAN=$(curl -fsS "$BASE_URL/scan?chainId=4663&addr=$P_ADDR")
echo "$SCAN" | jq -e '.verdict == "VERIFIED"' >/dev/null \
  || fail "P expected VERIFIED, got: $(echo "$SCAN" | jq -c .)"
echo "$SCAN" | jq -e '.terminalState == null and .degraded == false' >/dev/null \
  || fail "P must not be terminal/degraded: $(echo "$SCAN" | jq -c .)"
# the strongest single check must surface on a genuine token: listed via the
# issuer canonical list (uid cross-check lives in the scan pipeline)
echo "$SCAN" | jq -r '.powerReport[].result' | grep -q "LISTED" \
  || fail "P missing canonical LISTED row: $(echo "$SCAN" | jq -c .)"
# hidden-power disclosure on the genuine pattern (beacon carries the blocklist)
echo "$SCAN" | jq -r '.powerReport[] | select(.check == "Buyer blocklist in modifier") | .result' \
  | grep -q "PRESENT" || fail "P blocklist row should disclose PRESENT"
say "  P verdict: $(echo "$SCAN" | jq -r .verdict), rows: $(echo "$SCAN" | jq -r '.powerReport | length')"

say "EOA — expect NOT_CONTRACT terminal state"
EOA_SCAN=$(curl -fsS "$BASE_URL/scan?chainId=4663&addr=$EOA")
echo "$EOA_SCAN" | jq -e '.verdict == null and .terminalState == "NOT_CONTRACT"' >/dev/null \
  || fail "EOA expected NOT_CONTRACT, got: $(echo "$EOA_SCAN" | jq -c .)"

say "non-4663 chain is read-only with the notice (chainId 421614)"
MIRROR=$(curl -fsS "$BASE_URL/scan?chainId=421614&addr=$P_ADDR")
echo "$MIRROR" | jq -e '.notice != null' >/dev/null \
  || fail "421614 scan must carry the notice: $(echo "$MIRROR" | jq -c .)"

say "uid cross-check: on-chain uid() == issuer registry id"
UID_ONCHAIN=$(cast call "$P_ADDR" 'uid()(bytes32)' --rpc-url https://rpc.mainnet.chain.robinhood.com 2>/dev/null \
  || python3 - <<'EOF'
import json, urllib.request
req = urllib.request.Request(
    "https://rpc.mainnet.chain.robinhood.com",
    data=json.dumps({"jsonrpc":"2.0","id":1,"method":"eth_call","params":[{"to":"0x1Cdad396DB64BDa184d5182A97Dd9B3C62100b7D","data":"0xf514ce36"},"latest"]}).encode(),
    headers={"Content-Type": "application/json"},
)
print(json.load(urllib.request.urlopen(req))["result"])
EOF
)
[ "$(echo "$UID_ONCHAIN" | tr 'A-F' 'a-f')" = "$P_UID" ] \
  || fail "uid mismatch: on-chain $UID_ONCHAIN vs issuer $P_UID"
say "  uid matches byte-exact"

say "spike canonical-fetch mirror still alive (fallback)"
curl -fsS "https://vetted-spike-canonical-fetch.dujar-coding.workers.dev/assets" \
  | jq -e '.ok == true' >/dev/null || say "  WARN: spike mirror down (non-fatal — production fetches the issuer directly)"

say "ALL LIVE CHECKS PASSED"
