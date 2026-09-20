# demo/runbook.md — the 5-minute arc, start to finish on 4663

The arc (spec.md Scope 6, mapping journeys.md): **docs-page problem (0:30) →
live scan of a real address we did NOT deploy (0:30) → impostor twin compare
(0:30) → power report on the genuine replica (0:45) → guarded swap (0:45) →
beacon-upgrade → registry auto-revoke → guard refusal (1:00) → watchdog +
registry-as-primitive close (0:30)** — 4:30 of content, 0:30 of buffer. The
arc completes when it runs twice in a row ≤5:00 from this runbook (plan task
6 check).

Companion pieces: `demo/arc.sh` (the machine-checked beat driver: scan
retries + the upgrade beat + wall-clock), `demo/serve.sh` (the demo serving
mode), `demo/video-script.md` (1:1 narration), `demo/assets.md` (the cast +
state-setting choreography this runbook sequences).

## 0. Regimes and the task-6 gate

**This runbook's dry run (plan task 6) is gated**: it is the FIRST full
journey pass on live 4663 and it runs only after the funded-run runbook
(`.agent-workbench/step-7-contract-hardening/findings.md`) has executed.
Unfunded, everything in `demo/` and `scripts/seed-demo/` is built and
rehearsed (`arc.sh --anvil-rehearsal` covers the upgrade beat's on-chain leg
locally — Stylus contracts themselves need ArbWasm, so registry/guard legs
rehearse only against a funded chain or via `DRY_RUN=1`).

**Go-list for the funded window** (execute top to bottom; nothing else is
missing):

1. Fund the operator key (spike/.env) — 421614 faucet, then bridge to 4663
   via the Arbitrum portal (~10 min). Registrar key: `cast wallet new`, fund
   it on 4663 too (0.005 ETH is generous) — runbook §1–2 of step-7 findings.
2. `DEPLOY_KEY=$SPIKE_DEPLOY_KEY RPC_URL=https://rpc.mainnet.chain.robinhood.com ./scripts/deploy/replicas/deploy.sh`
   → append the logged addresses to `deployments/4663.json` under `replicas`
   (create the file; writer: step 6/9 per `deployments/README.md`) AND to
   `demo/assets.md`.
3. `DEPLOY_KEY=$SPIKE_DEPLOY_KEY ./scripts/deploy/core/deploy.sh --chain 4663`
   → registry + guard + mock tokens + the 11 receipts; stage F shallow-merges
   `deployments/4663.json` append-only over the `replicas` field.
4. Registrar handoff: `cast send $REGISTRY "transfer_registrar(address)" $NEW_REGISTRAR --private-key $SPIKE_DEPLOY_KEY --rpc-url https://rpc.mainnet.chain.robinhood.com`,
   validate with `cast call $REGISTRY 'registrar()(address)'`. From here the
   demo's verify/revoke sign with the NEW key only (Revised 2026-09-21 note 2).
5. `cargo stylus verify` for registry + guard on 4663 (Cloudflare-challenge
   fallbacks: Blockscout browser UI / `--verifier sourcify`).
6. Worker handoff: `wrangler secret put REGISTRAR_KEY` (the new key),
   `wrangler secret put DRIFT_ADMIN_SECRET` (generate one), then ONE reconfig
   that re-passes ALL currently-known non-empty vars on the command line —
   never edit wrangler.toml + redeploy from a stale checkout (it silently
   blanks the live vars; Revised note 3):
   ```bash
   cd scan-backend && npx wrangler deploy \
     --var REGISTRY_ADDRESS_4663:0x… \
     --var DRIFT_EXTRA_TOKENS:0x<replica>,0x<tokenB> \
     $( [ -n "$SCRATCH_PAIR" ] && echo --var REGISTRY_ADDRESS_46630:0x… --var REGISTRY_ADDRESS_421614:0x… )
   ```
   `DRIFT_EXTRA_TOKENS` is step-9-owned (runbook 6d): the replica + tokenB
   ride in the drift walk even if Verify-log enumeration hits a rate limit.
   If the e2e scratch activation (step 8) has run, re-pass its pair too.
