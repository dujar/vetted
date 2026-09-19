# Reconciliation 2026-09-19 — step-2 (spike gate) + step-5 (frontend screens) findings

Findings processed (both MERGED, both unreconciled):

- step-2 spike gate — **the findings file is NOT at the workbench path** `.agent-workbench/step-2-spike-gate/findings.md` (does not exist); the merged findings live in the REPO at `spike/findings.md` (step-2 scope put them there). Processed from there. GATE: PASS-PENDING-GAS (Stylus activation proven live on 4663 + 421614; ≤200k gas receipt pending testnet funding — operator runbook `spike/DEPLOY.md`).
- step-5 frontend screens — `.agent-workbench/step-5-frontend-screens/findings.md` (merged e37bc15).

Ground truth verified before editing: `packages/shared/abi.ts:49-63` PROBE_SELECTORS calibrated with the beacon-vs-proxy comment; frontend env hooks live in `frontend/src/lib/api.ts` / `guard.ts` / `wallet.ts` / `registry.ts` / `vite-env.d.ts`; canonical-fetch worker live-verified via curl on 2026-09-19 (`/assets` → ok, total 194, P/CRM @ 4663; `/health` → ok); `deployments/` still holds only pages/worker/README.

## Plans changed (all seven not-yet-built steps)

- **step-3-contracts-core/plan.md** — GATE decided: Stylus stands, the Solidity FAIL branch marked DEAD (kept for record). Task 3 probe targets corrected: `paused()` → token proxy, `isBlocked(buyer)` → resolved BEACON (state lives on the beacon, not the proxy — live-verified); impl resolution via beacon slot + `implementation()` (from-contract EIP-1967 impl-slot reads are impossible — no EXTSLOAD; genuine tokens have impl slot empty; port `spike/probe/` helpers). Task 4/5 mock tokens must mirror the genuine layout so the beacon-probe path is tested. Task 5's ≤200k re-assertion uses this step's own receipts — the pending spike receipt is parallel, NOT a start blocker; no gas numbers may be cited from step 2. Added learned: step-5 findings (VITE_* env contract, GUARD_REVERT_REASONS verbatim).
- **step-4-scan-backend/plan.md** — Task 1 probe targets pinned (same beacon/proxy split). Task 3 canonical ground truth: the spike canonical-fetch worker is LIVE — absorb the proven `spike/fetch/` logic into the scan worker; spike URL stays up as fallback mirror only (no hard dependency). Task 2 fingerprint: signature source = `spike/evidence/calibration_4663.json`, check list composed JOINTLY with step 6 (proxy-codehash byte-equality would fail an OZ-BeaconProxy replica — decide explicitly). Frontend seam pinned: `GET /scan?chainId=&addr=`, `GET /watchdog?chainId=` and `WatchdogStats {chainId, runs, baselinePerDay}` (frontend-side declaration flagged for this step to adopt into shared types). REVOKED `revocationTx` sourced from step 3's `Revoke` event. Power-report blocklist evidence = beacon probe.
- **step-6-replica-assets/plan.md** — Replica must mirror the GENUINE layout (blocklist mapping ON the beacon, transfer hook calls the beacon; pause on the token) — the spike's throwaway replica is the documented anti-pattern. Task 3 beacon must answer `isBlocked` alongside `implementation()`/`upgradeTo`; task 4 fidelity adds the calibrated probe-target assertions and composes the check list with step 4. Task 7 corrected: step 2 proved activation, not gas — "gas verified in step 2" reworded; don't cite figures; Foundry deploys aren't the gated suite.
- **step-7-contract-hardening/plan.md** — 4663 deploy outputs feed `VITE_REGISTRY_ADDRESS`/`VITE_GUARD_ADDRESS` (record both addresses in findings for the env handoff). Task 2 gas report: comparison base is step-3's integration receipts while the spike receipt is pending (fold in the DEPLOY.md receipt if it lands); measured path includes the beacon blocklist probe.
- **step-8-e2e-journeys/plan.md** — Env contract recorded: specs run against mock (default, CI-capable) or live (`VITE_API_MODE=live` + `VITE_API_URL` + the two VITE_* addresses from scratch deployments) — no page edits either way. Known defect from step 5 flagged: `ViemGuardProbeSource.isBlocked` probes the token (`frontend/src/lib/guard.ts:186-188`) instead of the resolved beacon — degrades to "probe unavailable" (safe) but blocklist preview rows are never real on genuine-pattern tokens; e2e must assert blocklist evidence from the `/scan` payload and file the frontend fix to the step-5 owner, not green the wrong target.
- **step-9-demo-harness/plan.md** — Power-report beat narrated from the `/scan` verdict payload (beacon-probe evidence; preview row degrades); live-scan beat backed by the live canonical-fetch worker absorbed into step 4.
- **step-10-deploy-submission/plan.md** — Env audit list made concrete (`VITE_API_MODE`, `VITE_API_URL`, `VITE_REGISTRY_ADDRESS`, `VITE_GUARD_ADDRESS`, `VITE_WALLETCONNECT_PROJECT_ID`); spike canonical-fetch worker is a fallback mirror — production smoke confirms step 4's absorbed fetch path.

Broken-path fixes: every `learned: ../step-2-spike-gate/findings.md` reference (steps 3, 4, 6 — written before the merge moved the file into the repo) repointed to `../../spike/findings.md` with a note that the file lives in the repo.

## No plan changes needed beyond the above

Step 9/10 scope already covered everything else in the findings (watchdog copy, GoPlus/EAS Q&A live in step 10 task 4).

## Escalations for the coordinator

1. **Stylus branch decision is settled** — no Solidity rewrite anywhere; steps 3/4/6 may build on Stylus (activation proven), gas budget asserted from each step's own receipts.
2. **Outstanding operator actions (parallel, non-blocking for steps 3/4/6):** (a) the ≤200k spike receipt — fund the spike key per `spike/DEPLOY.md` §0; (b) WalletConnect projectId (pre-existing, step 10).
3. **Step-5 defect** (guard.ts blocklist probe target) needs a fix owner — step 8 is instructed to route it back to the step-5 owner; a one-line-ish fix probing the beacon (pattern already in `resolvedImpl`).

## Judge

No spawn tool in this context — plan-judge-agent was dispatched by the caller (round logged as `reconcile-2026-09-19`).

**Verdict (round 3, 2026-09-19, plan-judgment.md:79): SHIPPABLE.** Zero gaps introduced by fee34b0; all six checks passed (beacon-not-proxy consistent across the seven plans, worker absorb coherent, env contract closed, gate decision non-blocking, 3/4/6 batch collision-free, routed defect durably recorded). Non-gap notes: step 7's Stack line still mentions the dead Solidity branch (cosmetic residue, no action required per the judge).

**Stamps applied on this verdict:** `reconciled: 2026-09-19` added to `.agent-workbench/step-5-frontend-screens/findings.md` (workbench commit) and to `spike/findings.md` (REPO-file commit — coordinator accepted this record as authoritative for that stamp; the file lives in the repo because step-2's scope put it there).
