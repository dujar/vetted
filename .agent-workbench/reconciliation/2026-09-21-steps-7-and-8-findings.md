# Reconciliation 2026-09-21 — step-7 (contract-hardening) + step-8 (e2e-journeys) findings

Findings processed (both MERGED, both unreconciled at round start; stamps WITHHELD
pending the judge round):

- step-7 contract-hardening — `.agent-workbench/step-7-contract-hardening/findings.md`
  (merged 93201ab; review APPROVED round 1, 0 blocking). Note: this file has NO
  `reconciled:` line at all (not even an empty one) — treated as unreconciled per the
  standing rule.
- step-8 e2e-journeys — `.agent-workbench/step-8-e2e-journeys/findings.md`
  (merged b17fdd3; review APPROVED round 1, 0 blocking, 5 dispositions; `reconciled:`
  present but empty).

Not-yet-built steps at round start: **9 (demo-harness) and 10 (deploy-submission)
only** — both plans amended; steps 1–8 are `done` and their plans are untouched.

## Ground truth verified before editing (all on main 6cd317a, clean)

- `e2e/scripts/seed-scratch.sh` → `bash -n` parse-clean (the funding-gated parse error
  is fixed; the script has still never been executed — no funded scratch run yet).
- Scratch activation runbook: `e2e/README.md:41` ("Scratch-chain activation (currently
  gated — operator key unfunded)").
- J3 empty-registry unit pin: `frontend/tests/registry.test.tsx:76` (loading/degraded/
  empty describe block; the empty test exists, no e2e hook — as step-8 finding 3 says).
- Dead guard test helpers: `mock_record` at `contracts/core/guard/src/lib.rs:706`,
  `install_beacon_token` at :733 (pre-existing on main, dispositioned follow-up).
- Frontend wiring facts from step 8 in the tree: `VETTED_CHAIN` export at
  `frontend/src/lib/chains.ts:56`; the `injected()` e2e seam at
  `frontend/src/lib/wagmi.ts:33` behind `VITE_E2E_STUB_WALLET`.
- Worker vars: ALL FOUR are empty `[vars]` entries at `scan-backend/wrangler.toml:13-21`
  (`REGISTRY_ADDRESS_4663/46630/421614`, `DRIFT_EXTRA_TOKENS`); `DRIFT_EXTRA_TOKENS` is
  read by `scan-backend/src/drift.rs:7` — the `--var` re-pass gotcha and step-9
  ownership are real.
- `deployments/` holds only `pages.json`, `worker.json`, `README.md` — no chain JSON;
  `deployments/421614.json` and `4663.json` are both PENDING the funded-run runbook.