7. `REGISTRAR_KEY=… OPERATOR_KEY=… bash scripts/seed-demo/seed.sh` (the
   standing state — it is idempotent, re-run freely between takes).
8. Cheapest journey-wide regression before recording: run the e2e live
   project once (`e2e/README.md` activation) — the three honest skips flip
   green, proving worker vars + seeded records are visible end to end.
9. `bash demo/serve.sh` (below) and the pre-demo checklist (§2).

## 1. The demo serving mode

`demo/serve.sh` bakes the live env into a local serve of the real bundle
(`deployments/4663.json` → `VITE_API_MODE=live`, `VITE_API_URL`,
`VITE_CHAIN_ID=4663`, `VITE_REGISTRY_ADDRESS`, `VITE_GUARD_ADDRESS`,
`VITE_REGISTRY_TOKENS`). Requires `VITE_WALLETCONNECT_PROJECT_ID` in the
environment or the swap beat cannot connect (warned at startup). The
production Pages bundle is the product-default MOCK build — do not demo live
beats from it.

## 2. Pre-demo checklist (untimed)

- [ ] Fresh browser profile; the funded **operator wallet imported** (it is
      the buyer on camera); Robinhood Chain 4663 added (RPC
      `https://rpc.mainnet.chain.robinhood.com`, chain id 4663, ETH).
- [ ] `bash demo/serve.sh` with `VITE_WALLETCONNECT_PROJECT_ID` set; load the
      scan page once before going live.
- [ ] Console pane ready with: `REGISTRY/GUARD/BEACON/REPLICA/IMPL_V2/TWIN1/TOKEN_A/TOKEN_B`
      exported from `deployments/4663.json`, plus `OPERATOR_KEY`,
      `DRIFT_ADMIN_SECRET` (never on camera, never committed).
- [ ] OBS: 1920×1080, 60fps, mic check; scene = browser + console split (the
      console is part of the story — the upgrade beat is driven there).
- [ ] **Product name locked** (the open question — recording waits on it;
      the frontend wordmark is `frontend/src/lib/brand.ts`, one file).
- [ ] Optional calm-window probe: `BASE_URL=https://vetted-scan-backend.dujar-coding.workers.dev ./scan-backend/scripts/live-check.sh`
      — a VERIFIED-shaped P receipt here doubles as beat 2's green light
      (spaced attempts; plan Revised 2026-09-20 note 2).

## 3. The beats

Console shorthand: `$RPC = https://rpc.mainnet.chain.robinhood.com`,
`$APP = http://localhost:4885`. Every beat's fallback names what to SAY —
the degraded branch is pre-written copy, never improvised, never fabricated.

### Beat 1 — the docs-page problem (budget 0:30)
- **Where:** docs.robinhood.com/chain/contracts (real page, no app).
- **Say:** "This is the issuer's entire machine answer: a static page. It
  says a token with a matching name and ticker at a different address is not
  a Robinhood Stock Token — for humans, stale on every upgrade, no feed."
- **Fallback:** page slow/unreachable → say the sentence from memory and
  move; the demo never bets on this fetch (spec risk 4).

### Beat 2 — live scan, an address we did NOT deploy (budget 0:30)
- **Where:** `$APP/#/?addr=0x1Cdad396DB64BDa184d5182A97Dd9B3C62100b7D`
  (Everpure • Robinhood Token — issuer-deployed, on 4663 since before this
  project existed; wallet stays disconnected — scans are anonymous and
  read-only).
- **Expected:** verdict **VERIFIED** (green), evidence rows incl. the issuer
  canonical-list match (on-chain `uid()` == issuer row id) and power-report
  rows, every line evidence-linked. Network selector shows Robinhood · 4663.
