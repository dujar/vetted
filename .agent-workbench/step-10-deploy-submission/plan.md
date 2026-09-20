# Step 10 — Production deploy, final smoke, submission (buffered before 2026-10-04 23:59 SGT)

## Resources
- spec:    ../product/spec.md   (pre-submission checklist spec.md:71; constraints spec.md:69; risks spec.md:73–80)
- knows:   ../knowledge/robinhood-chain.md, ../knowledge/arbitrum-sepolia.md, ../knowledge/goplus-api.md, ../knowledge/eas.md, ../knowledge/contract-verification.md  (rehearsed Q&A ammo; verification re-check recipes :12–38)
- learned: ../step-7-contract-hardening/findings.md, ../step-9-demo-harness/findings.md
- learned: ../step-4-scan-backend/findings.md  (worker LIVE @ 0ccf8c68 — REGISTRAR_KEY/DRIFT_ADMIN_SECRET unset until step 7's handoff; 4663 public RPC throttles Cloudflare egress, honest UNVERIFIED degrade verified live)
- learned: ../step-1-repo-scaffold/findings.md  (scaffold MERGED at 5e2ff35 — deploy pipeline proven, env audit facts)
- learned: ../../spike/findings.md  (step-2 findings — MERGED at 9d8dc30; file lives in the REPO at `spike/findings.md`: canonical-fetch worker LIVE as fallback mirror) + ../step-5-frontend-screens/findings.md  (MERGED at e37bc15 — the frontend env contract this step's audit must set)
- learned: ../step-8-e2e-journeys/findings.md  (MERGED at b17fdd3 — 28 e2e green + 3 activation-gated skips with the `e2e/README.md` activation runbook; Pages bundle redeployed 2026-09-21 with the sanctioned wiring fixes + the render-loop fix)

> **Revised** — step-1 findings reconciliation (2026-09-11): task 2's env audit needs no deploy-credential work — the wrangler OAuth token already has workers+pages scopes (hello worker + Pages live since step 1: `vetted-scan-backend.dujar-coding.workers.dev`, `vetted-1un.pages.dev`, alias in `deployments/pages.json`). The **only outstanding operator action in the whole project is the WalletConnect Cloud projectId** (`VITE_WALLETCONNECT_PROJECT_ID`) — without it the frontend boots read-only with zero connectors by design (not a bug). Check task's URL resolution: per-deployment URL probes intermittently failed TLS from the dev machine only — treat a local TLS failure as suspect and re-verify from off-machine before flagging a URL dead.

> **Revised** — step-2 + step-5 findings reconciliation (2026-09-19): task 2's env audit now has a concrete frontend list from step 5's contract — `VITE_API_MODE=live` + `VITE_API_URL` (→ step-4 worker), `VITE_REGISTRY_ADDRESS`/`VITE_GUARD_ADDRESS` (→ `deployments/4663.json`, written by step 7), `VITE_WALLETCONNECT_PROJECT_ID`. The spike's canonical-fetch worker (`vetted-spike-canonical-fetch.dujar-coding.workers.dev`, live 2026-09-19, 194 assets) is a FALLBACK mirror — production `/scan` absorbs the fetch (step 4); smoke should confirm the absorbed path, not the spike URL.

> **Revised** — step-4 findings reconciliation (2026-09-20):
> 1. **Secrets audit is a TWO-secret worker check:** step 4's live worker ships with `REGISTRAR_KEY` and `DRIFT_ADMIN_SECRET` DELIBERATELY UNSET (drift-check degrades with a clean skipped report until set). Task 2's audit verifies both were set at step 7 AND `REGISTRAR_KEY`'s address equals the deployed 4663 registry's registrar — alongside the frontend env list in the 2026-09-19 note.
> 2. **Paid/private RPC decision (demo window):** the public 4663 RPC rate-limits Cloudflare's shared egress (live-verified 2026-09-20; step 4's VERIFIED P receipt still pending a calm window; step 9 carries the retry fallback). Decide BEFORE the production smoke: if throttling persists, provision the paid/private endpoint (worker env swap), re-smoke `/scan` on 4663, and capture the VERIFIED-shaped P receipt as a submission asset.

> **Revised** — step-7 + step-8 findings reconciliation (2026-09-21):
> 1. **deployments/ writer-of-record:** the funded-run runbook (`../step-7-contract-hardening/findings.md`) writes BOTH files — the first funded run anywhere is end-to-end `deploy.sh` on 421614 (writes `421614.json`; step 3's funded run never happened), and stage F creates-or-shallow-merges `4663.json` append-only over step 6's replicas field. Both were still absent from `deployments/` at reconciliation (2026-09-21) — PENDING until the runbook executes at/before step 9. Task 2's audit reads `VITE_REGISTRY_ADDRESS`/`VITE_GUARD_ADDRESS` from `4663.json` and confirms `docs/gas-report.md` §4 flipped its PASS-PENDING-FUNDS header (runbook step 7 folds the 11 receipt actuals). If only one chain got funded, 4663 took priority (runbook rule) — the 421614-mirror check is then honestly N/A: record it as such, don't force it.
> 2. **The vars audit extends the secrets audit:** `REGISTRY_ADDRESS_4663` is a `[vars]` entry (`scan-backend/wrangler.toml:13`) that the runbook passes via `wrangler deploy --var` — never a `wrangler.toml` edit + redeploy from a stale checkout (it silently blanks the live var; step 9 re-passes it too when it adds `DRIFT_EXTRA_TOKENS`). The scratch pair `REGISTRY_ADDRESS_46630/421614` fills only if the scratch activation (`e2e/README.md:41` runbook) ran. Audit that what IS set points at the deployed registry — not that all three are set.
> 3. **Verification was executed by the runbook** (`cargo stylus verify` both chains, with the Cloudflare fallbacks: browser-UI Blockscout / sourcify on 4663, Etherscan v2 `--chain-id 421614` on the mirror) — task 2 re-checks the resulting explorer pages resolve; it does not re-run verification.
> 4. **Known warnings at freeze (task 1):** three dead test helpers in the guard test target (`BEACON`, `mock_record` at `contracts/core/guard/src/lib.rs:706`, `install_beacon_token` :733 — pre-existing on main) print dead-code warnings under `cargo test -p guard`; the step-7 review dispositioned them as follow-up cleanup, deliberately NOT folded into the reviewed diff. Warnings are not failures — do not block the tag on them and do not "clean up" contracts at freeze (product code frozen except blocking fixes).

## Stack
As built; no new dependencies.

## Goal
After this step the project is submitted with ≥24h of buffer: everything deployed and source-verified at production URLs, the demo arc smoke-green on those URLs, the spec's pre-submission checklist done, and the submission package uploaded.

## Scope
- Touches: deploy orchestration scripts, `submission/**`, release tag. Product code frozen except blocking fixes.

## User journey
N/A — operational close-out.

## Screens
N/A.

## Tasks
1. Freeze + tag `v1.0.0`; CI green at the tag; clean-checkout build verified. Known guard-test dead-code warnings are dispositioned follow-up, not a freeze blocker (Revised 2026-09-21 note 4).
2. Production state check: 4663 core + replicas deployed and source-verified (step 7 / step 6 artifacts; re-check verify status per knowledge/contract-verification.md — 4663 = Blockscout with the CF-challenge browser-UI fallback :19–27, 421614 = Etherscan v2 with `chainid=421614`, one key, v1 endpoints dead :27–38), worker + Pages at production URLs, secrets/env audit (REGISTRAR_KEY — its address must equal the deployed registry's registrar, pinned by step 7's deploy task; DRIFT_ADMIN_SECRET set alongside it — both ship deliberately unset until step 7's handoff, Revised 2026-09-20 note 1; WalletConnect projectId; chain RPCs), 421614 mirror consistent, and the demo-window RPC decision — paid/private 4663 endpoint if the public RPC still throttles Cloudflare egress (Revised 2026-09-20 note 2). Writer-of-record / vars-audit / verify-status refinements: Revised 2026-09-21 notes 1–3.
3. Full demo-arc smoke against production URLs — step 9's runbook, one pass, timed.
4. Pre-submission checklist (spec.md:71): browser spot-check Token Sniffer + De.Fi on a genuine 4663 token (their capabilities were unverified at research time — adjust Q&A if either catches the beacon pattern); rehearse the EAS answer with the knowledge/eas.md addresses and the GoPlus answer with the spike screenshot (enforcement / revocation / power-disclosure copy branch).
5. Submission package: final video, README, repo link, live URLs, contract addresses — uploaded targeting **2026-10-03 EOD SGT**, ≥24h ahead of the close (spec.md:69).
6. Post-submit: archive findings, note residual risks for judges' Q&A (fresh-redeployment-reads-IMPOSTOR lag = spec.md:79, twins-on-production mitigation = spec.md:80).

Check: submission confirmation recorded; all public URLs resolve (Pages, Worker, 4663 explorer, mirror explorer); the tagged release builds from clean checkout.

## Open questions
- Submission-portal mechanics (form fields, video hosting) — operator handles from the checklist; no plan dependency.
