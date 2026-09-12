#!/usr/bin/env bash
# Toolchain bootstrap: Rust stable (rust-toolchain.toml) + wasm32 target + cargo-stylus.
# cargo-stylus >= 0.10.8 is required (fragmented-contract verify fix — knowledge
# stylus-toolchain.md); we pin exactly 0.10.9.
# NOTE: no protoc step — the old protobuf/protoc pin gotcha is gone from the current
# Stylus quickstart; do not add one (knowledge stylus-toolchain.md).
set -euo pipefail

CARGO_STYLUS_VERSION="0.10.9"

rustup target add wasm32-unknown-unknown

if ! cargo stylus -V 2>/dev/null | grep -q "$CARGO_STYLUS_VERSION"; then
  echo "installing cargo-stylus $CARGO_STYLUS_VERSION from crates.io..."
  cargo install cargo-stylus --version "$CARGO_STYLUS_VERSION" --locked
fi

cargo stylus -V