- **Fallback (in order):**
  - `RPC_RETRYABLE` terminal or a spinner past ~8s → say: "The chain's
    public RPC is rate-limiting us right now — the scanner is built to
    retry rather than guess. By design it never shows an unearned verdict."
    Re-run `demo/arc.sh` beat 2 (retries with backoff), then reload.
  - **Honest UNVERIFIED with advisory rows** (completed scan, probes dropped)
    → say: "Under rate limiting some probes didn't answer — you get an honest
    UNVERIFIED with the partial evidence, never a fabricated verdict."
  - Backup addresses (same beat, swap the deep link):
    `0xd95B44124e475743a7589e68F3D74008A5536D44` (Salesforce • Robinhood
    Token, spike-calibrated). Any fresh official redeployment reads UNVERIFIED
    until verified — that equals the docs page's own update lag (spec risk 5).

### Beat 3 — impostor twin vs canonical (budget 0:30)
- **Where:** `$APP/#/?addr=$TWIN1&addr=$REPLICA` (the compare deep link).
- **Expected (the honest shape, verify loose end 2):** replica **VERIFIED**;
  twin **UNVERIFIED** (amber) with the depth-boundary notice visible. The
  live engine NEVER renders IMPOSTOR-red for the twins: IMPOSTOR requires
  issuer-list mimicry + the genuine signature pattern, and the twins are demo
  artifacts not on the issuer's list — exactly what the e2e live spec pins.
- **Say:** "Same name, same ticker, different contract — the docs page's own
  impostor definition. The engine won't wear the impostor word on a guess:
  what it does is refuse at execution. This address has no verification
  record — watch the swap beat refuse it on-chain."
- **Fallback:** if a judge asks to SEE the IMPOSTOR verdict card, say it is a
  mock-fixture state and show it explicitly in mock mode
  (`MOCK_IMPOSTOR_ADDR`, `frontend/src/lib/mockData.ts`) — labeled as a
  fixture, never passed off as a live 4663 scan.
- **Red flag:** if the twin ever shows IMPOSTOR on the live engine, STOP —
  that is a false-positive discipline breach (spec.md:27); it is a blocking
  finding, not a narration problem.

### Beat 4 — power report on the genuine replica (budget 0:45)
- **Pre-staged (before going live, console):**
  `cast send $BEACON "setBlocked(address,bool)" 0xd8da6bf26964af9d7eed9e03e53415d37aa96045 true --private-key $OPERATOR_KEY --rpc-url $RPC`
  — the engine's fixed probe buyer (`scan-backend/src/lib.rs:28`); blocking
  it is what makes the scan's blocklist row read PRESENT.
- **Where:** `$APP/#/?addr=$REPLICA`.
- **Expected:** **VERIFIED** with red power rows — "Buyer blocklist in
  modifier: PRESENT" (evidence links the BEACON read — the state is invisible
  on the token), "Upgradeability: BEACON_PROXY", "Global pause control:
  ABSENT" (it is off — the mechanism is what the row discloses).
- **Say:** "Verified is not safe. This token — verified against the issuer's
  list — carries a per-address blocklist hidden in a beacon modifier and a
  global pause switch. One call flips either. Every row links its evidence."
- **Optional live flip (only if ≥20s ahead of budget):** pause it on camera —
  `cast send $BEACON "pause()" --private-key $OPERATOR_KEY --rpc-url $RPC`,
  rescan → pause row PRESENT — then **UNPAUSE and verify
  `cast call $PROXY "paused()(bool)"` reads false before beat 5** (a paused
  token turns the genuine settle into GUARD_PAUSED). If running long, skip
  the flip; narrate from the blocklist row alone.
- **Fallback:** probe row reads "probe unavailable" (genuine-pattern rate
  limit) → narrate from the power-report rows that DID land; never from the
  registry preview row (plan Revised 2026-09-20 note 1 — the preview degrades
  on genuine-pattern tokens).

