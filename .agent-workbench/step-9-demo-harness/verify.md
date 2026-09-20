# Step 9 verify — demo harness plan (plan.md read in full, incl. all five Revised notes)

Checked against main 8648efc (tracker commit on 0d0c872, tree clean). All plan-cited artefacts exist and match the code: `demo/assets.md` beat choreography (:60-82), `contracts/replicas/FINGERPRINT.md`, `e2e/scripts/seed-scratch.sh` (bash -n clean), `scan-backend/scripts/live-check.sh` (BASE_URL env as the plan invokes it), `packages/shared/wire.md:89` 4663 RPC, `deployments/{worker,pages}.json` pins, `docs/criteria.md`, `state.md:40` product-name deferral, knowledge `robinhood-chain.md` RPC/explorer URLs and `goplus-api.md:26-27` copy branch, `frontend/tests/registry.test.tsx:76` empty-state pin, `e2e/README.md:35-58` activation runbook. VITE_ names as coded: `VITE_API_MODE`, `VITE_API_URL`, `VITE_CHAIN_ID`, `VITE_REGISTRY_ADDRESS`, `VITE_GUARD_ADDRESS`, `VITE_REGISTRY_TOKENS`, `VITE_WALLETCONNECT_PROJECT_ID`, `VITE_E2E_STUB_WALLET`. Worker route `POST /registry/drift-check` (routes.rs:19), `RPC_RETRYABLE` (rpc.rs:18), `DRIFT_EXTRA_TOKENS`/`DRIFT_ADMIN_SECRET` (lib.rs:111-112), wrangler.toml vars+cron all present; mock watchdog 6092/150 confirmed still in `frontend/src/lib/mockData.ts:70`; measured 258,707/~2,630-per-day in `scan-backend/src/watchdog.rs:8-10`. Two-regime dispatch is real: tasks 1-5 write files only; task 6 correctly gated on the step-7 runbook (operator key 0 wei per step-7 findings, reconfirmed). Step-10 plan scope (`submission/**`, deploy orchestration, audit-only task 2, smoke task 3 consuming step 9's runbook) does not collide with `demo/**` or `scripts/seed-demo/`; no findings-file contradictions found.

No hard gate trips: Screens are N/A (orchestration over existing screens), and the plan's User-journey section is the full scripted arc with unhappy paths.

## Loose ends

```
[missing prerequisite] No demo frontend serving mode: the live beats have no env-set bundle to run in
  where:    plan task 2 (runbook "per beat — URL"), task 4 (record), task 6 (fresh-browser dry run); Revised note 1
  evidence: production Pages (vetted-1un.pages.dev, deployments/pages.json) is the product-default MOCK bundle ("built WITHOUT ... env, product default unchanged"); in live mode the swap/registry sources throw without addresses (frontend/src/lib/wallet.ts:85, guard.ts:282, registry.ts:133) and scan beats hit mock fixtures; VITE_REGISTRY_ADDRESS/VITE_GUARD_ADDRESS don't exist until deployments/4663.json (runbook step 4); step-10 task 2 only AUDITS env — nobody sets Pages env before step 9's recording/dry run
  fix:      task 2's runbook names the demo serving mode — an env-baked local serve using the e2e live-server pattern (bakes VITE_* from deployments/4663.json + VITE_API_MODE=live/VITE_API_URL), or an explicit Pages env pass + redeploy inserted into step 9's funded sequence with step 10's audit as backstop
```

```
[dangling reference] "twins → IMPOSTOR" on-camera state is impossible on the live engine
  where:    demo/assets.md:8-9 and :63 (choreography step 0, which Revised note 4 tells tasks 2-4 to reuse verbatim)
  evidence: IMPOSTOR requires issuer-list mimicry + genuine-pattern signature match (scan-backend/src/rules.rs:306 `!c.listed && signature_matched`; lib.rs:165 `find_mimic_source` searches only the fetched issuer list — canonical.rs:63); "Aurelia Industries" is a placeholder not on the issuer list and both twins deliberately fail F2/F3/F4 (assets.md:52-58), so live twins scan UNVERIFIED — exactly what e2e/specs/scan.live.spec.ts:162-183 pins
  fix:      runbook + video script narrate the twin beat honestly as UNVERIFIED + docs-page impostor definition + GUARD_NO_RECORD refusal (keep IMPOSTOR-red for mock-mode beats only), or take a twin re-cast to an issuer-listed name to the coordinator as a step-6 cast change
```

```
[deferred decision] The 4663 buyer identity for the settle plumbing is unnamed; the ported default breaks on mainnet
  where:    plan task 1 ("buyer tokenIn, MM tokenOut — balance+allowance both sides" ported from seed-scratch.sh)
  evidence: the port source defaults buyer to the anvil well-known key (e2e/scripts/seed-scratch.sh:59) — a public key over a dead 0-balance address on 4663; the buyer must hold gas and sign the tokenIn approve for the on-camera settle
  fix:      one line in task 1/2 naming the 4663 buyer (simplest: the demo operator wallet the runbook's bridge step already funds) and dropping the anvil default in seed-demo
```

```
[deferred decision] Task 3's "measure wall-clock" touches chain state but task 3 is dispatched funding-free
  where:    plan task 3 vs Revised note 1 ("upgrade-beat scripting" listed under unfunded tasks 1-5)
  evidence: timing an upgradeTo → drift-check → guard-refusal sequence needs the funded 4663 cast; unfunded, only scripting plus an anvil rehearsal (the step-6 pattern) is possible
  fix:      one clause: unfunded-regime wall-clock comes from the anvil rehearsal; live actuals fold into task 6's budget table
```
