# Plan state

round: 2
plan-judge: 1 run (2026-09-12) — 3 gaps, all fixed this round (gap 1: step 8 re-scoped to the scratch deployments with the 4663 pass folded into 9/10; gap 2: step 1 now ships the registry read ABI + PROBE_SELECTORS, step 2 appends the calibrated bytes at merge; gap 3: registrar-key handoff pinned in step 7's 4663 deploy). No spawn tool in this context — the caller dispatches plan-judge-agent and re-invokes plan-agent with the verdict.

Project: Vetted (placeholder) — verification + guarded swap for Robinhood Chain stock tokens.
Deadline: 2026-10-04 23:59 SGT. Today: 2026-09-12. Day-7 spike gate: **2026-09-18, cannot slip** (spec.md:50).
No epics ledger — v1 planned directly from the approved spec.md.

| step | phase | feature            | dir                           | status  | depends on | window              | files in scope (no collisions between parallel steps) |
|------|-------|--------------------|-------------------------------|---------|------------|---------------------|--------------------------------------------------------|
| 1    | 1     | repo-scaffold      | step-1-repo-scaffold/         | done    | —          | Sep 12–13           | repo root, CI, packages/shared/, frontend scaffold+tokens/primitives, toolchain docs, deployments/ layout |
| 2    | 1     | spike-gate         | step-2-spike-gate/            | done    | 1          | Sep 12–18 (gate 9-18; merged 9-19) | spike/**, spike/findings.md + one merge-time append: packages/shared/abi.ts PROBE_SELECTORS block |
| 3    | 2     | contracts-core     | step-3-contracts-core/        | done    | 1, 2       | Sep 19–26           | contracts/core/**, scripts/deploy/core/, deployments/*.json (append) |
| 4    | 2     | scan-backend       | step-4-scan-backend/          | done    | 1, 2       | Sep 19–26           | scan-backend/** |
| 5    | 2     | frontend-screens   | step-5-frontend-screens/      | done    | 1          | Sep 13–24 (starts parallel with step 2) | frontend/src/pages, frontend/src/lib, frontend tests |
| 6    | 2     | replica-assets     | step-6-replica-assets/        | done    | 1, 2       | Sep 19–26           | contracts/replicas/** (own foundry project), scripts/deploy/replicas/, demo/assets.md |
| 7    | 3     | contract-hardening | step-7-contract-hardening/    | planned | 3          | Sep 27–29           | contracts/core tests+docs only, docs/, gas report, 4663 core deploy |
| 8    | 3     | e2e-journeys       | step-8-e2e-journeys/          | planned | 3, 4, 5, 6 | Sep 27–30           | e2e/** |
| 9    | 3     | demo-harness       | step-9-demo-harness/          | planned | 7, 8       | Oct 1–2             | demo/**, scripts/seed-demo/ |
| 10   | 3     | deploy-submission  | step-10-deploy-submission/    | planned | 7, 9       | Oct 3 (submit; close 10-04 23:59 SGT) | deploy orchestration, submission/**, release tag |

Status vocabulary: `planned` / `done` / `blocked` — nothing else. The caller writes status rows from builder reports; this file is rewritten whole by plan-agent, carrying all statuses forward.

## Next

1. If any `step-*/findings.md` has a `reconciled:` line with no date after it, run `reconcile-agent` first — once, alone, with no builders running. (Standing rule; no findings.md exists yet.)
2. Runnable now: **step 1**. The moment it lands, run in parallel: **step 2** (the day-7 gate — funds/bridging start the same day; the decision cannot slip past 2026-09-18) **and step 5** (depends only on step 1; it builds against fixtures, not the backend). One `implement-agent` each, single parallel block, `isolation: "worktree"`.
3. Gate branch: step 2 writes `GATE: PASS|FAIL` as the first line of `spike/findings.md`. PASS → launch the wide batch **steps 3, 4, 6** — all parallel, one agent each, alongside whatever remains of step 5. FAIL → **step 3 is rewritten same-day in Solidity** (its plan carries the branch; scope, revert strings and tests are unchanged), and steps 4, 5, 6 proceed unaffected.
4. A builder returning READY TO MERGE has a rebased, reviewed branch. Merge those **one at a time** — run the full suite on the base branch after each merge, before the next (shared append points: `packages/shared/` index + the `PROBE_SELECTORS` block step 2 replaces, `deployments/*.json` — take the small conflicts at merge). Red base → revert that merge, mark the step `blocked`.
5. After steps 3–6 are merged: **step 7 and step 8** run in parallel (7: contracts/core tests+docs + the 4663 deploy with the registrar-key handoff; 8: e2e against the step-3/6 scratch deployments — it does not wait for 7). Then **step 9** (needs 7 + 8: 8's green suite and 7's live 4663 deploy — 9's dry runs are the first full journey pass on 4663; any 4663-only divergence is a blocking fix before recording), then **step 10** (needs 7 + 9). Submit by **2026-10-03 EOD SGT** — ≥24h buffer ahead of the close.
6. Set each merged row to `done`, run `check-workbench`, and go back to 1.

Judged-criterion mapping (why the steps look like this): contract quality → steps 3 + 7; PMF → registry-as-primitive read interface (3) + watchdog widget (4, 5) + J3 page (5); real problem solving → live 4663 verdicts incl. REVOKED and the canonical-fetch degraded branch (4, 5; the live-4663 journey pass lands via 9's dry runs + 10's production smoke — round-2 gap-1 fix); innovation/demo → the upgrade→revocation→guard-refusal beat (3, 4, 6, 9). Registry ABI consumed by 5/8 is step-1-created (`packages/shared/abi.ts`) — round-2 gap-2 fix; the registrar/REGISTRAR_KEY handoff is pinned in step 7's 4663 deploy — round-2 gap-3 fix.
