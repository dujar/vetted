# Step 9 — demo harness (the scripted 5-minute arc + submission master assets)

status:     merged (main 25a1bf3 via --no-ff; review round 2 APPROVED — both round-1 blockers fixed as prescribed, non-blocking notes taken; full suite green on merged main: contracts 30+2+14, scan-backend 40+5, shared 21+4+4, frontend 84 + build, e2e 28 passed + 3 activation-gated skips)
branch:     step-9-demo-harness (off main 8648efc; merged, pushed)
reconciled: 2026-09-21 (folded into plan 10 — commit aecd05a, Revised note 6; record: reconciliation/2026-09-21-step-9-findings.md; judge round 5 2026-09-21: SHIPPABLE, 0 gaps)
deployed:   not deployed — unfunded regime (operator key 0 wei on 4663/421614, re-checked 2026-09-21); tasks 1–5 built + rehearsed, task 6 staged (go-list below)

## Review round 1 (2026-09-21) — 2 blocking, both fixed; dispositions

1. arc.sh beat 6 printed the post-revocation scan but never asserted it (a
   stale VERIFIED/UNVERIFIED read would exit 0 with §5 wall-clocks pasted) —
   **FIXED**: the REVOKED assertion now gates the beat (same shape as beat
   2's honesty gates); a stale read fails the run with the re-take message.
2. README demo-cast paragraph copied step-6's stale "the tool's own rules
   flag the impostor twins" — contradicting the honest twin beat (LE 2) —
   **FIXED**: "The tool's depth boundary marks the impostor twins UNVERIFIED
   (never a guessed IMPOSTOR), and the guard refuses them on-chain at
   execution."
3. Non-blocking notes: `6,092` appears only inside the runbook's prohibition
   (compliant, kept); friendly worker-unreachable message added to arc.sh
   preflight (was a raw curl error); `.replicas` JSON shape example with the
   forge-log-key mapping added to runbook go-list step 2.
   `--anvil-rehearsal` re-run green post-fix (0.3s upgrade leg, 2026-09-21).

## What was built

The demo harness, funding-free per the two-regime dispatch (Revised 2026-09-21 note 1):

- `scripts/seed-demo/seed.sh` — idempotent 4663 demo-state seeding: verify(replica at the beacon's live impl), verify+revoke(mock tokenB → the standing REVOKED record), twins left unregistered (GUARD_NO_RECORD beat), two-sided settle plumbing (buyer tokenA / MM replica, balance+allowance) ported from `e2e/scripts/seed-scratch.sh`. Signed by the NEW post-handoff registrar key (Revised note 2); buyer defaults to the funded demo-operator wallet — the anvil well-known default is dropped (verify LE 3). Gated on `deployments/4663.json` (task-6 gate); `DRY_RUN=1` + `DEPLOYMENTS_FILE` rehearse the full plan (validated end-to-end against a /tmp rehearsal file).
- `demo/runbook.md` — the arc beat by beat (URL, wallet, expected screen, budget, pre-written fallbacks: RPC_RETRYABLE retry copy, honest-UNVERIFIED framing, backup live-scan addresses from spike calibration), the funded go-list, narration honesty rules, timing table (§5).
- `demo/serve.sh` — the demo serving mode (verify LE 1): bakes `deployments/4663.json` → VITE_* into a local live-mode serve of the real bundle; no e2e wallet stub; warns without `VITE_WALLETCONNECT_PROJECT_ID` (swap beat needs a connector).
- `demo/arc.sh` — machine-checked beat driver: scan retries with backoff on the designed `RPC_RETRYABLE` terminal (Revised 2026-09-20 note 1), honesty assertions (VERIFIED-without-rows and twin-IMPOSTOR are blocking; watchdog `runs:0`-without-provenance violates the wire sentinel), the upgrade beat end-to-end (upgradeTo → manual `POST /registry/drift-check` (`X-Admin-Secret`) → REVOKED scan → on-chain commit+execute refusing `GUARD_RECORD_REVOKED`, `GUARD_IMPL_MISMATCH` in-tx fallback), per-beat wall-clock for §5. `--anvil-rehearsal` = the unfunded wall-clock source (verify LE 4): **measured upgrade leg 0.3s** broadcast→receipt on local anvil (2026-09-21); Stylus registry/guard + worker legs measure at task 6.
- `demo/video-script.md` — 1:1 narration, word-budgeted to beat budgets, judged-criteria mapping, honesty pre-flight, Q&A ammo pointers.
- `README.md` final draft (plan task 5): architecture diagram, quickstart incl. e2e, `docs/criteria.md` pointer, demo section + demo-cast labeling (spec.md risk 6), stale facts corrected (CI is six jobs, not four). `demo.gif` link commented until the take exists — links must resolve.
- All four verify loose ends applied as prescribed: LE 1 → `serve.sh` + runbook §1; LE 2 → runbook beat 3 + video beat 3 (twin = UNVERIFIED + docs-page definition + GUARD_NO_RECORD; IMPOSTOR-red mock fixture only, labeled); LE 3 → seed-demo buyer default + no anvil key anywhere outside `--anvil-rehearsal` (where a seconds-old throwaway chain legitimizes it, commented); LE 4 → arc.sh rehearsal mode + measured actual recorded in runbook §5.

## Where the plan was wrong

- seed-scratch's idempotency idiom (`grep "^0, "` on the decoded record tuple) was never executed and is unreliable — cast's tuple formatting isn't pinned. seed-demo shape-checks raw `getRecord` hex instead (7×64 chars; word0 status, word4 registrar; zeroed tuple = no-record per wire.md).
- A REVOKED record cannot re-verify (registry transitions are monotone, step-7 fuzz): naive "re-run seed.sh between takes" would send a reverting verify. seed-demo exits with the recovery path; runbook §6 documents re-deploying the replica cast for a fresh proxy.
- The power report's pause row reads PRESENT only when `paused()` answers TRUE (rules.rs:213 — capability-off renders ABSENT). The assets.md choreography's live pause would poison beat 5's settle (GUARD_PAUSED). Runbook beat 4 makes the pause flip optional with a mandatory unpause checkpoint; the blocklist row carries the disclosure beat instead.
- The scan's blocklist probe buyer is the engine's FIXED `FRESH_BUYER` 0xd8da…6045 (lib.rs:28), not the demo wallet — the hidden-blocklist beat must block THAT address for the PRESENT row. (assets.md choreography's `$BUYER` is the guard-level variable; runbook pre-stage names both.)

## What the next step needs to know

- **Task-6 go-list (staged; execute in order once funding lands):**
  1. Execute the step-7 funded-run runbook (`.agent-workbench/step-7-contract-hardening/findings.md` §runbook, steps 1–7: fund → 421614 end-to-end deploy → registrar `cast wallet new` + fund → 4663 core deploy → `deployments/4663.json` → `cargo stylus verify` (+ Cloudflare fallbacks) → worker secrets + gas actuals).
  2. 4663 replicas deploy BEFORE the core deploy's stage F: `DEPLOY_KEY=… RPC_URL=https://rpc.mainnet.chain.robinhood.com ./scripts/deploy/replicas/deploy.sh` → append the logged addresses under `.replicas` in `deployments/4663.json` (create file; writer 6/9) + `demo/assets.md`.
  3. ONE worker reconfig re-passing ALL non-empty vars (Revised note 3 — never wrangler.toml + stale redeploy): `cd scan-backend && npx wrangler deploy --var REGISTRY_ADDRESS_4663:0x… --var DRIFT_EXTRA_TOKENS:0x<replica>,0x<tokenB>` (+ `--var REGISTRY_ADDRESS_46630:… --var REGISTRY_ADDRESS_421614:…` if the e2e scratch activation ran). Secrets ship via `wrangler secret put` (REGISTRAR_KEY = post-handoff key; DRIFT_ADMIN_SECRET).
  4. `REGISTRAR_KEY=<new key> OPERATOR_KEY=<funded operator> bash scripts/seed-demo/seed.sh` (idempotent; `BUYER_KEY` optional override).
  5. `npx playwright test --project=live` in `e2e/` once — the honest skips flip green = cheapest journey-wide regression.
  6. `VITE_WALLETCONNECT_PROJECT_ID=… bash demo/serve.sh`; fresh browser profile + imported operator wallet (runbook §2 checklist).
  7. Dry runs: `bash demo/arc.sh` (live), twice ≤5:00 from the runbook; paste per-beat wall-clocks into runbook §5; between takes `BASE_URL=https://vetted-scan-backend.dujar-coding.workers.dev ./scan-backend/scripts/live-check.sh` with spaced attempts to capture the still-missing VERIFIED-shaped P receipt (Revised 2026-09-20 note 2 — record it here when one lands). Any 4663-only divergence (RPC behavior, explorer links, verdict on the real cast) is a blocking fix BEFORE recording (plan task 6).
  8. Recording waits on the product name (open question — coordinator lock; `frontend/src/lib/brand.ts` is the one file). Step 10 uncomments the `demo.gif` img in README.
- **Buyer identity (LE 3 decision):** the 4663 buyer is the demo-operator wallet (funded by the runbook's bridge step, imported into the browser profile); MM counterparty = the same key's admin/mint authority, so the settle is self-fill accounting unless a second key is supplied — mirror of step-3's MM_KEY fallback, disclosed in the seed output.
- **Worker drift-check auth header is `X-Admin-Secret`** (lib.rs CORS + handler); the cron is `*/15` and the manual endpoint runs the same walk — that equivalence is the on-camera honesty line.
- `DRIFT_EXTRA_TOKENS` belt-and-braces: the drift walk already enumerates Verify logs; the extra var covers log-enumeration failure under rate limits.

## Out of scope, left broken

- Worktree incident (disclosed, no damage): the anvil rehearsal's cleanup `rm -rf contracts/replicas/broadcast/.../31337` caught step-6's tracked receipt `run-1789794873744.json`; restored immediately via `git checkout` (verified clean before commit). Rehearsal artifacts (`run-1789936399082.json`, cache) are removed/gitignored.
- `demo/demo.gif` does not exist until the final take (README link commented; uncomment at step 10).
- VERIFIED-shaped live P receipt still pending (rate-limit windows; task 6 between-takes, go-list step 7).
- Stale watchdog figures remain in the frontend mock (6092/150, `frontend/src/lib/mockData.ts:70`) by design until a reconciled counter exists — narration never quotes them.
- The e2e scratch worker vars + 3 gated live-scratch skips remain as step 8 left them (funding-gated; go-list steps 3/5 activate).
