# Reconciliation 2026-09-20 — step-3 (contracts-core) + step-4 (scan-backend) + step-6 (replica-assets) findings

Findings processed (all MERGED, all unreconciled at round start; stamps WITHHELD pending the judge):

- step-3 contracts-core — `.agent-workbench/step-3-contracts-core/findings.md` (merged 10fb0d3; on-chain receipts deferred — operator key 0x151e…8E4C balance 0 on 421614/46630/4663, re-verified 2026-09-20).
- step-4 scan-backend — canonical at `.agent-workbench/step-4-scan-backend/findings.md` on main (merged 559344c + status commit 7926526, both ancestors of origin/main f10787e). The copy in step-4's worktree at `/tmp/step4-wt` was diffed against main's: **IDENTICAL** — main is canonical, the worktree held no divergent content.
- step-6 replica-assets — `.agent-workbench/step-6-replica-assets/findings.md` (merged 8c80aad). **Found by the sweep, not named in the dispatch:** its `reconciled:` header was empty (exactly what step-4's own "out of scope" note predicted the Next-1 sweep would cover). The standing rule is to process every unreconciled findings.md, so it was folded in too; its substance partially reached steps 3/4 at build time (branches sat on its merge) but steps 7–9 had not consumed it.

Ground truth verified before editing (all on origin/main f10787e): `scripts/deploy/core/deploy.sh` and `scan-backend/scripts/live-check.sh` exist; `deployments/` holds only pages/worker/README + `deployments/README.md` pins `4663.json` writers as **6 (replicas field, creates) then 7 (core append)**; `frontend/src/lib/api.ts` has NO `provenanceUrl` (N7 confirmed live) while `packages/shared/watchdog.ts:22` has it; the step-6 `ponytail:` pointer in step-3's findings was stale — the marker is at `contracts/core/guard/src/lib.rs:587` (corrected in step 7's plan); worker version 0ccf8c68 is the deployed live bundle.

## Plans changed (all four not-yet-built steps)

- **step-7-contract-hardening/plan.md** — New Revised block (2026-09-20, 5 notes) + learned lines for step-4 and step-6 findings:
  1. First-funded-run routing: the FIRST funded run anywhere must execute `deploy.sh` END-TO-END (stage D/E receipts + broadcast `receipts` array) so broadcast-JSON drift surfaces before 7/8 consume `deployments/`; if step 3's funded run hasn't happened by step-7 start, step 7's 4663 run is the first. Script facts folded (`--chain 4663` accepted, `--receipts-only`, `MM_KEY` two-party accounting).
  2. `deployments/4663.json` writer correction: the 2026-09-11 "this step is the pinned writer" note predates step 6's merge — README now pins 6-then-7; step 7 appends after step 6's replicas field. Task 5 amended accordingly.
  3. Task 5 handoff is TWO secrets: `REGISTRAR_KEY` + `DRIFT_ADMIN_SECRET` (step 4's worker ships both deliberately unset — drift-check degrades clean until set; step 9's manual drift-check beat needs it).
  4. Task 2 gas diff: the step-3 actuals are deploy.sh's committed receipt records and did not exist as of 2026-09-20 — produce them first. Receipts also carry what native tests cannot (probe→target wiring, zero movement on five revert paths — step-3 round-1 deferrals).
  5. Left-broken items step 7 owns: 8 cargo warnings (task 4) + `ponytail:` marker at `guard/src/lib.rs:587`; foundry `forge install` no-op gotcha from step-6 findings.
- **step-8-e2e-journeys/plan.md** — New Revised block (2026-09-20, 3 notes) + task amendments:
  1. N7 lands here: fold `provenanceUrl` into the frontend `WatchdogStats` (`frontend/src/lib/api.ts`) as part of J3 work; assert the sentinel shape (`runs: 0` + `provenanceUrl != null` = unavailable, never a measured zero).
  2. Honest degrade is an outcome, not a flake: `RPC_RETRYABLE`/honest-UNVERIFIED are legitimate (live-verified 2026-09-20); specs get a retry policy per step-4's guidance and, when the degrade is exercised, assert no ABSENT row with `severity: verified` was fabricated (task 2 amended).
  3. The live scratch pass is gated on FUNDED runs: step 3's core run AND step 6's replica run both pending (0 wei on all chains) — if funding hasn't landed at step-8 start, the live pass waits; mock-mode specs run regardless; never hand-forge deployment entries (task 5 amended).
- **step-9-demo-harness/plan.md** — New Revised block (2026-09-20, 4 notes) + task amendments:
  1. Scan retries built into `demo/arc.sh` + runbook fallbacks (task 2): on `RPC_RETRYABLE` wait + re-scan; on honest UNVERIFIED use the zero-fabricated-evidence framing. Paid/private RPC is the upgrade path — decision is step 10's.
  2. Capture the still-missing VERIFIED-shaped P receipt via spaced `scan-backend/scripts/live-check.sh` runs between timed takes (task 6 amended).
  3. Watchdog copy correction (task 4 amended): the spec's 6,092/~150-per-day figures are measured STALE (live batchCount 258,707, ~2,630/day, 2026-09-20) — demo copy quotes the measured figures WITH provenance; the `/watchdog` degrade sentinel and the 6092/150 frontend mocks stay until a reconciled counter exists — narrate the widget as baseline/illustrative, never live.
  4. Replica beats reuse step 6's assets: `demo/assets.md` beat choreography, clean-deploy cast, `implV2` upgrade target, `scripts/deploy/replicas/deploy.sh` for seeding — do not re-derive.
