# Plan judgment — round 1 (2026-09-12)

Set judged: steps 1–10 vs `.agent-workbench/product/spec.md` (approved 2026-09-12), `journeys.md`, `state.md` round-4/5 corrections, `knowledge/*`. All Resources-block paths resolve; cited knowledge line ranges spot-checked and accurate (stylus-toolchain.md:16–25,32–38; robinhood-chain.md:14–21; robinhood-stock-tokens.md:12–19; goplus-api.md:11–17; arbitrum-sepolia.md:11–13).

## What holds

- **Coverage — complete.** Every v1 scope item maps: scan → 4+5; registry → 3 (+4 task 6 drift cron = the spec's revocation trigger, no DB); guarded swap → 3+5; watchdog → 4+5; replica set → 6; demo arc → 9. All five spike bullets land in step 2 (canonical-fetch bullet superseded by `api.robinhood.com/rhj/assets` per knowledge robinhood-stock-tokens.md:12–19 — a refinement, consistent with state.md round 5, degraded branch retained). Verdict discipline → 4 task 3 + 5 depth-boundary notice; criteria doc → 7 (path linked by 5 task 4); wire contract → 1; pre-submission checklist → 10 task 4. All three screens built by 5; all three journeys e2e'd by 8. Nothing in the spec is unplanned.
- **Phases complete.** Phase 1 = 1 (scaffold, CI, test harness, deploy proven day one with recorded URLs) + 2 (exact spec'd pass/fail, `GATE: PASS|FAIL` on findings line 1). Phase 3 = 7 (hardening + gas report + docs + 4663 deploy) + 8 (e2e) + 9 + 10; observability answered at demo scale (wrangler tail + /health, step 4 task 7) — defensible for this window.
- **Order sound, gate branch respected.** No cycles, no forward refs, every declared edge is behaviorally real (3/4/6 genuinely need 2's fingerprint + gate; 8 needs all of 3–6). Next block honors the branch: PASS → 3,4,6 parallel; FAIL → step 3 same-day Solidity rewrite with scope/revert-strings/tests unchanged, and 4,5,6 are genuinely unaffected because the ABI/revert reasons are pinned in step 1's `packages/shared`.
- **Parallelism is real, not a chain.** Batches [1] → [2,5] → [3,4,6 (+5)] → [7,8] → [9] → [10]; max width 4; critical path ≈ six rounds. No undeclared file collisions in any parallel pair (2↔5, 3↔4↔6↔5, 7↔8): no rewritten file is shared; `deployments/*.json` and `packages/shared` are append-only by design and covered by the one-at-a-time merge rule. This is a fan-out build, not a 10-step chain.
- **Calendar realistic.** Sep 12 → Oct 3 submit with the Oct 4 23:59 SGT close as true slack; gate cannot slip and nothing front-runs it; the only compression risk is the 7↔8 edge below.

## Gaps

[gap] Step 8 tests against live 4663 contracts no declared dependency produces — step-8 scope says "live 4663 contracts and replicas", its stub wallet is funded on 4663, and task 5 seeds registry records on 4663; but the first 4663 registry+guard deploy is step 7 task 5 (step 3 explicitly excludes 4663: "4663 lands in step 7"), and the tracker launches 7 and 8 in parallel with 8's depends-on = 3,4,5,6 only. Fix before the Sep 27 batch: add 7 to step 8's depends-on (9 inherits it transitively — it has the same hole), or re-scope step 8's on-chain specs to step 3's 421614/46630 deployments and fold the 4663 pass into step 9's dry runs + step 10's production smoke.

[gap] Step 5 consumes an artefact no step creates — step-5 task 4 reads the registry "ABI from `packages/shared`", but step 1 ships wire types + golden fixtures only, no contract ABI; step 5 builds in the first parallel batch, before step 3 exists. Smallest fix: step 1 adds the minimal read ABI to `packages/shared` — the signatures are already fixed in step-3's plan text (`getRecord(token)`, `verify`/`revoke`, `commit`/`execute`) — and the same module appends step 2's calibrated probe selectors at merge (step 5's guard-preview needs the blocklist selector; its plan wrongly calls step 2 findings demo-copy-only).

[gap] The registrar-key handoff is unpinned — the registry's registrar is "set once" at deploy (step 3 task 1) and the backend signs revokes with `REGISTRAR_KEY` (step 4 task 6); no step makes them the same key, and a mismatch makes every drift-revoke revert as a non-registrar write — killing the demo's upgrade→revocation beat — with only step 10's secrets audit mentioning the key. Smallest fix: one line in step 7's 4663 deploy task — generate the registrar key first and deploy with (or registrar-transfer to) the backend `REGISTRAR_KEY` address; scratch deploys may keep throwaway keys.

## Verdict

3 gaps. Coverage, phases, ordering, parallel-safety, and calendar are sound; the set reaches a finished, submittable product once the three gaps above are patched — gap 1 is the blocking one (it stalls step 8 mid-build or silently re-scopes its on-chain coverage).

---

# Plan judgment — round 2, reconciliation re-check (2026-09-12)

Scope of this pass: not a fresh judgment — step 1 is done (5e2ff35) and a3159d7 folded step-1 findings into plans 2–10 (record: `reconciliation/2026-09-12-step-1-findings.md`). Checked the nine `> **Revised**` notes against each other, the tracker's dependency graph, the repo at a3159d7, and spec.md/journeys.md. Round-1 gaps 1–3 are all closed in current plan text (details below). Set remains SHIPPABLE.

## (1) Reconciliations keep the set SHIPPABLE — no contradiction with tracker or each other

- Round-2 gap-1 fix verified: step-8 goal now reads "does NOT target 4663 — the first 4663 registry+guard deploy is step 7, which runs in parallel; the live-4663 journey pass is step 9's dry runs + step 10's production smoke"; task 5 seeds scratch-chain state with step 3's integration registrar key; step-9 task 6 carries the 4663-only-divergence blocking-fix rule; step-10 task 3 is the production smoke. Step 8's depends-on (3,4,5,6) is now true — its on-chain targets (421614/46630) are written by exactly 3 and 6 (Revised note 2 names them).
- Round-2 gap-2 fix verified in the merged repo: `packages/shared/abi.ts` ships REGISTRY_ABI/GUARD_ABI/SELECTORS/GUARD_REVERT_REASONS literals plus the PROBE_SELECTORS reserved block (:49/:57, verified). Step 2's only touch outside `spike/` is replacing that block at merge with the shared vitest suite as tripwire (task 5); step 5 imports the constant and its mock-mode tests are fixture-driven so placeholders are tolerated (task 3) — step 5's depends-on=1 stays true, no hidden step-2 edge.
- Round-2 gap-3 fix verified: step-7 task 5 pins the registrar-key handoff (generate → set backend `REGISTRAR_KEY` → deploy with / transfer to that address); step-10 task 2 audits registrar-address equality.
- No Revised note contradicts another: serde-rename wire naming (steps 3/4) matches step-1 findings' `RECORD_REVOKED` note; on-chain `GUARD_*` strings stay byte-pinned in step-3 task 3 and are imported from `GUARD_REVERT_REASONS` by step 8 (Revised note 1). Worker-build correction (steps 2/4) matches the merged `scan-backend/wrangler.toml`. RPC pre-probe notes (3/4/5/6/7/8) all point at the same wire.md/chains.ts pins, verified live in the tree.

## (2) No new collisions between parallel steps

- Batch [2,5]: step 2's one shared file (`packages/shared/abi.ts` PROBE_SELECTORS) is a merge-time replacement absorbed under the tracker's one-at-a-time merge rule; step 5 is import-only (Revised note 4). No rewrite overlap.
- Batch [3,4,6 (+5)]: scopes are disjoint (`contracts/core` vs `scan-backend/**` vs `contracts/replicas` + `scripts/deploy/replicas` + `demo/assets.md` vs `frontend/src/**`). The one shared file pair — `deployments/421614.json`/`46630.json`, appended by 3 (pinned writer) and 6 — is append-only, declared in both plans' Revised notes with the merge-conflict note (tracker Next §4). Append point, not a dependency. packages/shared touches on both sides are append-point module adds only.
- Pair [7,8]: disjoint — 7 touches `contracts/core` tests/docs + `docs/` + `deployments/4663.json` (a file step 3 explicitly never writes); 8 creates `e2e/**` only.
- Repo grounding: `deployments/{worker,pages}.json` exist (step 9's Revised URL sources resolve); `frontend/src/components/verdicts.ts`, `lib/chains.ts`, `lib/wagmi.ts` exist as step 5's Revised notes cite; `spike/` and `docs/` absent as expected (creates by steps 2 and 7; step 5's `docs/criteria.md` link is the declared step-7 handoff, checked in step 7's own Check line).

## (3) Spike-gate FAIL branch still intact

Step 3 Stack + open question carry the same-day Solidity rewrite (identical scope/revert strings/tests, foundry replaces cargo-stylus, tracker row stays step 3, coordinator flagged). Revised note 5 explicitly hands the pinned SELECTORS/revert strings to the FAIL branch; step 3's Resources cite contract-verification.md for the `forge verify-contract` Etherscan-v2 recipe; steps 4/5/6 are genuinely unaffected because they build against step-1's `packages/shared` pins, and step 6 is Solidity regardless. Step 7 Stack covers "or the Solidity branch of it". Gate date (2026-09-18, step-2 task 10) unchanged; steps 3/4/6 still cannot start before the GATE line exists (declared in each learned line).

## (4) Nothing broken or dangling

Task-number references in every Revised note match the owning plan's Tasks section (re-verified: step 2 T2/T7, step 3 T1/T3/T5, step 4 T1/T4, step 5 T1/T3, step 6 T7, step 8 T1/T3/T6). Forward `learned:` references to not-yet-existing findings.md files are by design. The step-1 findings `reconciled:` dating is deliberately withheld until this verdict (reconciliation record, Deferred) — reconcile-agent can date it now. Known non-gap, unchanged from round 1: the tracker's files-in-scope column for step 6 doesn't list the deployments append the plan declares — the plan text is authoritative and consistent.

## Verdict

SHIPPABLE. The reconciled set is internally consistent, the dependency graph matches actual file behavior, the gate branch survives, and the three round-1 gaps are closed in merged or planned text. No new gaps introduced by a3159d7.

---

# Plan judgment — round 3, reconciliation re-check (2026-09-19)

Scope: the fee34b0 reconciliation pass (step-2 spike findings + step-5 frontend findings into plans 3, 4, 6, 7, 8, 9, 10). Sources read: `spike/findings.md` (repo, GATE PASS-PENDING-GAS), `.agent-workbench/step-5-frontend-screens/findings.md` (e37bc15), record `reconciliation/2026-09-19-steps-2-and-5-findings.md`. Tracker unchanged: steps 1, 2, 5 done; next batch 3, 4, 6 parallel. The six questions put to this pass:

1. **Beacon-not-proxy carried correctly — yes.** Step 3: Revised notes 2–3 + task 3 rewrite the guard checks (paused 0x5c975abb → token proxy, isBlocked 0xfbac3951 → resolved beacon, impl = beacon slot + beacon's `implementation()` with the extsload path kept for 0x5c chains) and task 4's test matrix requires mock tokens with blocklist on the beacon so the beacon-probe path is exercised, not just compiled around. Step 4: Revised note 1 + task 1 probe at calibrated targets, task 3's blocklist power-report evidence = the beacon probe result. Ground truth verified in the tree: `packages/shared/abi.ts:49-63` PROBE_SELECTORS calibrated with the beacon-vs-proxy comment. The same semantics propagate consistently to step 6 (replica beacon answers isBlocked, notes 1–2), step 7 (gas report includes the beacon staticcall), step 8 (assert blocklist evidence from `/scan`, not the preview), step 9 (narrate from the `/scan` payload) — one design, seven plans, no contradiction.
2. **Canonical-fetch worker: absorb, consistently.** Step 4 Revised note 2 + task 3 port `spike/fetch/` (exists: src/Cargo.toml/wrangler.toml verified) into the scan worker; spike URL demoted to fallback mirror, no hard dependency. Step 9 Revised says "absorbed into step 4's worker"; step 10 Revised says smoke confirms the absorbed path, not the spike URL; step 8 only cites the worker as provenance. No use-vs-absorb conflict anywhere.
3. **VITE_* env contract lands coherently.** Step 3 note 6 emits the addresses (its claim that frontend error strings point at `deployments/4663.json` verified literally: `wallet.ts:85`, `guard.ts:281`, `registry.ts:112`); step 7 note 1 sets them from `deployments/4663.json` and records both addresses in findings for the handoff; step 8 note 1 defines the mock-default/live-flip contract (`VITE_API_MODE` + `VITE_API_URL` + the two addresses, scratch deployments → step 3's depends-on is true); step 10's audit list is concrete. Chain 3 → 8 (scratch) and 7 → 10 (production) is closed.
4. **Stylus branch + gas re-assertion: correct.** Step 3 note 1 marks the Solidity branch DEAD (kept for record), declares the pending spike receipt NOT a blocker, and task 5 re-asserts ≤200k from this step's own receipts as the first live gas evidence; note 4 bans citing step-2 numbers; step 7 note 2 uses step-3 integration receipts as the diff base and folds in the DEPLOY.md receipt if it lands; step 6 note 4 says the same. Consistent and non-blocking as intended.
5. **No new collisions in the 3/4/6 batch.** Scopes disjoint: `contracts/core/**` + `scripts/deploy/core/` vs `scan-backend/**` vs `contracts/replicas/**` + `scripts/deploy/replicas/` + `demo/assets.md`. Only shared files are the `deployments/*.json` appends — append-only, declared in step 3 note 4 and step 6 note 2, covered by the one-at-a-time merge rule; `spike/fetch/` is read/copied by step 4, not edited. The step 4 ↔ 6 fingerprint composition is a declared joint decision (both name `spike/evidence/calibration_4663.json`, verified present, and the same hit/miss rule), not a file collision; step 8's e2e detects divergence if the parallel decisions drift.
6. **Routed defect is durably recorded — yes.** Step 8 Revised note 2 (committed in fee34b0) names the code (`frontend/src/lib/guard.ts:186-188` — verified real: `isBlocked` probes the token with the blocklist selector), the safe degradation, the e2e assertion rule, and routes the fix to the step-5 owner per step 8's scope rule; the reconciliation record's Escalation 3 puts the fix-owner decision on the coordinator. Two committed copies; the fix itself is coordinator-dispatched and no journey depends on the preview row (e2e and demo both use the `/scan` payload).

## Notes (non-gaps)

- Step 7's Stack line still reads "Same as step 3 (or the Solidity branch of it)" — the branch is DEAD per step 3's note 1. Cosmetic residue; step 7's own learned line already states activation is proven. No action required.
- All `learned:` references repointed to `../../spike/findings.md` resolve (file verified at the repo path); forward references to not-yet-written findings.md are by design.
- Withheld `reconciled:` stamps on both findings files may now be dated on this verdict. On the record's open question — stamping `spike/findings.md` inside the repo deviates from the `.agent-workbench/`-only commit rule — accepting the reconciliation record as authoritative is sufficient; the workbench stamp on step-5 findings should still be applied.
- Coverage/phases/ordering were judged complete in rounds 1–2; this pass only edited plans 3–10 within existing scopes, and nothing in the diff changes a scope boundary or a dependency edge.

## Verdict

SHIPPABLE. The reconciliation pass carries both findings correctly across all seven plans: the beacon-not-proxy design is consistent end to end, the worker absorb is coherent, the env contract closes, the gate decision is propagated without blocking, the parallel batch stays collision-free, and the routed defect is recorded with a working around-path. Zero gaps introduced by fee34b0.

---

# Round 3 (2026-09-20) — reconciliation of steps 3+4+6 findings into plans 7–10 (commit 4b52e6a)

Set judged: plans 7–10 as amended by 4b52e6a, against step-3/4/6 findings, `deployments/README.md`, the worker source, and `spec.md`/`journeys.md`. Ground truth re-verified live: `frontend/src/lib/api.ts` has NO `provenanceUrl` (only `packages/shared/watchdog.ts:22`); `deployments/4663.json` does not exist yet; worker reads `REGISTRY_ADDRESS_{4663,46630,421614}` (`scan-backend/src/lib.rs:107–109`, all empty in `wrangler.toml:13–15`).

## What holds

1. **Every listed reconciled item traces to a plan line.** Step-3 first-funded-run routing + script facts → step-7 plan.md:19 (Revised note 1) and :46 (task 5); gas-actuals diff → step-7 plan.md:43 (task 2: actuals "did NOT exist as of 2026-09-20", first funded run precedes the diff) — matches step-3 findings.md:22–24. Two-secret handoff (REGISTRAR_KEY + DRIFT_ADMIN_SECRET, generate/set) → step-7 plan.md:21 + :46, audited in step-10 plan.md:16 + :36 — matches the live worker state in step-4 findings. Honest-UNVERIFIED degrade + retry guidance → step-8 plan.md:22 + :43, step-9 plan.md:16 + :39, step-10 plan.md:17 + :36 — matches step-4 findings' live receipt outcome. N3 (F2–F4 subset): no plan in 7–10 asserts full FINGERPRINT.md mirroring, so the deliberate-subset approval in the record (line 33) stands uncontested. N4 measured figures 258,707/~2,630 with provenance, mocks stay → step-9 plan.md:18 + :41 — matches step-4 findings' N4 + "mocks stay" rule. N7 provenanceUrl → step-8 plan.md:21 + :45 (fold into frontend type here, sentinel asserted). 4663.json writer correction → below. Replica-beat reuse → step-9 plan.md:19 (choreography, clean-deploy cast, implV2, replicas deploy.sh for `scripts/seed-demo/`) — matches step-6 findings' step-9 bullet verbatim. forge-install no-op gotcha → step-7 plan.md:23 — matches step-6 findings' "Where the plan was wrong".
2. **Dependency order still sound.** 7 ∥ 8 is behavior-true: step-8's Goal (:29) explicitly refuses the 4663 target and runs against scratch deployments, so nothing in 8 reads a step-7 output. 9 consumes 7 (DRIFT_ADMIN_SECRET handoff for the manual drift-check beat; live 4663 deploy; task 6's first-4663-journey-pass rule) and 8 (green suite). 10 consumes 7 (secrets audit, verify re-check) + 9 (runbook smoke). No cycles, no forward references; all cited `learned:` findings belong to already-done steps.
3. **4663.json writer correction consistent in all three places.** `deployments/README.md:13` ("step 6 creates it with the `replicas` field … step 7 appends the core deploy"), step-6 findings deploy-order bullet (4663 creates with `replicas`), step-7 Revised note 2 (:20) + task 5 (:46, "after step 6's replicas field"). The stale 2026-09-11 note (step-7 plan.md:12) is explicitly named and superseded by note 2 — a builder reads the correction in place.
4. **No write collisions 7 vs 8 — with one correction to the premise.** 8's true write-set is no longer "e2e/** only" (see gap 2). Beyond that: `deployments/4663.json` is 7-write/8-read and 8 never targets 4663; `docs/criteria.md` is 7-write only (8 asserts the criteria-panel UI, not the doc file); 8's nightly e2e workflow is a new CI file and 7 touches no workflow; `gas-report.md`, `contracts/core/**`, `docs/` are 7-only; `frontend/src/lib/api.ts` is 8-only. Clean.
5. **Spec's v1 cut fully planned in the 7–10 remainder.** Criteria published → 7 task 3 (`docs/criteria.md`, the path step 5's panel links); 4663 deploy + source-verify + mirror → 7 task 5, re-checked in 10 task 2; all journeys + unhappy paths → 8; arc + runbook + video + seed + README → 9; freeze/tag/smoke/checklist/package → 10. Carried flags are honest, not unplanned: watchdog live-counter mechanism stays the coordinator flag with the measured-copy correction routed to 9; funding stays the sole operator blocker (record escalation 1); VERIFIED P receipt is task-level in 9 (task 6) and 10.

## Gaps

- [gap] Worker `REGISTRY_ADDRESS_4663/46630/421614` fill is unowned — the deployed worker reads them (`scan-backend/src/lib.rs:107–109`, empty per `wrangler.toml:13–15`) and step-4 findings.md:52 assigns the fill to "step 3/7 deploys", but no task in plans 7–10 names them (every `REGISTRY_ADDRESS` hit in the plans is the frontend `VITE_` var — a different mechanism). Unset, `/scan` serves `record: null` and degrades to canonical-fetch-only: step-8's live J1 REVOKED spec cannot pass and step-9's manual drift-check beat has no records to poll (clean skipped report) — both surface mid-build — belongs in step-7 task 5's handoff (set 4663 immediately after the deploy, from `deployments/4663.json`) plus one clause in step-8 task 5 (fill the scratch vars from the funded `deployments/*.json` before the live pass).
- [gap, minor] Step-8's own Scope line contradicts its reconciled task — plan.md:32–33 says "Creates: e2e/** only" and routes product fixes "back to the owning step", while Revised note 1 (:21) + task 4 (:45) deliberately edit `frontend/src/lib/api.ts`; the tracker's step-8 files-in-scope column still reads "e2e/**" — no collision with 7 today (7 never touches frontend/), but the contradiction invites the builder to route N7 back out and re-opens the merge-time collision ledger on stale data — amend step-8's Scope line and the tracker column to carry the one-file exception.

## Stamps

With this ruling the `reconciled:` stamps on step-3/4/6 findings may be dated. The tracker's Next block (7+8 parallel → 9 → 10, submit 2026-10-03) needs no change; only the step-8 files-in-scope cell (gap 2) is stale.

## Verdict

2 gaps (1 substantive, 1 minor). The reconciliation itself is faithful — every named item landed where the record claims, the writer correction is consistent everywhere it appears, and ordering/collisions hold. Gap 1 is the one that would have surfaced mid-build with the plan already trusted.

---

# Round 4 (2026-09-21) — reconciliation of steps 7+8 findings into plans 9–10 (commit ef21e2b)

Set judged: plans 9–10 as amended by ef21e2b, against `reconciliation/2026-09-21-steps-7-and-8-findings.md`, step-7 findings (93201ab) and step-8 findings (b17fdd3), the tracker, and spec.md. Steps 1–8 done; 9 and 10 planned. Ground truth re-verified live on main ef21e2b: `e2e/scripts/seed-scratch.sh` is `bash -n` clean; `e2e/README.md:41` is the activation runbook; the J3 empty-state unit pin exists at `frontend/tests/registry.test.tsx:76–79`; all four worker vars are empty at `scan-backend/wrangler.toml:13–21` incl. `DRIFT_EXTRA_TOKENS`; `deployments/` has no chain JSON; the funded-run runbook (7 steps, incl. the `REGISTRY_ADDRESS_4663 --var` pass at findings :109–113) is in step-7 findings :66–160.

## What holds

1. **Every carried item traces to a plan line.** Funded-run runbook as step-9 prerequisite → step-9 plan.md:10 (learned), :23 (Revised note 1), :51 (task 6 "Prerequisite: … has executed"). seed-scratch.sh inheritance (parse-fixed, never executed, `bash -n` clean) → step-9 :26 (note 4) + :46 (task 1 ports the two-sided swap-settle plumbing). Activation-gated e2e skips' path → step-9 :26 (e2e/README.md:41 runbook, live project once post-runbook as the regression check). Stale runbook-text correction (process carry 5) → step-9 :25 (note 3: "the runbook text 'step 8 re-passes' is stale … first real pass is the runbook's"). J3 empty-state narration honesty → step-9 :27 (note 5). Dead-helpers freeze note → step-10 :24 (note 4) + :42 (task 1). Registrar-key/drift-secret facts → step-9 :24 (note 2) + :48 (task 3); DRIFT_EXTRA_TOKENS ownership + `--var` gotcha → step-9 :25 + :48, audited step-10 :22 + :43. Writer-of-record + vars-audit + verify-not-re-run → step-10 :21–23 + :43.
2. **Dependency order coherent.** 9 depends on 7,8 — both done. 10 depends on 7,9 — 9 still planned, so 10 correctly waits. No cycles, no forward references beyond the by-design `learned: ../step-9-demo-harness/findings.md` in step-10's Resources (unwritten until 9 merges). Round-3 gap 1 (worker vars fill unowned) is now closed: runbook 6c owns the first pass (step-7 findings :109–113), step-9 note 3 owns re-passes, step-10 note 2 audits what IS set. Round-3 gap 2 (tracker step-8 scope column) is fixed in the tracker row ("+ one-file exception").
3. **Step 9 is executable in both regimes — with one wording snag (gap 2).** Funded: runbook executes (any builder, no build work — note 1), then tasks 1–6 build and task 6 dry-runs as the first full 4663 journey pass. Unfunded: tasks 1–5 (seed scripts, runbook/arc.sh text, upgrade-beat scripting, video script, README) have no funding dependency in their text; task 6's explicit prerequisite + note 4's live-e2e flip are the staged gates. Dispatch is safe in either regime.
4. **Spec's submission requirements fully planned across 9+10** (except gap 1's disclosure): video script + recording → 9 task 4, final video in package → 10 task 5; README/quickstart/criteria-link/demo-gif/cast-labeling → 9 task 5; repo hygiene + freeze + tag + CI-at-tag + clean-checkout build → 10 task 1 + scope line; pre-submission checklist (spec.md:71: Token Sniffer/De.Fi spot-check, EAS + GoPlus rehearsal) → 10 task 4; package upload with ≥24h buffer (2026-10-03 EOD target) → 10 task 5; live URLs + contract addresses → 10 tasks 2/5 + Check line; Q&A residual risks (spec.md:79–80) → 10 task 6. Journeys stay reachable: J1/J2/J3 green, the 3 activation-gated skips get their activation path in step-9 note 4.
5. **erc20() junk-decode — decision received, not yet landed (gap 1).** Verified real in step-7 findings :148–152 ("Out of scope, left broken"); grep of both plans returns zero hits for erc20/junk/security.

## Gaps

[gap] The coordinator's erc20() junk-decode decision (carry to judges' Q&A + prominent security-note disclosure in step-10's submission docs; NOT a sanctioned contracts change) exists nowhere in the plans — step-10 task 6's residual-risk list names only spec.md:79/80 and task 5's submission package has no security-note slot, so a freeze-time builder would submit without the one disclosure the coordinator has ordered — belongs in step-10 (before any step-10 dispatch) — smallest fix: one clause in task 6 adding the guard `erc20()` junk-returndata item to the Q&A list (cite step-7 findings.md:148–152, worst case: phantom active order for a lying token's buyer, execute reverts whole, funds safe) and one clause in task 5 adding the prominent security-note to the submission docs/README.

[gap, minor] Step-9 Revised note 1's header "the funded-run runbook executes at or before this step's start" contradicts the unfunded dispatch regime the coordinator intends — a literal builder dispatched before funding reads it (plus the record's escalation 1 "gates step 9's start") and stalls instead of starting tasks 1–5, whose text is funding-free; task 6's prerequisite is the operative gate and already says the right thing — belongs in step-9 note 1 (before dispatch) — smallest fix: reword the header to gate task 6's dry runs explicitly, e.g. "executes before the first dry run (task 6); non-chain tasks 1–5 proceed unfunded."

## Stamps

With this ruling the withheld `reconciled:` stamps may be dated: step-8 findings.md:5 has the empty line awaiting its date; step-7 findings.md has no `reconciled:` line at all — add one. Tracker needs no change for these gaps (both fixes are plan-text clauses; statuses stay 9/10 planned).

## Verdict

2 gaps (1 substantive, 1 minor). The reconciliation is faithful — all carried items trace to plan lines, the dependency graph is true, round-3's gaps are closed, and the two-regime dispatch works. Both fixes are single-clause plan edits; once applied, the set ships: steps 9 then 10 reach the spec's success line with the submission package, freeze, and disclosures complete.

---

# Round 5 (2026-09-21) — reconciliation of step-9 findings into step 10 (commit aecd05a)

Set judged: step-10 plan.md as amended by aecd05a (Revised note 6, items 1–4) against `reconciliation/2026-09-21-step-9-findings.md`, step-9 findings (merged 7841eaf, `reconciled:` still empty by design), the step-10 verify pass (untracked verify.md, produced against the PRE-aecd05a plan — its plan.md:44–47 line refs predate note 6), the tracker (steps 1–9 done, 10 planned), and spec.md. Ground truth re-verified live on main aecd05a: `deployments/` holds only pages.json/worker.json/README.md (note 6.1's trigger condition is real at step-10 start); step-9 findings carries the "Task-6 go-list" as numbered items 1–8; `demo/runbook.md` §6 "Failure appendix" exists; README.md:1 and `frontend/src/lib/brand.ts:2` still carry the placeholder framing verify LE 3 cites. Steps 1–9 done-state untouched: the diff is the new record file + 6 plan lines, nothing else.

## The four items trace

1. **Go-list as production deploy path → note 6.1 (plan.md:28).** Condition-based on `4663.json` absence at step-10 start (verified absent), cites go-list items 1–8 which exist verbatim in step-9 findings:46–54 in the same order (runbook → replicas before stage F → one worker reconfig → seed-demo → live playwright → serve → dry runs). Task 2 becomes verify-mode, task 3 runs on its output — both bindings named inside the note.
2. **Name-lock resolution + demo.gif → note 6.2 (plan.md:29).** fc8b004 on main is the lock commit (message confirms "PRODUCT NAME LOCKED: Vetted"; tracker row 9 set done). The demo.gif uncomment is bound to task 5 inside the note; verify LE 3 will append it to the task line proper — refinement, not conflict.
3. **P-receipt routing → note 6.3 (plan.md:30).** Matches step-9 findings go-list step 7 ("between takes ... record it here when one lands") and its out-of-scope line; smoke chases only as fallback ("only if step-9's findings still show it pending at step-10 start"). The literal reading does make task 3 chase when the go-list ran at step-10 start after findings showed pending at start — chasing via `live-check.sh` is idempotent, and by then the receipt is either landed (nothing to chase) or captured on the spot. No contradiction.
4. **Smoke re-runs via runbook §6 → note 6.4 (plan.md:31).** Matches step-9 findings "What the plan was wrong" (monotone registry transitions, seed-demo exits with the recovery path) and runbook §6 exists.

## Both funding regimes executable

Note 6.1's dispatch is condition-based (file absent or not), not timing-based, so it is deterministic however funding lands: absent-at-start + funded → go-list first, tasks 2/3 verify/smoke its output; funded + already ran → explicit no-op ("audit, don't redo"). The never-funded arm is exactly verify LE 1's prescribed fix (go/no-go date + declared unfunded variant or escalation), to be applied by the builder right after this round — the two compose: 6.1 is the funded-but-unexecuted arm, LE 1 is the funding-never-lands arm. A builder applying LE 1 should word it as the funding-absent branch of note 6.1 so "execute the go-list FIRST" is not read as unconditional.

## Verify loose ends vs note 6 — compatible

- **LE 1 ↔ 6.1:** complementary arms, no overlap in trigger (see above).
- **LE 2 ↔ 6.2/6.3:** disclosure-only (known-limits list mirrors what the live URLs show, incl. P-receipt status at freeze); no routing conflict with 6.3's chase rules; regime-aware wording matches 6.1's branches.
- **LE 3 ↔ 6.2:** 6.2's "brand.ts is final as-is" fixes the name VALUE ("Vetted" already in the constant); LE 3 strips the stale placeholder LABEL at brand.ts:2 and README.md:1 (anchors verified present). Value-final and label-cleanup do not conflict; the builder edits the comment, not the constant.
- **LE 4 ↔ 6:** tag-at-end-of-task-5 (or tag=freeze + re-cut rule) is consistent with 6.2's gif-uncomment landing in task 5 before the tag — tagged tree = submitted tree. No interaction otherwise.
- **LE 5 ↔ 6.3 (caveat, not a gap):** today the repo is public and 6.3's "step 10 links it from there" (the step-9 findings path) resolves. Only if the coordinator's LE-5 decision lands on the SCRUB branch would that package link die with `.agent-workbench/**`. The LE-5 fix already forces the decision before freeze; under the scrub branch the same pass should copy the receipt (or its live-check evidence) into `submission/**` rather than link the workbench path. One clause, decided at the same moment — no plan line is wrong today.

## Verdict

SHIPPABLE, 0 gaps. All four reconciled items trace to concrete note-6 lines with live triggers and resolvable targets; step 10 executes in both regimes; the five verify prescriptions compose with note 6 without contradiction (LE 1 = the unfunded arm of 6.1; LE 3 = the label cleanup 6.2 deliberately doesn't cover); nothing in steps 1–9's done-state is disturbed. With this ruling the withheld `reconciled:` stamp on step-9 findings.md may be dated (standing pattern), and the builder applying verify LE 1–5 may dispatch.
