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
