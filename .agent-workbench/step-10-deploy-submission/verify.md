# Verify — step-10-deploy-submission plan (2026-09-21)

Repo: main fc8b004, clean. All plan resources exist and were read in full: product/spec.md (line refs 69/71/73–80 exact), all five `knows:` knowledge files, all seven `learned:` findings files. Every cited artefact resolves: demo/{runbook,video-script,assets}.md + arc.sh + serve.sh, README final draft, docs/{criteria,threats,gas-report}.md, step-9 task-6 go-list, step-7 funded-run runbook, scan-backend/scripts/live-check.sh, scripts/deploy/{core,replicas}/deploy.sh, scripts/seed-demo/seed.sh, spike/DEPLOY.md + evidence (calibration_4663.json, goplus_token_security_4663_P.json). Line-cited anchors verified exact: scan-backend/wrangler.toml:13 (`REGISTRY_ADDRESS_4663`), guard lib `BEACON`:655 / `mock_record`:706 / `install_beacon_token`:733 / `erc20()`:497, e2e/README.md:41 (scratch activation), knowledge/contract-verification.md:19–27 + :27–38, gas-report.md:74 (§4 PASS-PENDING-FUNDS) + :96 (declared unfunded-at-close ship state). Cited commits 5e2ff35/9d8dc30/e37bc15/b17fdd3/8648efc all exist. CI = six jobs (contracts, replicas, mock-token, scan-backend, shared, frontend), triggers push-main + PR. No git tags exist yet (v1.0.0 is new). deployments/ holds only pages.json, worker.json, README.md — no 4663.json/421614.json, confirming the PENDING state the plan records.

Hard gates: neither trips — plan declares Screens N/A and User journey N/A (operational close-out, plan.md:36–40); no new UI surface is built.

Deadline math: realistic. Today 09-21 → 12 days for 6 tasks; freeze target 2026-10-03 EOD SGT vs close 2026-10-04 23:59 SGT meets the ≥24h buffer iff EOD is read as late evening — the only calendar-critical unknown is the funding gate (loose end 1).

The erc20() carry is handled as the dispatch requires: Revised note 5 records the coordinator decision + funds-safe worst case, task 5 makes the disclosure a REQUIRED package item, task 6 names it in the Q&A list, and note 4 freezes product code against freeze-time tightening. Dead-helper warnings are dispositioned (note 4 + task 1). Verified-status re-check vs re-run split (note 3) matches the knowledge file's recipes.

## Loose ends

1. **[missing prerequisite] No zero-funding branch — tasks 2/3/5 silently assume the funded run executed.**
   where:    plan tasks 2/3/5 (plan.md:44–47); demo/runbook.md §0; docs/gas-report.md:96
   evidence: runbook §0 gates the full live pass on the step-7 funded-run runbook having executed; at reconciliation deployments/4663.json and 421614.json are absent (ls) and the operator key reads 0 wei (step-9 findings). If funding never lands, task 2 has nothing to audit, task 3 has no production URLs to smoke, task 5's "contract addresses" is empty — and the hackathon's hard requirement "must deploy on an Arbitrum chain" (spec.md:69) fails, taking the Robinhood-reserved-prize eligibility with it. gas-report.md:96 declares the unfunded-at-close end-state for the REPORT, but the plan never carries that branch for the submission itself.
   fix:      one paragraph in the plan: a go/no-go date for funding (e.g. 2026-09-29, preserving the 3-day smoke/record/package window) and, if it passes unfunded, what ships — the declared unfunded variant (anvil-rehearsal recording per runbook §0, PASS-PENDING-FUNDS honesty copy, no contract-address section) or an explicit escalation.

