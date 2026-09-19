# Step 4 verify — scan-backend plan (2026-09-19)

Checked against main at 43efd99 (clean). Read: plan.md including both Revised blocks, knowledge/ (workers-rust.md, robinhood-chain.md, robinhood-stock-tokens.md, goplus-api.md, rust-wasm-signing.md), spike/findings.md + spike/evidence/calibration_4663.json + spike/fetch/ source, step-1/step-5 findings, step-3 plan, reconciliation/2026-09-19-steps-2-and-5-findings.md, step-feature-state.md, spec.md/journeys.md cited lines, packages/shared (abi.ts, types.ts, types.rs, wire.md, index.ts, src/lib.rs, fixtures, both round-trip tests), frontend/src/lib/api.ts + mockData.ts + chains.ts, scan-backend/ (lib.rs, Cargo.toml, wrangler.toml, Cargo.lock), deployments/ (README, worker.json), .github/workflows/ci.yml, and the local cargo registry copy of worker-0.8.5.

## Hard gates

- **UI without a mockup: PASS.** No screen, view, or visual component — Screens: N/A is correct; the only renderers are step 5's already-built ones.
- **Missing user journey: PASS.** JSON API behind J1/J2/J3; no navigation surface of its own.

## Verified clean (asked-for checks that passed)

