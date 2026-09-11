---
topic: stylus-toolchain
version: stylus-sdk 0.10.9 / cargo-stylus 0.10.9 / Rust stable 1.99.0
checked: 2026-09-11
sources:
  - https://crates.io/api/v1/crates/stylus-sdk
  - https://api.github.com/repos/OffchainLabs/stylus-sdk-rs/releases
  - https://docs.arbitrum.io/stylus/quickstart.md
  - https://releases.rs/
---

- cargo-stylus lives in the stylus-sdk-rs MONOREPO now; crates.io `cargo-stylus` and
  `stylus-sdk` are both 0.10.9 (2026-08-11). The old standalone repo
  OffchainLabs/cargo-stylus is stale — last release 0.6.3, 2025-08-20. Old tutorials
  pointing at it are wrong.
- Rust: quickstart requires "v1.91 or newer"; stable is 1.99.0 (2026-08-05). crates.io
  declares NO MSRV for 0.10.9 — use current stable, don't pin old.
- Target: `wasm32-unknown-unknown` (build output path in quickstart:
  `target/wasm32-unknown-unknown/release/*.wasm`).
- 0.8.0 (2025-02) changed hostio access: global fns like `stylus_sdk::msg::value()`
  deprecated → use the `Host` via `.vm()` on `#[storage]` contracts; `stylus_sdk::testing`
  gives a native unit-test host. If you remember pre-0.8 global hostio style, it's gone.
- 0.9.0 (2025-05) added constructors and trait-based inheritance; removed HostIO caching.
- 0.10.0 (2026-01) added cargo-workspace support — each contract crate is marked with a
  `Stylus.toml` file — plus `stylus-tools`, nested-struct/tuple return types,
  `contract-client-gen` (contract-as-library calls).
- Deploy flow (quickstart, verbatim commands): `cargo stylus check` (dry-run: compiles
  WASM + validates deploy AND activation; takes minutes on first run), then
  `cargo stylus deploy --endpoint <RPC> --private-key 0x…`. Deploy sends TWO txs
  (deployment + activation) automatically.
- Size (ArbOS 61+): Stylus contracts up to 96 KB; compressed WASM > 24 KB is auto-split
  into fragments + a "collection" contract — callers use the collection address. `verify`
  of fragmented contracts was broken before 0.10.8 — pin >= 0.10.8.
- Reproducible builds/verify run via Docker (0.10.6+); 0.10.9 adds a `[wasm-opt]` table in
  `Stylus.toml` that pins a Binaryen version + flags, folded into the project hash. If
  verify mysteriously mismatches across machines, this is the knob.
- The old protobuf/protoc pin gotcha no longer appears anywhere in the current quickstart —
  do not add a protoc install step on stale advice.
- Unchanged: `sol_storage!`/`#[entrypoint]`/`#[public]` programming model, alloy types.
