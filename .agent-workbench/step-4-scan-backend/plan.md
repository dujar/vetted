# Step 4 — Scan backend: stateless Workers-Rust verdict engine + watchdog + drift-revoke

## Resources
- spec:    ../product/spec.md   (Scope 1 & 4 spec.md:34,37; revocation trigger spec.md:35; discipline rules spec.md:25–30; depth boundary spec.md:29; no database spec.md:56)
- journeys: ../product/journeys.md  (J1 states journeys.md:12–22; degraded banner :20; watchdog :15)
- knows:   ../knowledge/workers-rust.md, ../knowledge/robinhood-chain.md, ../knowledge/robinhood-stock-tokens.md, ../knowledge/goplus-api.md, ../knowledge/rust-wasm-signing.md
- learned: ../step-2-spike-gate/findings.md  (calibrated fingerprint + probe selectors; /rhj/assets sample; beacon-resolution mechanics — must exist before this step starts)
- learned: ../step-1-repo-scaffold/findings.md  (scaffold MERGED at 5e2ff35 — working hello worker in `scan-backend/`, wire contract pins, live-probed RPCs)

> **Revised** — step-1 findings reconciliation (2026-09-11):
> 1. `scan-backend/` already contains the deployed hello worker (workers-rs 0.8.5, `/health` live) — build path is `worker-build@^0.8` producing `build/index.js`; knowledge workers-rust.md's "no separate build step" is stale. Reuse the existing crate shape and `wrangler.toml`; deploy credentials proven (OAuth token has workers+pages scopes).
> 2. Task 4's wire shape: `packages/shared/wire.md` is the source of truth (`GET /scan?chainId&addr`; `status` u8→"VERIFIED"/"REVOKED"; `riskFlags` u256 as decimal string). Extend shared types only via append points (`index.ts` / `src/lib.rs` / new module files); the Rust side already carries explicit per-variant `#[serde(rename)]` (e.g. `RecordRevoked` → `RECORD_REVOKED`) — don't re-derive naming. Shape changes touch `types.ts` + `types.rs` + fixtures + both round-trip tests together.
> 3. Task 1's RPC endpoints for 4663/46630/421614 are already live-probed and pinned in `packages/shared/wire.md` chains table + `frontend/src/lib/chains.ts` — use them; no endpoint discovery.

## Stack
workers-rs 0.8.5 on Cloudflare Workers — stateless, no DB (spec.md:56); alloy-consensus primitives + k256 for RPC and registrar signing compiled to wasm.

## Goal
After this step, address-in → verdict-out works end to end: bytecode fetch, EIP-1967/beacon resolution, probe staticcalls, fingerprint match, live issuer canonical fetch, rule engine → VERIFIED / IMPOSTOR / UNVERIFIED / REVOKED with per-line evidence; the watchdog counter endpoint; and the registrar's drift-watch that auto-revokes stale records. The registry stays the state (spec.md:35); the backend stays stateless.

## Scope
- Creates: `scan-backend/**` only (routes, rule engine, probe client, tests, wrangler.toml cron).
- Out of scope: UI (step 5), contracts (step 3), registrar custody policy doc (step 7).

## User journey
N/A — JSON API behind the J1/J2/J3 UIs.

## Screens
N/A.

