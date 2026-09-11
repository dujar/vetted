# vetted_ (placeholder name)

Token verification + guarded swap for Robinhood Chain stock tokens. Verdicts: VERIFIED / IMPOSTOR / UNVERIFIED / REVOKED — every line evidence-linked, nothing guessed. Built for the Arbitrum Open House Singapore Online Buildathon (deadline 2026-10-04 23:59 SGT).

## Layout

| path | what | owner step |
|---|---|---|
| `contracts/` | Stylus contracts — cargo workspace, one `Stylus.toml` per crate | 3 (core), 6 (replicas), 7 (hardening + 4663) |
| `contracts/core/hello/` | placeholder crate proving the toolchain harness (entrypoint + native test) | 1 |
| `scan-backend/` | Cloudflare Workers in Rust — verdict engine | 1 (scaffold), 4 |
| `frontend/` | Vite 8 + React 19 + Tailwind 4 + wagmi 3 | 1 (scaffold/tokens/primitives), 5 (screens) |
| `packages/shared/` | the wire contract: types (TS + Rust), registry ABI, golden fixtures | 1 |
| `scripts/` | toolchain + deploy scripts | 1, 3, 6 |
| `e2e/` | end-to-end journeys | 8 |
| `demo/` | demo assets + narration | 6, 9 |
| `deployments/` | append-point JSONs, one per deploy target — written only by deploy steps | deploy steps |
| `.agent-workbench/` | build plans, knowledge, step state | the build system |

Build order and current step status: `.agent-workbench/step-feature-state.md`.

## Quickstart

```bash
./scripts/toolchain.sh                                  # Rust stable + wasm32 target + cargo-stylus 0.10.9

cd contracts && cargo test --workspace                  # contracts (native stylus test host)
cd contracts && cargo build --workspace --target wasm32-unknown-unknown

cd scan-backend && cargo test                           # backend unit tests (pure route fns)
cd scan-backend && npx wrangler deploy                  # deploy worker

cd frontend && npm ci && npm run build && npm test      # frontend
cd packages/shared && npm ci && npm test && cargo test  # wire-contract round-trips, both sides
```

## The wire contract

`packages/shared/wire.md` is the contract every consumer builds against: verdict enum, guard revert reasons, scan request/response, registry record, chain ids + RPCs. `types.ts` and `types.rs` must round-trip the golden fixtures in `packages/shared/fixtures/` exactly — CI enforces it. Registry read ABI + selector constants: `packages/shared/abi.ts` (the `PROBE_SELECTORS` block is step-2-reserved).

## CI

`.github/workflows/ci.yml` — four jobs: contracts (wasm build + native tests), scan-backend, shared (both round-trips), frontend (tsc + build + vitest).
