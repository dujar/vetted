# Step 1 — Monorepo scaffold: toolchain, CI, hello-world deploys, shared wire types, theme → code

## Resources
- spec:       ../product/spec.md
- journeys:   ../product/journeys.md
- theme:      ../product/theme.css
- components: ../product/components.html
- screens:    ../product/screens/scan.html (structure reference only — screens are built in step 5)
- knows:      ../knowledge/frontend-stack.md, ../knowledge/workers-rust.md, ../knowledge/stylus-toolchain.md, ../knowledge/robinhood-chain.md, ../knowledge/arbitrum-sepolia.md
- exists:     (nothing — repo is empty except .agent-workbench/)

## Stack
Rust/Stylus contracts (stylus-sdk 0.10.9, wasm32-unknown-unknown), workers-rs 0.8.5 backend, Vite 8.3 / React 19.3 / Tailwind 4.3 frontend, wagmi 3.7.7 + viem 2.56.3, TS+Rust shared wire types. All versions per knowledge, verified 2026-09-11 — no assumed versions.

## Goal
After this step the repo is a CI-green monorepo where every later builder can land work without touching the same files: the `contracts/ scan-backend/ frontend/ packages/shared/ scripts/ e2e/ demo/ deployments/` layout exists, toolchains are pinned, each member has one passing test, a hello-world Worker and Pages are deployed (the deploy pipeline proven on day one), and the **wire contract** — verdict enum, guard revert reasons, record shape — is pinned so steps 3–6 build independently against the same types.

## Scope
- Creates: repo root (`.gitignore`, README stub, `rust-toolchain.toml`), `.github/workflows/ci.yml`, `contracts/` (cargo workspace root; `core/` and `replicas/` reserved dirs — Stylus.toml per crate, workspace support since sdk 0.10.0, knowledge stylus-toolchain.md:24–25), `scan-backend/` (workers-rs hello crate + wrangler.toml + one test), `frontend/` (vite scaffold + tailwind 4 vite plugin + wagmi + react-query + theme tokens from theme.css + primitives from components.html: Button, Card, Chip/Badge, VerdictBanner, skeleton; one vitest test), `packages/shared/` (wire.md + types.ts + types.rs + golden fixtures + round-trip tests both sides), `deployments/` (append-point dir, one JSON per chain, written only by deploy steps), `scripts/`, `e2e/` and `demo/` reserved.
- Wire contract pinned here (source of truth: journeys.md): verdict enum VERIFIED|IMPOSTOR|UNVERIFIED|REVOKED (spec.md:1, journeys.md:14); guard revert reasons verbatim GUARD_NO_RECORD|GUARD_RECORD_REVOKED|GUARD_PAUSED|GUARD_BLOCKLISTED|GUARD_IMPL_MISMATCH (journeys.md:32); scan request/response incl. power-report rows each carrying an evidence-link field (journeys.md:14, spec.md:30); registry record {status, risk flags, verifiedAt, implementation pointer, registrar} (spec.md:35); chain ids 4663 mainnet / 46630 scratch / 421614 mirror + RPCs from knowledge robinhood-chain.md:14–21, arbitrum-sepolia.md:11–13.
- Registry read ABI shipped here (`packages/shared/abi.ts`: human-readable entries + exported selector constants) — signatures fixed from step-3's plan text: `getRecord(token)`, `verify(token,…)`, `revoke(token,reason)`, `commit(tokenIn,tokenOut,amountIn,minOut)`, `execute()`, plus the `GUARD_*` revert-reason constants. Steps 5/8 consume real ABI before any contract exists; `PROBE_SELECTORS` (`paused()`, blocklist probe) ship as clearly-marked placeholder bytes that step 2 replaces with its calibrated ones at merge.
- Frontend wallet decision (made here — the knowledge trap, frontend-stack.md:14–19): **wagmi 3.7.7 + built-in wagmi/connectors (WalletConnect); AppKit excluded** (wagmi-2.x-only). WalletConnect projectId from env.
- Out of scope: any screen, any contract logic, any backend rule (steps 3–6); cargo-stylus in CI (local tool only).

## User journey
N/A — infrastructure; no UI surface beyond theme primitives.

## Screens
N/A — no new surface; screens arrive in step 5 from the existing mockups.

## Tasks
1. `git init` + GitHub repo (public recommended — contract-quality credibility) + push; `.gitignore` (target/, node_modules/, .env*, .dev.vars, out/, cache/).
2. Toolchain: `rust-toolchain.toml` (stable ≥1.91, knowledge stylus-toolchain.md:16–17), `rustup target add wasm32-unknown-unknown` (one target for both Stylus and workers, knowledge workers-rust.md:19–20); `scripts/toolchain.sh` installs cargo-stylus **0.10.9** from crates.io (≥0.10.8 required — fragmented-contract verify fix, knowledge stylus-toolchain.md:32–33). Do NOT add a protoc step (knowledge :37–38).
3. `contracts/`: cargo workspace root + placeholder crate `contracts/core/hello` with one `#[entrypoint]` contract and one native test via `stylus_sdk::testing` — proves the harness, no deploy.
4. `scan-backend/`: workers-rs hello crate, `#[event(fetch)]` main, `GET /health` → `{ok:true}`; one route unit test; wrangler.toml; `npx wrangler deploy` → workers.dev URL recorded in `deployments/worker.json`.
5. `frontend/`: vite 8 react-ts scaffold; `@tailwindcss/vite` 4.3.3, CSS-first `@import "tailwindcss"` — no postcss config (knowledge frontend-stack.md:26–29); wagmi 3.7.7 + viem 2.56.3 + @tanstack/react-query; `src/lib/chains.ts` with the three verified RPCs; `src/theme/tokens.css` = theme.css as custom properties (dark security-terminal, green/red duality); primitives per components.html; one vitest test (VerdictBanner renders all four verdicts).
6. `packages/shared/`: `wire.md` + `types.ts` + `types.rs` + golden JSON fixtures (one per verdict, one per guard reason); **`abi.ts`** — the read ABI per the Scope bullet above (registry fns from step-3's plan text, guard revert-reason constants, placeholder `PROBE_SELECTORS`). vitest + cargo tests round-trip both implementations against the fixtures; `index.ts`/`lib.rs` re-exports are append points — later steps add module files, never edit others'.
7. CI `.github/workflows/ci.yml`: jobs — contracts (`cargo build --workspace --target wasm32-unknown-unknown` + `cargo test --workspace`), backend (`cargo test`), frontend (tsc + build + vitest), shared (both round-trip tests); green on first push.
8. Cloudflare Pages hello (static shell) via `npx wrangler pages deploy` → URL recorded in `deployments/pages.json`.
9. README stub: layout map + build order pointing at `.agent-workbench/step-feature-state.md`.

Check: push → all CI jobs green; from a clean checkout, `wrangler deploy` + `pages deploy` succeed and URLs resolve; `cargo test --workspace` and `npm test` pass locally.

## Open questions
- Operator actions, day 1: WalletConnect Cloud projectId; Cloudflare account + API token; GitHub repo creation. None block scaffold code, all block the hello-world deploys.
- Repo public vs private — recommend public for the contract-quality criterion; operator confirms.
