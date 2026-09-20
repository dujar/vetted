# Step 10 — production deploy, final smoke, submission

status:     ready-to-merge (awaiting review — this invocation has no spawn tool; the caller dispatches review-agent on the branch and re-invokes)
branch:     step-10-deploy-submission (off main 16baada)
reconciled:
deployed:   not deployed — UNFUNDED regime at step start AND end (operator key 0x151e…8E4C reads 0 wei on 4663/421614/46630; re-checked at step end). Two-regime honored: the full unfunded package is built; the funded-run completion is staged as `submission/post-funding-checklist.md`. Funding go/no-go: 2026-09-29 (verify LE 1).

## What was built

The unfunded-regime submission package, complete except chain-state items:

- **README security note (the required package item, plan note 5 + verify LE 2):** prominent "Security note — read before trusting a swap" section disclosing the guard `erc20()` junk-returndata decode (`contracts/core/guard/src/lib.rs:497`) with the precise funds-safe worst case (phantom active order → `execute()` reverts whole; no theft path), the freeze-not-hot-fix rationale, and the "known limits" list matching the live URLs (depth boundary; watchdog degrade sentinel by design; PASS-PENDING-FUNDS gas; drill-in tx lag; the PENDING-FUNDS clause for deploys/VITE_*/P-receipt, regime-aware per LE 1).
- **Name lock landed (note 6 item 2):** README title is `# Vetted`; `brand.ts` comment states the lock; the stale recording-waits gates in `demo/runbook.md` §2 and `demo/video-script.md` (header + pre-flight) marked resolved — they would have stalled the post-funding operator indefinitely.
- **`submission/` package:** `README.md` (package index: contents table, live URLs verified 2026-09-21, contract-addresses PENDING-FUNDS section, repo-visibility decision, freeze/tag rules, operator upload checklist); `qa-prep.md` (judged-criteria mapping + 10 rehearsed answers: Q1 the erc20() carry, Q2 fresh-redeployment lag spec.md:79, Q3 twins-on-production spec.md:80, Q4 why-not-EAS with the 421614 addresses, Q5 why-not-GoPlus with the spike evidence, Q6 Blockaid, Q7 Token Sniffer/De.Fi no-specifics-until-spot-checked rule, Q8 registrar blast radius, Q9 watchdog figures, Q10 gas proof); `post-funding-checklist.md` (the executable funded-window run: go-list §1 mapping, audit re-run, timed smoke, P-receipt routing, final Pages deploy with the live env, gif + README:80 uncomment, tag, upload); `production-state-audit-2026-09-21.md` (committed audit record, unfunded state).
- **`scripts/release/` (deploy orchestration):** `production-check.sh` — the task-2 audit as a runnable, unfunded-aware instrument (PASS/WARN/FAIL/PENDING-FUNDS/N-A lines; distinguishes the designed `RPC_RETRYABLE` terminal and `record:null` degrade from real failures; fabricate-nothing checks on VERIFIED scans); `cut-release-tag.sh` — the task-1/LE-4 instrument (refuses dirty tree / non-main / existing tag; CI-green gate via `gh` or `--local-suite`; clean-checkout build check; annotated freeze note; prints the v1.0.1 post-tag rule).
- **Plan addendum** (workbench, in-branch): verify LE 1–5 dispositions recorded in plan.md as a Revised block.

## Where the plan was wrong

- LE 3 named only README.md:1 and brand.ts:2, but the same stale open-question framing gates recording in `demo/runbook.md` §2 and `demo/video-script.md` — left alone, the post-funding operator would wait on a lock that note 6 item 2 already resolved. Fixed both (one line each); counted as applying note 6, not scope creep.
- Nothing else structural. Script-level: the audit's first draft classified the DESIGNED `RPC_RETRYABLE` terminal (which arrives as `verdict:null` + `terminalState`) as a FAIL — fixed in-script before the record was captured; the committed audit output is from the fixed classifier.

## What the next step needs to know

- **Tag mechanics (LE 4, binding):** `v1.0.0` is cut at the END of packaging on the SUBMITTED tree — operationally, on this step's merge commit on main, by the CALLER, via `scripts/release/cut-release-tag.sh` (it refuses to run off-main; CI is read off the main push of the tagged commit; clean-checkout build is built in). Post-tag blocking fix → `v1.0.1`, never a re-cut. Do not tag the branch head pre-merge.
- **Funding path:** `submission/post-funding-checklist.md` steps 0–9 execute everything staged (go-list → audit → timed smoke → P receipt → Pages live-env deploy → gif/README:80 → tag → upload). Go/no-go 2026-09-29; after that, coordinator escalation or the declared unfunded variant ships (defined in `submission/README.md`).
- **Repo visibility (LE 5, coordinator decision 2026-09-21):** repo stays PUBLIC with the `.agent-workbench/` narrative as evidence — no scrub. Contingency recorded: if it flips, COPY the P receipt into `submission/` instead of linking the workbench path.
- **CI:** green on main at 16baada (run 35538448485; the following `completed(cancelled)` run is the concurrency group superseding it, not a failure). The three guard-test dead-code helpers (`lib.rs` `BEACON`:655, `mock_record`:706, `install_beacon_token`:733) are dispositioned follow-up — warnings, not failures, named in the tag's freeze note; no contract changes at tag time.
- **Worker:** unchanged (secrets deliberately unset until the registrar handoff; `REGISTRY_ADDRESS_4663` empty until go-list step 6 — the audit's PENDING-FUNDS lines evidence both live).

## Out of scope, left broken

- `demo/demo.gif` does not exist unfunded (no take to cut from); the README link stays commented on purpose (links must resolve) — checklist step 6 uncomments it.
- Token Sniffer / De.Fi browser spot-checks (spec.md:71) need a real browser — no browser tool in this context; staged as checklist step 0 "anytime" with the no-specifics rule (qa-prep Q7).
- Worker secrets audit carried as WARN in the committed record: this worktree has no scan-backend/node_modules, so `wrangler secret list` was skipped — hand-verify per the audit's WARN line during the go-list.
- Watchdog live-counter mechanism unpinned (coordinator-carried since step 4) — disclosed as the degrade sentinel by design; Q9 carries the narration.
- Guard `erc20()` junk-decode: coordinator-carried to Q&A, disclosed (README + Q1), fix queued post-freeze — deliberately NOT touched at freeze.
