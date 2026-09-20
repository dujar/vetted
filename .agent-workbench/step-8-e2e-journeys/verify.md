# Step 8 verify — e2e journeys plan vs repo (2026-09-20, main b889011)

Hard gates: neither trips. Screens: "N/A — tests over the existing screens" (only UI-adjacent change is the api.ts type field; no new surface). Journey: the plan's user-journey section IS the journeys.md matrix (entry, order, unhappy paths all stated). No NO-GO.

Confirmed sound: mock/live env contract exists as described (`api.ts:93-103`, `guard.ts:277-290`, `wallet.ts:81-88`, `registry.ts:108-121`); `abi.ts` exports SELECTORS + GUARD_REVERT_REASONS as literals (abi.ts:31-46); wire.md pins the 46630 scratch RPC; worker route shapes match api.ts (`/scan?chainId=&addr=`, `/watchdog?chainId=` — live per deployments/worker.json); wrangler var names in task 5 match `wrangler.toml:13-15` exactly; worker sets `revocation_tx` from Revoke logs (lib.rs:263-272), so the seeded live J1 REVOKED beat links a real tx; the watchdog degrade sentinel is live-servable today (worker.json check); UI retry seam for RPC_RETRYABLE exists (ScanPage.tsx:256-268); compare deep link route works in both formats (router.ts parseHash fallback); FINGERPRINT F2–F4 matcher vs twin failure modes is consistent (assets.md:59-70 — twins fail F2/F3/F4, replica passes); no file-scope collision with step 7 (contracts/core tests+docs, docs/, gas report, 4663 deploy — disjoint from e2e/** + api.ts); CI has no e2e job today (per-push omission is task 6's deliberate choice; Playwright is named but installed nowhere yet — frontend/package.json has no playwright, no root package.json).

Loose ends:

1. [Missing prerequisite] J2 has no wallet seam in the built frontend — the stub wallet cannot connect
   where:    plan Stack (line 27) "injected EIP-1193 stub wallet"; task 3 swap.spec.ts
   evidence: wagmi config builds `connectors: PROJECT_ID.length > 0 ? [walletConnect(...)] : []` — no `injected()` connector exists (frontend/src/lib/wagmi.ts:20-22); connect throws "no wallet connector configured" (wallet.ts:100-102); SwapPage renders the zero-connector alert with the connect button disabled (SwapPage.tsx:101-107). A window.ethereum shim is unreachable: the deployed bundle only calls `useWalletAdapter()`.
   fix:      plan must name the seam — an e2e-only env flag that adds wagmi's `injected()` connector to getWagmiConfig (a frontend file beyond the declared one-file api.ts exception; scope must admit it) — or J2 browser journeys are impossible in both mock and live modes.

2. [Missing prerequisite] Live scratch-chain J2 contradicts the frontend's hardcoded 4663 — "no page edits either way" doesn't hold
   where:    plan Revised 2026-09-19 note 1 (task 5 runs the same specs with live env); task 3
   evidence: wrong-network gate is `wallet.chainId !== robinhoodChain.id` = 4663 (SwapPage.tsx:113); `switchToVetted` switches to 4663 (wallet.ts:104-106); the live guard-preview and registry clients hardcode the 4663 RPC with VITE_REGISTRY_ADDRESS (guard.ts:283-287, registry.ts:114-117) — pointed at the 46630 pair they read the scratch address on the wrong chain → GUARD_NO_RECORD for every token.
   fix:      pin the mechanism in the plan: Playwright route-proxies the 4663 RPC host to the scratch RPC for the live pass (no frontend edits), or extend the exception with a chain-env override in guard.ts/registry.ts/wallet.ts.

3. [Missing prerequisite] J3 live-scratch cannot run through the registry UI at all
   where:    plan Goal (line 30) "J1/J2/J3 green against … the live step-3/step-6 scratch deployments"; task 4
   evidence: RegistryPage calls `source.rows(4663)` (RegistryPage.tsx:31) on a 4663-pinned client (registry.ts:114-117) and the table is KNOWN_TOKENS-driven (registry.ts:82-104, mockData.ts:95-148) — seeded scratch records never appear; live reads reject → the degraded state renders.
   fix:      scope J3's live portion to mock-mode UI + payload-level assertions (worker `/scan` record, or direct getRecord via the harness) and say so; or extend frontend scope.

4. [Missing prerequisite] baseURL target doesn't have the screens — no task redeploys Pages
   where:    plan task 1 "base URL = deployed Pages"; Goal (line 30)
   evidence: deployments/pages.json is still the step-1 static shell ("step 5 ships the real screens"); step-5 findings: "deployed: not deployed (pages redeploy is step 8/10's job)". No plan task builds+deploys the step-5 bundle, so every spec would run against the stale shell.
   fix:      add the pages build+deploy to task 1 (or pin `vite preview` of the local build as baseURL for the mock pass).

5. [Untouched sibling] The N7 provenanceUrl fold-in breaks an existing unit test outside the one-file exception
   where:    plan Scope (line 33) "no other frontend file is touched"; task 4 / Revised 2026-09-20 note 1
   evidence: folding `provenanceUrl: string | null` (packages/shared/watchdog.ts) into api.ts's WatchdogStats forces MockWatchdogSource's literal (api.ts:82) to include it; frontend/tests/api.test.ts:80-84 asserts `toEqual({chainId, runs: 6092, baselinePerDay: 150})` — vitest toEqual fails on the extra key, so the CI frontend job goes red. Also task 4 says "watchdog rows" in registry.spec.ts, but the registry page has no watchdog (WatchdogWidget renders only in ScanPage.tsx:212).
   fix:      name frontend/tests/api.test.ts in the exception (one-line expectation update), and state the sentinel assertion is payload-level (worker /watchdog, servable today) since the widget cannot render the field without more frontend edits.

6. [Dangling reference] J1 degraded banner "(intercept /rhj/assets)" — the browser never requests that URL
   where:    plan User journey (line 37) "degraded banner (intercept /rhj/assets)"
   evidence: the canonical fetch happens server-side in the worker (scan-backend/src/lib.rs:75-89, rules.rs:23); the frontend's only fetches are /scan and /watchdog (api.ts:49,72); mockScanResponse has no degraded:true path (mockData.ts:161-229).
   fix:      intercept the worker's /scan response and fulfill a degraded:true ScanResponse body; for J3's degraded state, intercept the JSON-RPC POSTs or build with a bogus VITE_REGISTRY_ADDRESS.

7. [Missing prerequisite] Scratch chains can never emit IMPOSTOR — the live compare spec must not assert the red card
   where:    plan User journey (line 37) "compare deep link — deployed impostor twin vs canonical replica"
   evidence: canonical evidence is fetched only when `chain_id == CHAIN_MAINNET` (scan-backend/src/lib.rs:237-247) and IMPOSTOR requires `inputs.canonical` (rules.rs:293-306); on 46630 the twins scan UNVERIFIED + advisory heuristics.
   fix:      one sentence in task 2: IMPOSTOR-red is asserted in mock mode (fixtures verbatim); live-scratch asserts replica VERIFIED (with the non-4663 notice the worker attaches, rules.rs:151-153) vs twin UNVERIFIED.

8. [Missing prerequisite] Task 5's seed list omits the J2 settle's token plumbing
   where:    plan task 5 ("canonical replica verified, one pre-revoked record, twins unregistered")
   evidence: guard `commit` escrows tokenIn via transferFrom buyer→guard (contracts/core/guard/src/lib.rs:8-9, 258) and `execute` pulls the MM's standing tokenOut allowance (lib.rs:10-13) — with only registry records seeded, the settle beat reverts for lack of buyer balance/allowance and MM balance/allowance.
   fix:      extend task 5's seeding (or the harness pre-flight) to fund buyer + MM from the deployed mock tokens and set both allowances to the guard.

9. [Deferred decision] Task 6's "nightly-or-manual workflow" vs a scope line that admits no workflow file
   where:    plan task 6 vs Scope (line 33) "Creates: e2e/**, plus ONE deliberate exception"
   evidence: a workflow lands at .github/workflows/ — outside e2e/**, and ci.yml is step-1-owned; task 6 doesn't say whether this step writes it or only runs locally.
   fix:      one clause: either task 6 is local/on-demand only for this step, or the scope line names the new workflow file.

10. [Dangling reference] Two small premise slips around the unfunded scratch reality
    where:    plan Stack (line 27); task 1 fixture loader
    evidence: the fallback trigger "if step-2 findings say 46630 gas never materialized" appears nowhere in spike/findings.md (it records all faucets headless-blocked, key 0 wei on all three chains — spike/findings.md:18; step-3 findings:6); and task 1's loader reads `deployments/*.json` + `demo/assets.md`, both of which carry PENDING/no files today (deployments/ has only pages.json, worker.json, README.md; assets.md tables are "PENDING DEPLOY") — Revised note 3 says "mock-mode specs run regardless", but a loader that hard-requires the JSONs crashes exactly then.
    fix:      cite the real condition (operator key unfunded per step-3 findings:6) and make the loader tolerate missing files / PENDING placeholders by skipping scratch-gated specs.

Smallest viable plan patch: fix 4 (build/serve the real bundle), fix 1 + 2 (name the wallet seam and the chain mechanism, admitting frontend-scope edits if that's the choice), then 5–10 are one-liners. Until 1–4 are answered the task list as written cannot produce green journeys.
