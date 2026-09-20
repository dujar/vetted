# Reconciliation 2026-09-21 — step-9-demo-harness findings

Source: `../step-9-demo-harness/findings.md` (status: merged, review round 2 APPROVED;
`reconciled:` empty at round start — the only unstamped findings file).
Only step 10 (`step-10-deploy-submission`, planned) is not yet `done`; steps 1–9 are done,
so step 9's plan is a record and was not touched. Step 10's Resources block already carried
`learned: ../step-9-demo-harness/findings.md`.

## Folded in

1. **Task-6 go-list is the production deploy path (staged, unfunded regime).** Step 9 merged
   with tasks 1–5 built and task-6 dry runs staged behind the go-list (findings items 1–8,
   entry = step-7 funded-run runbook, ordering: replicas BEFORE core stage F, ONE worker
   reconfig re-passing all non-empty vars). → New step-10 Revised note 6.1: if
   `deployments/4663.json` is still absent at step-10 start, run the go-list first; task 2
   verifies rather than builds. Step-10 Revised note 1 (2026-09-21, steps-7-and-8 round)
   already said the deploy files were PENDING "until the runbook executes at/before step 9" —
   step 9 merged without executing it, so the note now points at the concrete go-list
   instead of an implied event.
2. **Product name lock RESOLVED.** Coordinator locked "Vetted" (standing delegation; tracker
   commit fc8b004 on main — the commit also set step 9's tracker row to done). Step-9
   findings listed "recording waits on the product name" as an open question; that is closed.
   → Step-10 Revised note 6.2: recording and the task-5 package are no longer name-blocked;
   `frontend/src/lib/brand.ts` is final; task 5 uncomments the `demo.gif` img in README once
   the take exists (step 9 left it commented deliberately).
3. **VERIFIED-shaped P receipt routing.** Still missing at merge (rate-limit windows). Captured
   during step-9 task-6 between-takes (`live-check.sh`, spaced attempts, go-list step 7) and
   recorded in STEP-9's findings when it lands; step 10's package links it from there.
   → Step-10 Revised note 6.3 refines the 2026-09-20 note-2 capture clause accordingly;
   task 3's smoke chases it only if still pending at step-10 start.
4. **Smoke re-runs are not re-seeds.** Registry transitions are monotone (REVOKED cannot
   re-verify); recovery between passes is runbook §6 (re-deploy the replica cast). → Step-10
   Revised note 6.4 for task 3.

## Not folded into plans (no plan assumption touched)

- `demo/demo.gif` absent until the final take — already carried by findings + note 6.2.
- Honest-twin framing, pause-choreography fix, and the FRESH_BUYER blocklist-probe address are
  folded into `demo/runbook.md` itself; task 3 runs "step 9's runbook" and inherits them.
- Stale watchdog figures in the frontend mock (6092/150) stay by design; narration never quotes
  them — no step-10 task touches the mock.
- `X-Admin-Secret` / cron-equivalence facts: runbook + arc.sh carry them; step 10 references
  the runbook.
- **Worktree `vetted-step9-wt` (at 01f43ce, pre-merge) is prunable** — content merged via
  25a1bf3/7841eaf. Coordinator housekeeping; no plan mentions worktrees.
- Untracked `.agent-workbench/step-10-deploy-submission/verify.md` (the step-10 verify pass
  output) predates this round and is not part of this commit.

## Judge

No spawn tool in this context — plan-judge-agent must be dispatched by the caller (one round,
fed this record + the step-10 diff). **Stamp withheld:** `reconciled:` on step-9 findings stays
empty until the judge rules, per the standing pattern (step-7's stamp was also applied
post-verdict).