2. **[deferred decision] The security note is scoped to erc20() only; the known-deferred state a judge will hit is never enumerated.**
   where:    plan task 5 (plan.md:47) + task 6 (plan.md:48)
   evidence: task 5 mandates the PROMINENT erc20() disclosure and task 6 names exactly three Q&A risks (spec.md:79, :80, erc20()). The README/submission is nowhere required to disclose the remaining known-deferred items visible at the live URLs: watchdog live counter unpinned → ships the runs:0 + provenanceUrl degrade sentinel (step-4 findings, coordinator flag), the VERIFIED-shaped P receipt possibly still missing at freeze (step-9 findings out-of-scope; task 2 note 2 and go-list step 7 both carry it as pending), and the registry drill-in revocationTx "not indexed yet" (step-8 findings). The 4663.json/VITE_* deferrals are regime-conditional — they close if the funded run executes (see loose end 1).
   fix:      extend task 5's README item one sentence: the security note also carries a "known limits" list matching what the live URLs actually show (watchdog degrade sentinel by design, P-receipt status at freeze, drill-in tx lag), regime-aware per loose end 1.

3. **[unhandled call site] Step-9's handoff items to step 10 are absent from the plan.**
   where:    plan task 5 (plan.md:47)
   evidence: step-9 findings assign two finish-at-step-10 actions the plan never names: (a) cut `demo/demo.gif` from the best take and uncomment README.md:80 (`<!-- ![demo arc](demo/demo.gif) -->` — go-list step 8 + out-of-scope note; video-script.md preface says the gif comes from beats 4–6 of the best take); (b) clear the stale placeholder-name framing now that the name is LOCKED (tracker commit fc8b004): README.md:1 still reads "vetted_ (placeholder name — locked by the coordinator before demo recording)" and frontend/src/lib/brand.ts:2 still calls the (already correct) constant a placeholder.
   fix:      append both to task 5: render demo/demo.gif from the final take then uncomment README.md:80; drop the placeholder-name framing in README.md:1 and brand.ts:2.

4. **[deferred decision] Release-tag mechanics underspecified: tag timing, CI trigger, post-tag fix rule.**
   where:    plan task 1 (plan.md:43) vs task 5 (plan.md:47); scope plan.md:34
   evidence: task 1 tags v1.0.0 before task 5 finalizes the README, so the tagged tree cannot contain the required security disclosure — "the tagged release builds from clean checkout" (plan.md:50) then verifies a tree that is not the submitted one. .github/workflows/ci.yml:4–5 triggers are push-to-main + PRs only (no `tags:` filter), so "CI green at the tag" resolves only as "CI green on the main commit the tag points at". And "product code frozen except blocking fixes" gives no rule for a blocking fix landing after the tag: re-cut v1.0.0 or bump v1.0.1.
   fix:      state it in task 1: either cut the tag at the END of task 5 (tagged tree = submitted tree) or declare tag = freeze commit plus a re-cut-on-blocking-fix rule, and note CI is read off the main push of the tagged commit.

5. **[deferred decision] The public repo's content is never decided — the internal workbench narrative ships to judges.**
   where:    plan task 5 "repo link" (plan.md:47); open questions plan.md:52–53
   evidence: `git ls-files` shows 66 tracked `.agent-workbench/**` files on main of the public repo (github.com/dujar/vetted, public since step 1), including verify verdicts, review rounds ("closing judge round-4 gap 1"), Q&A-ammo planning, and coordinator notes. No step ever decided whether judges should see the internal build narrative; it is part of what the "repo link" artefact serves, and any scrub must happen BEFORE task 1's freeze to be meaningful.
   fix:      record the coordinator decision in the plan before freeze: accept the process narrative as public evidence (no action), or add a pre-freeze task relocating/privatizing `.agent-workbench/**`.

## Clean checks (no finding)

- Two-secret + vars audit (note 1/2) matches wrangler.toml:13 and the step-7 runbook's `--var` discipline; `e2e/README.md:41` activation gating is real.
- CI green as submission gate: present (task 1) and satisfiable; e2e is deliberately local-only (step-8 LE 9), so the journey gate is task 3's timed pass.
- Hackathon requirements coverage: 4663 primary deploy + verify (task 2, funding-gated — loose end 1), repo public (task 5, content question — loose end 5), video demo (task 5 + video-script.md), criteria/threats/gas docs all on disk and README-linked.
- Knowledge files agree with the plan everywhere they overlap (verification recipes, EAS addresses for the Q&A rehearsal, GoPlus copy branch, 4663 RPC/bridge facts).