### Beat 5 — guarded swap: refusal + settle (budget 0:45)
- **Where:** `$APP/swap`. Wallet: the operator wallet connected (buyer).
- **5a refusal:** tokenIn = `$TOKEN_A` ("Mock Stock A / MSA" — say plainly:
  "a plain stand-in token, the cash leg"), tokenOut = `$TWIN1`, 100 MSA.
  Execute → on-chain revert, reason surfaced verbatim: **GUARD_NO_RECORD**.
  Say: "No verification record → the swap contract refuses at execution.
  Funds never moved."
- **5b settle:** tokenOut = `$REPLICA`, same amounts. Execute → settles,
  guard receipt shown. Say: "Everything else settles — deterministic
  reverts only."
- **Fallback:** wallet/connect misbehaves → run both legs from the console
  with cast (`commit` + `execute` as buyer, MM plumbing already seeded) and
  narrate the receipt; the revert string surfaces in the receipt either way.
  Buyer gas empty → `scripts/seed-demo/seed.sh` re-run tops it up.

### Beat 6 — THE BEAT: upgrade → auto-revoke → refusal (budget 1:00)
- **Console, on camera:** `time cast send $BEACON "upgradeTo(address)" $IMPL_V2 --private-key $OPERATOR_KEY --rpc-url $RPC`
- **Say:** "Same token address, same proxy — the implementation just changed
  underneath it. The registry records the implementation pointer. Watch."
- **Trigger the revocation:** `curl -sS -X POST https://vetted-scan-backend.dujar-coding.workers.dev/registry/drift-check -H "X-Admin-Secret: $DRIFT_ADMIN_SECRET"`
  → the report names the replica in `revoked[]` with the revoke tx hash.
  **Say:** "This is the exact walk the backend's cron runs every 15 minutes;
  I triggered it by hand for the camera. On production nobody is here — the
  schedule is."
- **Show it:** `$APP/#/?addr=$REPLICA` → **REVOKED**, with the revocation tx
  linked (narrate from the /scan payload — the registry table's drill-in
  shows "not indexed yet"; that is a UI indexing gap, do not claim it live).
  Then `$APP/swap`, tokenOut = `$REPLICA` → **GUARD_RECORD_REVOKED** surfaced.
- **Say:** "Yesterday this swap settled. The token was upgraded since, the
  record revoked itself, and the guard refuses it at execution. No incumbent
  reports, none enforce — this is the difference."
- **Fallbacks:**
  - drift-check errors/retries (worker rate-limited): the guard STILL refuses
    in-tx via its own implementation probe → GUARD_IMPL_MISMATCH surfaces in
    the swap beat. Say: "The window between poll and revoke is exactly why
    the guard re-probes inside the transaction." Retry the drift-check after
    the beat; show the report when it lands.
  - upgrade tx rejected (nonce/gas): re-broadcast; the beat is one storage
    write, tenths of a cent.
- **Timing:** this beat's wall-clock is measured (unfunded: the anvil
  rehearsal covers the on-chain leg; live actuals land in the §5 table at
  task 6).

### Beat 7 — watchdog + registry-as-primitive close (budget 0:30)
- **Where:** back on the scan page (watchdog widget beside the verdict), then
  `$APP/registry`.
- **Watchdog narration (measured figures WITH provenance — never the stale
  6,092/~150 spec copy; widget shown as baseline/illustrative):** "The
  sequencer's own screen — 'any transaction associated with a sanctioned
  address will be excluded' — ran far more than the published six-week
  figures suggest: our L1 read of the SequencerInbox counter
  (0xBd0D173E…ba96) measured 258,707 batches with delivery-style events at
  roughly 2,630 per day (measured 2026-09-20; the widget on screen compares
  against the published ~150/day baseline until a reconciled counter ships).
  One recurring read, no stored history — a reason to come back."
