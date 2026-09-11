---
topic: rust-wasm-signing
version: alloy-signer-local 2.4.2 / k256 0.13.4 / getrandom 0.2.17 (compile-verified 2026-09-11)
checked: 2026-09-11
sources:
  - https://crates.io/api/v1/crates/k256, /alloy-signer-local, /getrandom (registry, 2026-09-11)
  - https://docs.rs/getrandom/latest/getrandom/ (WebAssembly support section)
  - https://developers.cloudflare.com/workers/runtime-apis/web-crypto/
  - live compile: cargo check --target wasm32-unknown-unknown --release, 2026-09-11 (clean)
---

- COMPILE-VERIFIED: EIP-1559 signing compiles for Workers (wasm32-unknown-unknown) with exactly
  this set: alloy-signer-local 2.4.2 + alloy-signer 2.4.2 (default-features = false),
  alloy-consensus 2.4.2 + alloy-eips 2.4.2 (default-features = false, features = ["std"]),
  alloy-network 2.4.2 (default-features = false — it has NO "std" feature, only "k256"),
  k256 0.13 (default-features = false, features = ["ecdsa","arithmetic"]),
  getrandom = { version = "0.2", features = ["js"] }. `cargo check --target
  wasm32-unknown-unknown --release` finished clean 2026-09-11.
- THE footgun: getrandom 0.2.17 enters via crypto-bigint 0.5.5 → rand_core 0.6 → elliptic-curve
  0.13 (pulled from BOTH alloy-signer and k256's ecdsa path). Without the js feature it is a
  COMPILE error, not a runtime panic: compile_error!("the wasm*-unknown-unknown targets are not
  supported by default, you may need to enable the \"js\" feature") at
  getrandom-0.2.17/src/lib.rs:346. You must add getrandom 0.2/js to your own Cargo.toml —
  libraries won't (docs.rs explicitly warns libraries against enabling it). js routes to
  crypto.getRandomValues, which the Workers runtime provides (web-crypto page).
- If the graph ever moves to getrandom 0.3/0.4 (latest 0.4.3), the mechanism CHANGED: feature
  `wasm_js` PLUS RUSTFLAGS '--cfg getrandom_backend="wasm_js"' (docs.rs/getrandom). Not needed
  at the current pins.
- API paths I would have misremembered (alloy 2.x): the sync-sign trait import is
  `use alloy_network::TxSignerSync;` — NOT alloy_signer::signers (that module no longer exists);
  2718 encoding is `use alloy_eips::eip2718::Encodable2718;` — NOT re-exported from
  alloy_consensus root. Flow: PrivateKeySigner::from_bytes(&B256) →
  signer.sign_transaction_sync(&mut TxEip1559{..}) → tx.into_signed(sig) → TxEnvelope →
  encoded_2718() → send bytes as eth_sendRawTransaction via the worker's fetch. (all
  compile-verified)
- Version skew: standalone k256 is 0.14.0 (2026-07-08, crates.io) but alloy-signer-local 2.4.2
  pins k256 ^0.13 (resolves 0.13.4). Add k256 = "0.14" alongside and you get TWO k256 crates
  with incompatible types — use 0.13 in anything that touches alloy signers.
- Honest production shape: the recipe above pulls NO HTTP stack — RPC send stays in the Worker's
  fetch; registrar key in a Worker Secret; signing itself needs no RNG (fixed key). An external
  key service (e.g. Turnkey) is the production upgrade path, but its Cloudflare-Workers support
  was NOT verified here (docs.turnkey.com sitemap has no Workers/serverless page, 2026-09-11) —
  treat "delegated signer" as unvetted for this build.
- Unverified, week-1 spike: running the built wasm inside a real Worker (wasm-bindgen
  getRandomValues wiring at runtime) and bundle size with the crypto stack. Compile-clean is not
  runtime-pass.