- `docs/{criteria,threats,gas-report}.md` exist; `docs/gas-report.md:3,:74` still
  PASS-PENDING-FUNDS (flips on the runbook's last step).
- Stamped findings: steps 1, 3, 4, 5, 6 (`reconciled:` dated); step-2's stamp lives in
  the repo's `spike/findings.md` (2026-09-19 precedent). Steps 7/8 are the only
  outstanding ones — matches this round.

## Plans changed (the two not-yet-built steps)

- **step-9-demo-harness/plan.md** — learned line added for step-7 + step-8 findings;
  new Revised block (2026-09-21, 5 notes); inline amendments to tasks 1, 3, 6:
  1. The **funded-run runbook is a prerequisite, folded in whole** (7 steps: fund
     operator key → end-to-end deploy.sh 421614 writing `421614.json` as first-funded
     writer-of-record → registrar key + funding → 4663 deploy + transfer_registrar →
     `4663.json` creation → verify with CF fallbacks → worker handoff → gas actuals).
     Task 6's dry run stays the first full 4663 journey pass — it runs AFTER the
     runbook, not instead of it; the runbook itself is no step-9 build work.
  2. Demo seeding/revokes sign with the **NEW post-handoff registrar key** (post-
     transfer, the deploy key reverts on registrar-only writes); task 3's manual
     drift-check beat needs the runbook's `DRIFT_ADMIN_SECRET`.
  3. Step 9 owns `DRIFT_EXTRA_TOKENS` and must re-pass ALL known non-empty registry
     vars on the same `wrangler deploy --var` command line (blanking gotcha); the
     runbook's "step 8 re-passes" text is stale (step 8 merged before the runbook and
     left vars empty) — first real pass is the runbook's, then step 9's.
  4. `scripts/seed-demo/` **inherits `e2e/scripts/seed-scratch.sh`'s swap-settle
     plumbing** (buyer tokenIn + MM tokenOut, balance+allowance both sides — needed by
     the genuine-settles beat, absent from task 1's original text); seed-scratch.sh is
     parse-fixed but never executed, so first real execution happens here; the 3
     activation-gated e2e skips activate per `e2e/README.md:41` from funded
     `deployments/*.json` — run the live e2e project once post-runbook as the cheapest
     journey-wide regression check.
  5. Narration honesty: revocation-tx evidence comes from `/scan` (registry UI
     drill-in keeps `revocationTx: null`); the J3 empty-registry state is unit-pinned
     only (`frontend/tests/registry.test.tsx:76`, coordinator-carried) — the demo
     always shows a seeded registry, so narration must not claim the empty state was
     journey-verified. Pages bundle (2026-09-21 redeploy) already carries VETTED_CHAIN
     + the render-loop fix.
- **step-10-deploy-submission/plan.md** — learned line added for step-8 findings; new
  Revised block (2026-09-21, 4 notes); inline pointers in tasks 1 and 2:
  1. Writer-of-record correction: the runbook writes BOTH `421614.json` (step 3's
     funded run never happened) and `4663.json` (append-only over step 6's replicas
     field); audit also confirms gas-report §4's PASS-PENDING-FUNDS flip; single-chain
     funding → the 421614-mirror check is honestly N/A.
  2. Vars audit extends the two-secret audit: `REGISTRY_ADDRESS_4663` set via `--var`
     (never a wrangler.toml edit), scratch pair only if the scratch activation ran;
     audit what IS set, not that all three are set.
  3. Verification was executed by the runbook with CF fallbacks — task 2 re-checks
     pages resolve, does not re-run verification.
  4. Dead guard-test helpers (`BEACON`, `mock_record` :706, `install_beacon_token`
     :733) print dead-code warnings — dispositioned follow-up; do not block the tag on
     them and do not "clean up" contracts at freeze.

## No plan changes needed beyond the above

- The **paid/private-RPC decision** and **submission-package targeting** were already
  pinned in step 10 by the 2026-09-20 round (Revised notes there; tasks 2 and 5) — the
  dispatcher's "already pinned" reading is correct; nothing new to fold.
- Step 7's judge-grade docs (criteria/threats/gas-report) already have consumers: step
  9 task 5 links `docs/criteria.md`; step 10 task 2 checks gas-report status.
- Step 8's frontend wiring fixes (VETTED_CHAIN, lazy singletons, VITE_REGISTRY_TOKENS)
  are already in the live Pages bundle — no step 9/10 action beyond narration honesty
  (folded as step-9 note 5).

## Escalations for the coordinator

1. **Funding remains the sole on-chain blocker** and now gates step 9's start: the
   runbook is complete and needs no build work (fund the operator key per
   `spike/DEPLOY.md` §0 + the 4663 bridge; any builder can execute). Step 9's plan now
   names the runbook as its prerequisite — step 9 dispatches after funding + this
   round's judge.
2. **`guard` `erc20()` bool-decode accepts junk returndata** (step-7 out-of-scope item,
   flagged for a coordinator decision): tightening is a behavior change; NO remaining
   step owns `contracts/**` (9 = demo, 10 = deploy/submission). Decide before
   submission: a coordinator-sanctioned contracts change (needs plan-agent — the step
   set has no owner for it) or carry to judges' Q&A (step-7 findings document the
   worst case: phantom active order for a lying token's buyer, execute reverts whole,
   funds safe).
3. **J3 empty-registry e2e coverage** — coordinator-carried residue (forcing it needs
   an out-of-scope frontend hook; step-5-owner follow-up if ever ordered). Recorded in
   step 9's plan (narration honesty); no remaining step can act on the hook itself.
4. **Dead guard-test helpers** — dispositioned follow-up cleanup; recorded in step
   10's freeze note so a freeze-time builder neither blocks on the warnings nor
   "fixes" contracts at tag time.
5. Process carry: the runbook text "step 8 re-passes 4663 alongside its scratch pair"
   is stale (step 8 merged first, vars still empty) — corrected in step 9's plan; do
   not follow it when executing the runbook.

## Judge

No spawn tool in this context — plan-judge-agent must be dispatched by the caller
(round to be logged as `reconcile-2026-09-21`). `reconciled:` stamps on the two
findings files are WITHHELD until the verdict; this round ends at the plan edits +
this record.