- **Registry-as-primitive:** the table (replica now REVOKED-red with reason,
  tokenB's standing REVOKED record, twins absent), the criteria panel
  (registrar, `docs/criteria.md`), the on-chain read interface —
  "wallets and aggregators consume `getRecord` from their own contracts."
- **Close:** repo + `docs/criteria.md` + the honest scope line: "Signature
  matches on the Robinhood stock-token pattern get full verdicts; everything
  else gets a labeled UNVERIFIED plus structural heuristics. It never
  guesses, and it enforces where it verified."
- **Fallback:** watchdog widget shows the degrade sentinel (runs: 0 +
  provenance link) → that IS the honest state; narrate the provenance.

## 4. Narration honesty rules (bind at recording)

1. Depth boundary stated on camera (beat 7 close; beat 3 shows it).
2. Never script a fabricated verdict: `RPC_RETRYABLE` → retry + say so;
   honest UNVERIFIED → the partial-evidence framing. A "successful" demo
   take that edited around a degraded scan is a failed take.
3. Deferred-deploy items never implied live: the cron is real (*/15) and the
   demo says so when triggering it by hand; the watchdog widget is narrated
   as baseline/illustrative; the registry drill-in's revocation tx is not
   claimed.
4. Banned wording: "trust layer" (Blockaid's phrase), "first coverage"
   (GoPlus lists 4663). The line is: *first correct coverage-shaped answer
   for this chain's pattern — coverage without enforcement was already
   here; enforcement was not.* Twin beats never claim IMPOSTOR-red on the
   live engine (beat 3).
5. Demo-cast labeling: the replicas are demo artifacts — no liquidity, no
   holders, placeholder issuer, labeled in the repo (`demo/assets.md`) and
   on camera if asked (spec.md risk 6).

## 5. Timing budget (task 6 fills the actuals; the check is ≤5:00 twice in a row)

| beat | budget | actual (take 1) | actual (take 2) |
|---|---|---|---|
| 1 docs-page | 0:30 | | |
| 2 live scan | 0:30 | | |
| 3 twin compare | 0:30 | | |
| 4 power report | 0:45 | | |
| 5 guarded swap | 0:45 | | |
| 6 upgrade→revoke→refusal | 1:00 | | |
| 7 close | 0:30 | | |
| **total** | **4:30 (+0:30 buffer)** | | |

`demo/arc.sh` prints per-beat wall-clock to paste into this table.
Unfunded-regime actual (anvil rehearsal, 2026-09-21): the upgrade beat's
on-chain leg — `upgradeTo` broadcast→receipt + the implementation read — ran
**0.3s** on local anvil (`demo/arc.sh --anvil-rehearsal`); live beat-6 total
= this + the drift-check + refusal legs, measured at task 6 on 4663.

## 6. Failure appendix

- **Scan retry loop:** `bash demo/arc.sh` runs beats with spaced retries on
  the designed `RPC_RETRYABLE` terminal (default 6 attempts, 10s backoff).
  If throttling persists into the demo window: a paid/private RPC endpoint
  for the worker is the upgrade path — step 10's decision (the worker keeps
  rpc headroom: `scan-backend/src/rpc.rs`).
- **Canonical fetch down (degraded branch, pre-written):** all verdicts drop
  to UNVERIFIED with the on-screen degraded banner; say: "Ground truth is
  the issuer's live list; it's unreachable right now, so we show honest
  UNVERIFIED — the registry records remain on-chain." (spec.md:55 rules.)
- **Worker down:** beats 2/4/6 scan legs degrade; the swap beats still run
  (guard + registry are on-chain). Say so; do not fake scan results.
- **Recording:** OBS, full take + per-beat clips (beat boundaries = §5 rows);
  if a take dies mid-beat, restart from the previous beat boundary with the
  seeder re-run for state (`scripts/seed-demo/seed.sh` is idempotent — it
  re-verifies nothing that already holds; to reset the replica record after
  an accidental early upgrade, re-deploy the replica cast, re-seed, and
  re-point `deployments/4663.json` `replicas`).
