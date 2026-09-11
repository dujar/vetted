---
topic: workers-rust
version: worker 0.8.5 (crates.io, 2026-06-12)
checked: 2026-09-11
sources:
  - https://crates.io/api/v1/crates/worker
  - https://developers.cloudflare.com/workers/languages/rust/
  - https://raw.githubusercontent.com/cloudflare/workers-rs/main/README.md
---

- Crate is `worker` (singular) on crates.io — repo cloudflare/workers-rs. Current 0.8.5
  (2026-06-12). If you remember 0.4/0.5 era APIs, two minors of churn have landed since.
- Status: Cloudflare documents Rust as a first-class Workers language
  (developers.cloudflare.com/workers/languages/rust/). The current README carries NO
  beta/unsupported banner (checked 2026-09-11). The honest production answer for a thin
  stateless JSON API doing outbound RPC calls is workers-rs on Workers — a Rust container
  on Fly/Railway is unnecessary infra here, not a safety upgrade.
- Toolchain: `wasm32-unknown-unknown` target (same as Stylus — one rustup target add for
  both halves of the project).
- Shape: `#[event(fetch)] pub async fn main(req: Request, env: Env, ctx: Context) ->
  Result<Response>`; outbound `fetch` returns the same `Response` type (streaming body +
  serde JSON deserialization supported) — the RPC-probe backend is exactly the happy path.
  Service bindings via a `Fetcher` in `Env`.
- Build/deploy: README uses `wrangler` directly (`npx wrangler dev` / `npx wrangler
  deploy`, config in `wrangler.toml`) — no separate worker-build step in the current
  quickstart path. Durable Objects / SQLite KV bindings are declared in wrangler.toml
  (not needed — spec says no database; registry state lives on-chain).
- Footgun if skipping workers-rs for raw wasm-bindgen: importing a `.wasm` in Workers
  yields a `WebAssembly.Module`, not an `Instance` — you must patch the generated JS
  wrapper (docs give the snippet). Use workers-rs and skip this.
- CORS/auth story: scans are anonymous; add permissive CORS headers on the Response for
  the browser frontend; no session state anywhere.