- **step-10-deploy-submission/plan.md** — New Revised block (2026-09-20, 2 notes) + learned line for step-4 findings + task 2 amendment: secrets audit now verifies `DRIFT_ADMIN_SECRET` set alongside `REGISTRAR_KEY` (address = registry registrar), and the demo-window paid/private 4663 RPC decision happens BEFORE the production smoke (re-smoke `/scan`, capture the VERIFIED P receipt as a submission asset).

## No plan changes needed beyond the above

- step-4 R3-N1 (blocklist `Ok(_)` conservativeness — fold a `decode_abi_bool` check) and R3-N2 (two None-branches unpinned): no step 7–10 touches `scan-backend/src` (7 = contracts tests/docs, 8 = e2e/**, 9 = demo/**, 10 = deploy orchestration) — they stay coordinator-carried for whenever the scan-backend suite is next touched.
- N3 (matcher = F2–F4 subset of step-6 FINGERPRINT.md): reviewed and approved as deliberate; no plan text assumed otherwise.
- N5/N8 knowingly kept (N8's `estimate_gas` headroom is now cited by steps 9/10 as the paid-RPC upgrade seam).
- Step-3 events/beacon facts (`packages/shared/events.ts` shipped with the merge, record.reason = keccak256(utf8(reason)), extraction-failure skips): already covered by step 8's existing learned line + Revised notes; no new claim needed.

## Escalations for the coordinator

1. **Funding is the sole blocker on the contract side** (step-3 + step-6 deploys, receipts, gas actuals): `spike/DEPLOY.md` runbook; the first funded run must be the end-to-end `deploy.sh` run (now pinned in step 7's plan).
2. **Watchdog live counter mechanism remains UNPINNED** (carried coordinator flag from step 4): no L1 read reconciles with the published 6,092/150 figures; the product-copy correction (measured 258,707 / ~2,630/day) is routed to step 9's demo copy with provenance, and the frontend mocks stay stale until a mechanism is pinned.
3. **VERIFIED-shaped P receipt still pending** — needs a calm rate window + retries; capture is now task-level in step 9 (and a submission asset in step 10).
4. **Paid/private 4663 RPC decision** owned by step 10 before the production smoke (step 9 carries the retry fallback; the worker keeps the rpc headroom).
5. Process note carried from step-6 findings: the shared-root checkout incident (a builder's branch switching wiped another's commit mid-build) is why all builder work now runs in dedicated worktrees — keep enforcing that; do not run builders against the shared root.
6. step-4's `/tmp/step4-wt` worktree is redundant (its findings copy is identical to main's; branch tip 7926526 is merged) — safe to prune.

## Judge

No spawn tool in this context — plan-judge-agent must be dispatched by the caller (round logged as `reconcile-2026-09-20`).

**Judge round 1 (2026-09-20, plan-judgment.md:83–106): 2 gaps — 1 substantive, 1 minor; the reconciliation itself ruled faithful** (every listed item traces to a plan line, dependency order sound, writer correction consistent in all three places, no 7-vs-8 write collisions, spec's v1 cut fully planned).

**Gap fixes (this commit, per the verdict's own prescription — non-structural, no replanning needed):**
1. Worker `REGISTRY_ADDRESS_{4663,46630,421614}` fill was unowned (read at `scan-backend/src/lib.rs:107–109`, empty in `wrangler.toml:13–15`; step-4 findings assigned it to "step 3/7 deploys" but no task named it). Fixed: step-7 Revised note 3 + task 5 set `REGISTRY_ADDRESS_4663` from `deployments/4663.json` immediately after the 4663 deploy; step-8 gained Revised note 4 + a task-5 clause filling the scratch pair (`46630`/`421614`) from the funded `deployments/*.json` before the live pass.
2. Step-8 Scope line ("e2e/** only") contradicted Revised note 1 + task 4 editing `frontend/src/lib/api.ts`. Fixed: Scope line amended to carry the one-file exception (type field only, no behavior change), and the tracker's step-8 files-in-scope cell amended to match — that single-cell tracker edit is the only change to `step-feature-state.md` (judge-directed; statuses, Next block, and the plan-judge header untouched).

**Accepted + stamps applied (2026-09-20, this commit).** The coordinator accepted the gap fixes as non-structural (they follow the verdict's own smallest-fix prescription; nothing structural moved) — no further judge round; the caller committed the plan-judgment.md residue as b9aa17f. Stamps applied on this closure: `reconciled: 2026-09-20` on all three findings files at their canonical main copies (`.agent-workbench/step-{3,4,6}-*/findings.md`). Round closed — no unreconciled findings remain; steps 7+8 may fan out. Note: `/tmp/step4-wt` (step-4's worktree) is prunable — its findings copy was diffed identical to main's and its branch tip 7926526 is merged.

`reconciled:` stamps on step-3/4/6 findings remain withheld pending coordinator acceptance — the judge's Stamps section pre-authorizes dating them on this ruling (`reconciled: 2026-09-20`). Stamping note: all three findings files should be stamped at their canonical main copies (`.agent-workbench/step-{3,4,6}-*/findings.md`); step-4's `/tmp/step4-wt` worktree copy was diffed identical to main's this round, so main's stamp covers it and the worktree stays prunable.