## Tasks
1. RPC probe client: `eth_getCode`, `eth_getStorageAt` (impl slot `0x360894…82bbc`, beacon slot `0xa3f0a…3d50`), `beacon.implementation()` staticcall, probe staticcalls (`paused()`, per-address blocklist) with the step-2-calibrated selectors; `eth_call` against the public RPCs of 4663/46630/421614 — rate-limit-aware (single-flight + short in-isolate cache; the public 4663 RPC is rate-limited, knowledge robinhood-chain.md:18).
2. Fingerprint matcher — the depth boundary (spec.md:29): bytecode + beacon layout matches the calibrated Robinhood signature → full verdicts; anything else → UNVERIFIED + structural heuristics labeled advisory; no code → `NOT_CONTRACT` terminal state (journeys.md:18).
3. Rule engine as pure functions over fixtures (the heavily-tested core), discipline rules 1–4 verbatim (spec.md:25–30). IMPOSTOR requires positive evidence: same name/symbol canonical at a different address, impl ≠ canonical impl, or mimic-while-unverified — canonical ground truth = live `/rhj/assets` (knowledge robinhood-stock-tokens.md:12–27) + registry records; missing evidence → UNVERIFIED, never guess; canonical fetch failure → `degraded: true`, all verdicts UNVERIFIED (journeys.md:20); REVOKED links the revocation tx (journeys.md:14). Every power-report row carries its evidence payload (tx / slot / bytecode diff / probe result, spec.md:30).
4. `GET /scan?chainId&addr` → wire-shaped JSON per `packages/shared` types; golden fixtures are the contract test. Non-4663 chains scan read-only with the stock-token-verdicts-only-on-4663 notice field (journeys.md:19).
5. `GET /watchdog?chainId=4663` → cumulative sequencer-filterer runs in ONE RPC read vs the published baseline constant (~150/day; 6,092 total measured 2026-08, spec.md:37 — two numbers, no stored history). Pin the counter source first (see Open questions) — leading candidate: L1 SequencerInbox `0xBd0D173EEb87D57A09521c24388a12789F33ba96` event logs via one `eth_getLogs` count (knowledge robinhood-chain.md:42–43; official screening quote for widget copy :40–41). Degrade: baseline + provenance link if the count doesn't reconcile.
6. Registrar identity + drift revoke: `REGISTRAR_KEY` as a Worker secret; EIP-1559 signing per the **compile-verified recipe** in knowledge/rust-wasm-signing.md — alloy-signer-local 2.4.2 (+alloy-signer, alloy-consensus, alloy-eips, alloy-network all 2.4.2, default-features off, eips `std`), k256 **0.13** (never 0.14 alongside alloy — two incompatible k256 crates), getrandom 0.2 with the `js` feature in this crate's Cargo.toml (without it: compile error at getrandom's `compile_error!`, not a runtime panic); flow `PrivateKeySigner::from_bytes` → `sign_transaction_sync(&mut TxEip1559)` → `into_signed` → `TxEnvelope` → `encoded_2718()` → `eth_sendRawTransaction` via the worker's fetch (imports: `alloy_network::TxSignerSync`, `alloy_eips::eip2718::Encodable2718`); `POST /registry/drift-check` (admin-secret gated, same code path as the cron) + wrangler.toml cron trigger — for each ACTIVE record, resolve live impl vs record.impl, drift → revoke tx (spec.md:35). The manual endpoint exists for the demo's upgrade→revocation beat reliability; the cron is the production story.
7. CORS permissive, no session state (knowledge workers-rust.md:32); structured logs — the hackathon's observability is `wrangler tail` + `/health`; no separate observability step exists and none is needed for the demo window.
8. Tests: unit (rule engine over shared golden fixtures; matcher over recorded bytecode fixtures from 4663), integration script (read-only live 421614 + live /rhj/assets fetch), drift-check end-to-end once step 3's scratch deployments exist. Check: `cargo test` green; live script returns a VERIFIED-shaped verdict for a genuine 4663 token (e.g. P) and IMPOSTOR-shaped for an unregistered mimic fixture.

Check: `cargo test` green in CI; `wrangler dev` serves /scan with a fixture verdict; the live integration script's P-token verdict matches the on-chain ground truth assembled in findings.

## Open questions
- Watchdog counter mechanism is NOT pinned in knowledge — needs one spike-grade reconciliation against the ~150/day baseline; if the L1-log count doesn't reconcile within a day, ship the degrade path (baseline + provenance) and flag the coordinator. Never store history (spec.md:37).
- RESOLVED 2026-09-11 by knowledge/rust-wasm-signing.md: the wasm EIP-1559 signing gap is closed with the compile-verified recipe in task 6 (its declared week-1 unknowns — runtime getRandomValues wiring inside a live Worker, and bundle size — are still unverified: confirm both in `wrangler dev` early in this step). Fallback only if runtime wiring fights back: `scripts/registrar-cli` runs the identical loop operator-side — the key stays backend-held; record the decision in findings.
