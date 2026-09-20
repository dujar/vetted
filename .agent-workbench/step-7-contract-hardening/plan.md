# Step 7 — Contract hardening + quality evidence (the judged criterion)

## Resources
- spec:    ../product/spec.md   (judging weights spec.md:69; criteria-published requirement spec.md:35; ≤200k budget spec.md:51; risks spec.md:73–80)
- knows:   ../knowledge/stylus-toolchain.md (verify + reproducible builds :33–36), ../knowledge/robinhood-chain.md (96 KB limits :38), ../knowledge/contract-verification.md (explorer recipes + the 4663 Cloudflare-challenge footgun :19–27 — applies to any verify path, not just forge)
- learned: ../step-3-contracts-core/findings.md  (build notes, gas actuals, integration harness)
- learned: ../step-4-scan-backend/findings.md  (worker LIVE @ 0ccf8c68 — REGISTRAR_KEY/DRIFT_ADMIN_SECRET deliberately unset pending this step's handoff; drift-check degrades clean until set)
- learned: ../step-6-replica-assets/findings.md  (replica cast + deploy runbook in demo/assets.md; step 6 CREATES deployments/4663.json with the replicas field — this step appends; foundry deps gotcha)
- learned: ../step-1-repo-scaffold/findings.md  (scaffold MERGED at 5e2ff35 — deployments writer pins, live-probed 4663 RPC)
- learned: ../../spike/findings.md  (step-2 findings — MERGED at 9d8dc30; file lives in the REPO at `spike/findings.md`: Stylus ACTIVATION proven on 4663, ≤200k gas receipt still pending on funding (`spike/DEPLOY.md`); calibrated probe targets)

> **Revised** — step-1 findings reconciliation (2026-09-11): this step is the pinned writer of `deployments/4663.json` per `deployments/README.md` (append-only — never rewrite step 3's scratch entries), and the 4663 RPC endpoint is already live-probed and pinned in `packages/shared/wire.md` chains table + `frontend/src/lib/chains.ts` — use it for the deploy `--endpoint`; no re-probing.

> **Revised** — step-2 + step-5 findings reconciliation (2026-09-19):
> 1. **The 4663 deploy outputs feed the frontend env contract**: `VITE_REGISTRY_ADDRESS`/`VITE_GUARD_ADDRESS` are set from `deployments/4663.json`'s addresses (Pages env at steps 8/10; the frontend's live mode errors already point there — frontend `src/lib/guard.ts`/`wallet.ts`/`registry.ts`). Record both addresses in findings explicitly so the env handoff is one lookup.
> 2. **Gas report comparison base** (task 2): step 2's spike receipt may still not exist (PASS-PENDING-GAS — blocked on testnet funding, operator runbook `spike/DEPLOY.md`; activation itself IS proven, live `cargo stylus check` exit 0 on 4663). Step 3's integration receipts (421614 + 46630) are the committed live actuals to diff against; if the spike receipt lands via DEPLOY.md in the meantime, fold it in. Include the guard's beacon staticcall in the measured path — the blocklist probe targets the resolved beacon, not the token (calibrated, `packages/shared/abi.ts:49-63`).

> **Revised** — step-3 + step-4 + step-6 findings reconciliation (2026-09-20):
> 1. **First-funded-run routing — no funded on-chain run exists yet anywhere.** Step 3's receipts are deferred (operator key 0x151e…8E4C balance 0 on 421614/46630/4663 — re-verified 2026-09-20; funding per `spike/DEPLOY.md`; RunReceipts has never executed end-to-end — no usable stylus devnode locally) and step 6's replica deploys are funding-gated the same way. Per step 3's round-2 review: the FIRST funded run must go through `scripts/deploy/core/deploy.sh` END-TO-END (forge receipt stages + broadcast `receipts` array) so broadcast-JSON shape drift surfaces BEFORE steps 7/8 consume `deployments/`; those receipts are step 3's committed gas actuals. If step 3's funded run still hasn't happened at step-7 start, this step's run is the first — same rule. Script facts: `--chain 4663` accepted; `--receipts-only` re-runs receipts against a prior deploy (script friction is recoverable without redeploying); `MM_KEY` distinct from `DEPLOY_KEY` = two-party settle accounting (one-key fallback = self-fill, logged).
> 2. **`deployments/4663.json` writer correction:** the 2026-09-11 note said this step is the pinned writer — `deployments/README.md` (updated at step 6's merge) now pins writers as **6 (replicas field, creates the file) then 7 (core append)**. Keep the append-only shallow merge; never rewrite step 6's replica entries either.
> 3. **Task 5 handoff is TWO secrets:** step 4's live worker (`vetted-scan-backend.dujar-coding.workers.dev`, version 0ccf8c68) ships with `REGISTRAR_KEY` and `DRIFT_ADMIN_SECRET` both DELIBERATELY UNSET (live-verified 2026-09-20: drift-check degrades with a clean skipped report). Generate/set both — the registrar key per the task below, plus `DRIFT_ADMIN_SECRET` for the admin-gated `POST /registry/drift-check` step 9's upgrade→revocation beat calls.
> 4. **Verify expectations + the receipts' extra value:** stage G source-verifies only MockBeacon + MockTokenImpl (hull forwarders are raw-runtime deploys, unverifiable by construction); the receipts assert what the native tests cannot — probe→target wiring and zero movement on all five revert paths (step-3 round-1 findings 2+3 deferrals) — cite them in `gas-report.md`.
> 5. **Left-broken items this step owns (step-3 findings):** the 8 `cargo build` warnings in the registry/guard lib targets (task 4's quality pass) and the `ponytail:` marker at `contracts/core/guard/src/lib.rs:587` (line verified; step-3 round-1 finding 4 said :588). Foundry gotcha if this step runs the mock-token suite: `forge install` no-ops on a project-local `.gitmodules` inside a git repo — deps are pinned plain files restored by pinned clones (pattern: `contracts/replicas/README.md`; the mock-token CI job mirrors it).

## Stack
Same as step 3 (or the Solidity branch of it).

## Goal
After this step, the contract surface is defensible to a judge in five minutes: fuzz/invariant-tested, gas-budgeted with a committed report, documented, trust-modeled, deployed on 4663 and source-verified — the "smart contract quality" criterion answered with artifacts, not prose.

## Scope
- Touches: `contracts/core/**` (tests + docs only — no behavior change without a coordinator flag), `docs/` (`criteria.md`, `threats.md`), gas report artifact, the 4663 mainnet deploy of core + verification.
- Out of scope: any feature change; backend; frontend; replicas.

## User journey
N/A.

## Screens
N/A.

## Tasks
1. Fuzz + invariant tests (proptest/arbitrary over the native host): registrar authority is the only write path; record transitions are total and monotone per the rules; guard — every revert path leaves zero state change and zero balance delta; check order is deterministic (an earlier reason is never masked by a later one); no path exceeds the probe gas budget.
2. Gas report: re-measure full guard execution + probes (incl. the beacon blocklist probe — Revised note 2) on 421614 and 4663 receipts; commit `gas-report.md` with the ≤200,000 assertion (spec.md:51) and drift vs the step-3 integration actuals explained (the step-2 spike receipt if the DEPLOY.md operator run has landed by then; Revised note 2). The step-3 actuals are deploy.sh's committed receipt records and did NOT exist as of 2026-09-20 (operator key unfunded) — the first funded run executes them end-to-end before this diff is written (Revised 2026-09-20 note 1).
3. Docs: rustdoc complete on public surfaces; `docs/threats.md` — trust model (registrar key compromise blast radius, why single-registrar is acceptable for v1, revocation-as-safety framing); `docs/criteria.md` — the published verification criteria (spec.md:35 requirement; the registry page's criteria panel links exactly this path).
4. Fresh-eyes review pass: reentrancy posture (external calls are staticcalls during checks; effects ordering), 96 KB runtime / 192 KB init headroom (knowledge robinhood-chain.md:38), overflow posture, error-taxonomy completeness vs the five guard reasons.
5. 4663 mainnet deploy of registry + guard via the same `scripts/deploy/core/deploy.sh` end-to-end (`--chain 4663`; first-funded-run routing — Revised 2026-09-20 note 1; append-only into `deployments/4663.json` after step 6's replicas field — Revised note 2); **registrar-key handoff first:** generate the production registrar key, set it as the backend Worker's `REGISTRAR_KEY` secret and `DRIFT_ADMIN_SECRET` alongside it (both ship deliberately unset — Revised note 3), and deploy with registrar = that address (or `registrar-transfer` to it immediately after deploy) — step 4's drift-revokes sign with `REGISTRAR_KEY`, and a mismatch makes every revoke revert as a non-registrar write, killing the demo's upgrade→revocation beat (scratch/integration deploys keep throwaway keys). `cargo stylus verify` on 4663 and the 421614 mirror (Blockscout source-verified; explorer behaviors and fallbacks — mainnet `/api` may sit behind a Cloudflare bot challenge server-side, browser-UI verification passes — in knowledge/contract-verification.md:19–27); append `deployments/4663.json`.
6. `findings.md`: what hardened, what was found, what was consciously accepted.

Check: fuzz suites green in CI (seeded ≥10k cases each); `gas-report.md` committed with receipt links; verified contract pages live on the 4663 explorer; `docs/criteria.md` resolves at the path step 5's criteria panel links.

## Open questions
- None blocking. If fuzzing surfaces a design change rather than a fix (e.g. check reordering), stop and flag the coordinator before touching shared semantics.
