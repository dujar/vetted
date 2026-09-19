# Step 9 — Demo harness: the scripted 5-minute arc + submission master assets

## Resources
- spec:    ../product/spec.md   (the arc beat-by-beat with timings spec.md:39; degraded branch spec.md:55; demo network default spec.md:59)
- journeys: ../product/journeys.md  (arc ↔ journey mapping, journeys.md:47)
- knows:   ../knowledge/robinhood-chain.md (RPC/explorer URLs :14–21), ../knowledge/goplus-api.md (copy branch :26–27)
- learned: ../step-3-contracts-core/findings.md, ../step-4-scan-backend/findings.md, ../step-6-replica-assets/findings.md, ../step-8-e2e-journeys/findings.md
- learned: ../step-1-repo-scaffold/findings.md  (scaffold MERGED at 5e2ff35 — live-verified RPC/URL table, production worker+pages already deployed)
- learned: ../../spike/findings.md  (step-2 findings — MERGED at 9d8dc30; file lives in the REPO at `spike/findings.md`: canonical-fetch worker LIVE, calibrated probe targets) + ../step-5-frontend-screens/findings.md  (MERGED at e37bc15 — mock/live env contract for the screens)

> **Revised** — step-1 findings reconciliation (2026-09-11): the runbook's endpoints and beat URLs come from the live-verified table in `packages/shared/wire.md` (chains) and `deployments/{worker,pages}.json` — the production worker (`vetted-scan-backend.dujar-coding.workers.dev`) and Pages (`vetted-1un.pages.dev`) have been live since step 1. Caveat carried in findings: per-deployment URL probes intermittently failed TLS from the dev machine only — if a beat URL "fails" during a dry run, re-check from off-machine before rewriting the runbook fallback.

> **Revised** — step-2 + step-5 findings reconciliation (2026-09-19): the power-report beat's blocklist evidence is real only via step 4's server-side BEACON probe (the calibrated blocklist state lives on the beacon; the frontend's live preview row degrades to "probe unavailable" on genuine-pattern tokens — known step-5 defect, see step 8's Revised note 2). Narrate the power report from the `/scan` verdict payload, not the preview rows. The live-scan beat's canonical ground truth is backed by the live canonical-fetch worker (194 assets, re-verified 2026-09-19) absorbed into step 4's worker.

## Stack
Bash + cast + OBS recording; no new runtime dependencies.

## Goal
After this step the 5-minute arc runs start-to-finish on 4663 from a fresh browser with a runbook in hand: every URL/address/keystroke pre-staged, the upgrade→auto-revoke→guard-refusal beat reliable via the manual drift-check endpoint, and the video script + repo README ready for submission.

## Scope
- Creates: `demo/**` (runbook, beat scripts, video script, recording), `scripts/seed-demo/` (idempotent registrar seeding).
- Out of scope: product fixes (go to owning steps); submission-form mechanics (step 10).

## User journey
The demo arc IS the scripted journey (spec.md:39, mapping journeys.md:47): docs-page problem (30s) → live read-only scan of a real 4663 address we did NOT deploy (30s) → impostor twin vs canonical compare deep link (30s) → power report on the genuine replica: hidden blocklist, global pause, beacon-upgradeable (45s) → guarded swap: impostor reverts with the reason surfaced, genuine settles (45s) → **the beat no incumbent has:** beacon-upgrade the canonical replica → registry auto-revokes → the guard refuses the formerly verified token (60s) → watchdog widget + registry-as-primitive + roadmap close (30s).

## Screens
N/A — orchestration over existing screens.

## Tasks
1. `scripts/seed-demo/`: registrar-signed, idempotent seeding of 4663 demo state — verify(canonical replica); revoke(one record, for the REVOKED states); twins left unregistered; uses the same registrar code path as step 4.
2. Beat runbook (`demo/runbook.md` + `demo/arc.sh`): per beat — URL, wallet/account, expected screen state, timing budget, and the fallback (what to say and click if fetch/RPC/wallet misbehaves: the degraded branch is pre-written copy, spec.md:55; the live-scan beat carries a backup address list).
3. Make the upgrade beat reliable: script the beacon-upgrade tx + the manual `POST /registry/drift-check` call + the guard refusal; measure wall-clock. If cron latency would eat the 60s beat, the runbook uses the manual endpoint and the narration says the cron runs the same code on schedule — honest, not staged.
4. Video script (`demo/video-script.md`) mapped 1:1 to beats, narration hitting the judged criteria (enforcement, revocation, power disclosure — never "trust layer", never "first coverage": spec.md:4, knowledge goplus-api.md:26–27); record the full take + per-beat clips.
5. README final draft: architecture diagram, quickstart, `docs/criteria.md` link, demo gif, demo-cast labeling (spec.md:80) — repo hygiene feeding the contract-quality and real-problem-solving criteria.
6. Full dry run on a fresh browser profile, timed; actuals vs budget logged in the runbook. This dry run is the **first full journey pass on live 4663** — step 8's e2e suite greened against the step-3/6 scratch deployments (46630/421614); treat any 4663-only divergence (RPC behavior, explorer links, verdict on the real cast) as a blocking fix before recording.

Check: the arc completes ≤5:00 twice in a row from the runbook; video recorded; README renders with resolving links.

## Open questions
- The final product name must be locked BEFORE recording (state.md:40 defers it to the coordinator at screens phase) — flag at step start; recording waits on it.
