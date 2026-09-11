# Step 1 — repo scaffold

status:     merged
branch:     step-1-repo-scaffold (PR #1, merged into main as 5e2ff35; merge suite green)
deployed:   worker https://vetted-scan-backend.dujar-coding.workers.dev (GET /health → {"ok":true}) · pages https://vetted-1un.pages.dev (HTTP 200) · GitHub https://github.com/dujar/vetted (public)

## What was built

The CI-green monorepo every later builder lands inside: toolchain pins (`rust-toolchain.toml` stable + wasm32, `scripts/toolchain.sh` → cargo-stylus 0.10.9, no protoc), four-job CI (`.github/workflows/ci.yml`), `contracts/` cargo workspace + `core/hello` placeholder (#[entrypoint] + native `stylus_sdk::testing` TestVM tests, wasm build proven), `scan-backend/` workers-rs 0.8.5 hello (`/health` → `{"ok":true}`, pure-fn route tests per verify.md loose end 5), `frontend/` Vite 8 + React 19 + Tailwind 4 (CSS-first, no postcss) + wagmi 3.7.7 (AppKit excluded) + theme tokens/primitives + `lib/chains.ts` with live-verified RPCs, `packages/shared/` wire contract (`wire.md`, `types.ts` ⇄ `types.rs`, `abi.ts`, golden fixtures, byte-identical TS + structural Rust round-trip tests), `deployments/` append-point layout, `e2e/`+`demo/` reserved, README stub. Both hello deploys are LIVE (pipeline proven day one; wrangler OAuth token does have workers+pages write — verify.md loose end 6's premise is stale).

## Where the plan was wrong

- `stylus_sdk::testing` tests don't compile without the template's crate shape: `#![cfg_attr(not(any(test, feature="export-abi")), no_std/no_main)]` + `#[macro_use] extern crate alloc;` (the storage macros expand `alloc::` paths). Applied in `contracts/core/hello/src/lib.rs` — step 3's crates must copy this header.
- Guard revert reason renames: serde `SCREAMING_SNAKE_CASE` on `RecordRevoked` yields `RECORD_REVOKED`, not `GUARD_RECORD_REVOKED` — explicit per-variant `#[serde(rename)]` in `types.rs`.
- The 46630 RPC: docs.robinhood.com/chain/connecting is JS-rendered (Vocs), no markdown endpoint. URL recovered from the page's own asset bundle (`rpc.testnet.chain.robinhood.com`) and live-probed `eth_chainId → 0xb626` — satisfies loose end 3's intent; evidence in `wire.md` chains table.
- Knowledge workers-rust.md said "no separate worker-build step": the current template still builds via `[build] command = cargo install 'worker-build@^0.8' && worker-build --release`, output `build/index.js` (not the old shim.mjs). `scan-backend/wrangler.toml` uses it; deploy proven.
- SELECTORS in `abi.ts` are literals (so step 8 can use them without a runtime); a vitest test recomputes them with viem — it caught my first transcription, then passed with `getRecord=0x617fba04, verify=0x73c7cf61, revoke=0xafd0224b, commit=0x498ab631, execute=0x61461954`.

## What the next step needs to know

- **Wire contract**: `packages/shared/wire.md` is the source of truth; scan request is `GET /scan?chainId&addr`; registry record is the seven-field step-3 shape with `status` u8→"VERIFIED"/"REVOKED" and `riskFlags` u256-as-decimal-string (JSON has no u256). Changing shape = edit `types.ts` + `types.rs` + fixtures + both round-trip tests together; `index.ts`/`src/lib.rs` are append points — add module files, never edit others'.
- **abi.ts**: step 5 consumes `REGISTRY_ABI`/`GUARD_ABI`/`SELECTORS`/`GUARD_REVERT_REASONS`; step 2's ONLY merge-time touch outside `spike/` is replacing the marked `PROBE_SELECTORS` placeholder block (`paused`, `blocklist` keys stay stable).
- **chains.ts**: 4663 + 46630 defined here with live-probed RPCs; 421614 re-exports viem's `arbitrumSepolia`. 46630 needs no more probing in step 3's window.
- **wagmi**: `lib/wagmi.ts` `getWagmiConfig()` is lazy and env-gated — no projectId (`VITE_WALLETCONNECT_PROJECT_ID`) boots read-only with zero connectors instead of throwing. Only operator action left anywhere: a WalletConnect Cloud projectId. Deploys need nothing (token has scopes).
- **deployments/**: append-point JSONs, one per target — writers pinned in `deployments/README.md` (step 3 → `421614.json`/`46630.json`, step 7 → `4663.json`).
- **CI**: four jobs (contracts wasm+native, scan-backend, shared both round-trips, frontend tsc+build+vitest); runs on push to main + PRs. Node 20 deprecation annotations from actions v4 are cosmetic — bump to v5 someday, not urgent.
- **Workspace**: `contracts/Cargo.toml` members = `core/*` — step 3 drops crates in, no workspace edit. Release profile (lto, panic=abort) already at workspace root.

## Out of scope, left broken

- Nothing broken. Deliberate shortcuts: frontend `verdicts.ts` duplicates the shared VERDICTS union (step 5 switches to `vetted-shared` imports when wiring the API client — noted in-file); `pages.json` records the production alias, the per-deployment URL probes intermittently failed TLS from this machine only.

## Review round 1 (APPROVED) — non-blocking notes, disposition

1. `e2e/`+`demo/` missing from the pushed tree (empty dirs) — FIXED: `.gitkeep` added to both; reserved layout now survives a clean checkout.
2. plan.md:45 still carries the stale `blocked-auth` fallback parenthetical — ACCEPTED AS IS: the plan is a historical record; the truth (deploys live, token had scopes) is recorded here and in `deployments/*.json`. plan-agent's reconcile round owns plan text.
3. `verdicts.ts` duplicates the shared union — ACCEPTED AS IS: deliberate, marked with the step-5 switch point; drift window ends when step 5 wires `vetted-shared` imports.
4. 4 rustc `unexpected cfg condition value: contract-client-gen` warnings in `contracts/core/hello` — ACCEPTED AS IS: emitted by stylus-sdk macro expansion, not scaffold code; cosmetic.