- **Absorb-or-use decision is concrete.** Revised note 2 says absorb: port `spike/fetch/` logic into the scan worker, spike URL stays as fallback mirror only, note in findings. The port is same-crate-shape (spike/fetch/Cargo.toml: worker 0.8.5 + serde + serde_json, identical pure-fn pattern) and the source documents the User-Agent footgun (spike/fetch/src/lib.rs:86-89). Buildable as written; not a decide-at-build-time item.
- **Dangling references: none in the consumed exports.** abi.ts:49-63 PROBE_SELECTORS calibrated exactly as Revised note 1 describes (paused 0x5c975abb / blocklist 0xfbac3951); calibration_4663.json carries the full fingerprint (beacon 0xe10b6f6b…1b00, proxy codehash 0x6c1fdd40…, beacon slot set + impl slot empty, uid()==registry id); api.ts seam matches task 4/5 route-for-route (`GET /scan?chainId=&addr=` api.ts:50, `GET /watchdog?chainId=` api.ts:72); WatchdogStats {chainId, runs, baselinePerDay} api.ts:23-29 exactly as Revised note 4; ScanResponse/TerminalState/PowerReportRow/evidenceUrl all exist (types.ts). Mock numbers already 6092/150 (mockData.ts:70). EIP-1967 slot abbreviations match calibration. Chain/RPC constants match wire.md chains table + knowledge robinhood-chain.md (4663/0x1237, 46630/0xb626, 421614); the rate-limit note (robinhood-chain.md:18), L1 SequencerInbox address (:42-43), and screening quote (:40-41) all resolve at the cited lines. Signing recipe (task 6) matches rust-wasm-signing.md verbatim: alloy 2.4.2 set, k256 0.13, getrandom 0.2/js, TxSignerSync + Encodable2718 imports, sync-sign flow.
- **workers-rs 0.8.5 supports the cron task**: `#[event(scheduled)]` documented in worker-macros-0.8.5/src/event.rs:68; schedule.rs present in the pinned crate (local cargo registry).
- **File-scope collisions with parallel steps 3/6: none.** Plan writes scan-backend/** only; no task touches contracts/**, contracts/replicas/**, scripts/deploy/*, or demo/. deployments/worker.json is a reserved writer pin for step 4 (deployments/README.md:9). packages/shared and ci.yml are the shared surfaces both other parallel steps also touch — merge-order rule (step-feature-state.md Next 4) covers them, but see loose ends 5–7 for what the plan should name.
- **Registry dependency stubs correctly against abi.ts alone for tests.** getRecord selector 0x617fba04 is pinned (abi.ts:32); task 8's IMPOSTOR check is fixture-based (no live impostor exists — step-2 findings loose end 7); drift-check e2e is explicitly deferred to step 3's scratch deployments. REVOKED-verdict detection itself needs only getRecord. What does not resolve from abi.ts is the revocationTx extraction — loose end 2.
- **Depth-boundary composition is derivable, not a stall.** Revised note 3's three-way criteria (live P/CRM hit, step-6 replica hits, twins miss) plus calibration_4663.json pin the list uniquely enough to build: structural layout checks (beacon slot set + impl slot empty, beacon resolves implementation(), paused/isBlocked answer at calibrated targets), not proxy-codehash byte-equality. Both plans 4 and 6 carry the same instruction; keep the shipped list in findings for step 6's fidelity test.
- **CI has a scan-backend job** (ci.yml:24-29) running `cargo test` — task 8's "cargo test green in CI" lands on existing plumbing; no new job needed.
- spec.md/journeys.md citations resolve: discipline rules 1–4 (spec.md:25-31), depth boundary :29, revocation trigger :35, watchdog :37 (~150/day; the 6,092 total is spec.md:8), no-DB :56; J1 states journeys.md:12-22 incl. :18 no-code, :19 non-4663 notice, :20 degraded.

## Loose ends

```
[missing prerequisite] No registry-address config seam for /scan and drift-check
  where:    plan tasks 3/4/6 (registry records, ACTIVE records) vs scan-backend/wrangler.toml:8
  evidence: the worker must eth_call getRecord somewhere, but no plan line names where the registry address comes from per chain; grep of plan.md finds only REGISTRAR_KEY. deployments/421614.json / 46630.json are step 3's outputs (deployments/README.md:11-12) and do not exist yet (step 3 still planned, no findings). Step 5 hit this exact gap and used VITE_REGISTRY_ADDRESS env (step-5 findings.md:19; step-3 plan Revised note 6).
  fix:      pin per-chain wrangler vars (e.g. REGISTRY_ADDRESS_421614/_46630/_4663) in task 4/6, unset => record:null and registry-dependent rules degrade to canonical-fetch-only.
```

```
[dangling reference] Revoke event signature is pinned nowhere — revocationTx cannot be extracted
  where:    plan Revised note 5 / task 3 vs abi.ts:10-14 and step-3 plan.md:45
  evidence: abi.ts carries functions only (getRecord/verify/revoke), no events; step-3's plan says just "Verify/Revoke events" with no signature or indexed-ness, so topic0 for eth_getLogs is not computable from any pinned surface. The frontend already renders "not indexed yet" without revocationTx (step-5 findings.md:44).
  fix:      step-3's merge pins the exact event signature(s) into packages/shared (a merge-time block like PROBE_SELECTORS); until then step 4 ships revocationTx:null on the existing not-indexed path — say so in task 3.
```

```
[deferred decision] Watchdog counter (task 5) leaves three build-blocking specifics open
  where:    plan task 5 + Open questions; knowledge robinhood-chain.md:42-43
  evidence: (a) the eth_getLogs read is against Ethereum L1, but no L1 RPC endpoint is pinned anywhere (robinhood-chain.md gives L1 contract addresses only; wire.md chains table has no L1 row; task 1 pins RPCs for 4663/46630/421614 only); (b) which SequencerInbox event topic counts one "filterer run" is unknown (open question admits it; reconciliation target = 6,092/6wk, spec.md:8); (c) the degrade path "baseline + provenance link" does not fit WatchdogStats {chainId, runs, baselinePerDay} (api.ts:23-29) — no provenance field, runs is a required number.
  fix:      pin an L1_RPC_URL var + a topic candidate in task 5, and add provenanceUrl (nullable) plus the degrade value of runs to the shared WatchdogStats extension; the degrade-and-flag-coordinator escape stays as written.
```

```
[orphan output] Signing fallback scripts/registrar-cli does not exist and no step owns it
  where:    plan Open questions (RESOLVED paragraph) vs step-feature-state.md scope column
  evidence: scripts/ contains only toolchain.sh; scripts/deploy/core is step 3's, scripts/deploy/replicas step 6's, scripts/seed-demo step 9's. If the wasm runtime wiring "fights back", the named fallback is unbuildable inside step 4's scan-backend/** scope — the degraded path is not precise enough to execute as written.
  fix:      one plan line: if the fallback triggers, step 4 builds scripts/registrar-cli itself (scope amendment) or the fallback reroutes to step 7's registrar handoff.
```

```
[scope contradiction] Scope "Creates: scan-backend/** only" vs the Revised notes' required writes
  where:    plan.md:30 vs Revised note 4 (packages/shared WatchdogStats) and the writer pin at deployments/README.md:9
  evidence: task work must append WatchdogStats to packages/shared (index.ts/src/lib.rs append points) and step 4 is worker.json's co-writer, but the Scope line names neither — the implementer must guess whether shared/worker.json edits are in scope.
  fix:      amend Scope: "scan-backend/** + packages/shared append (watchdog module) + deployments/worker.json deploy record".
```

```
[no check] The shared WatchdogStats extension ships with zero wire coverage
  where:    Revised note 4 vs packages/shared/tests/roundtrip.test.ts:30,:65
  evidence: both round-trip tests read only fixtures/verdict and fixtures/guard; a type appended without a fixture dir + test extension gets no round-trip in either language, contradicting wire.md:4-6 ("golden fixtures … CI enforces both directions") and the plan's own "golden fixtures are the contract test" (task 4).
  fix:      add fixtures/watchdog/ + extend both round-trip tests (deliberate step-1-file edits, flagged at merge like step 5's ci.yml edit), or state the deferral to step 8 explicitly in Revised note 4.
```

```
[no check] wasm32 compile of the signing stack is never exercised by CI
  where:    task 6 + Check line vs .github/workflows/ci.yml:24-29
  evidence: the scan-backend job runs native `cargo test` only; the recipe's entire known risk is wasm32-only (getrandom js compile_error, alloy on wasm32 — rust-wasm-signing.md:9,17-18,44-46). Native CI stays green while the deployed artifact stops compiling.
  fix:      add `cargo check --target wasm32-unknown-unknown` to the scan-backend CI job (step-1-owned ci.yml edit, flagged at merge), or extend the Check line's wrangler-dev gate to "builds including the signing path".
```

```
[unhandled sibling] Rust side has no source of truth for the calibrated probe bytes
  where:    task 1 vs abi.ts:58 ("never hardcode the value")
  evidence: PROBE_SELECTORS lives in TS; the Rust worker cannot import it and must inline 0x5c975abb/0xfbac3951. The only tripwires are the TS vitest recompute and task 8's live P-token check — a transcription error otherwise surfaces only live.
  fix:      mirror the two bytes as consts in the scan-backend crate with a unit test asserting them against the calibration values.
```

## Notes

- spec.md:37 carries ~150/day; the 6,092 total the plan cites to :37 is actually spec.md:8. Cosmetic — the number itself is sourced.
- goplus-api.md is on the knows line but no task calls GoPlus — correct per the spike result (GoPlus structurally cannot flag this pattern); not counted.
