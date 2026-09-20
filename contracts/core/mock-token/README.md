# contracts/core/mock-token — mock pattern tokens (step 3, plan tasks 4/5)

The deployable mock stock tokens for the guard's on-chain integration, mirroring
the GENUINE Robinhood stock-token layout (ground truth:
`spike/evidence/calibration_4663.json`, `spike/findings.md`):

- **Beacon** (`MockBeacon.sol`) — carries the upgradeable implementation
  pointer (`implementation()` 0x5c60da1b / `upgradeTo`) AND the per-address
  blocklist (`isBlocked(address)` 0xfbac3951). The blocklist STATE lives here,
  not on the token — the guard probes it on the RESOLVED BEACON.
- **Impl** (`MockTokenImpl.sol`) — minimal ERC-20 + global pause. Runs ONLY as
  the delegatecall target of the forwarder (or standalone as the plain
  no-pattern token). Deliberately NO `isBlocked` — that selector must revert
  through the proxy/impl (calibrated, live-verified). Storage semantics under
  the genuine forwarder (no proxy init): all storage reads zero per-proxy until
  a tx through the proxy writes it — `mint`/`pause` go THROUGH the proxy.
  `name`/`symbol`/`admin` are immutables (identical through every proxy;
  per-token names are cosmetic — the guard never probes them).
- **Forwarder** — NOT a Solidity-written proxy: the tokens' runtime code IS the
  genuine 283-byte forwarder (`spike/evidence/p_proxy.hex`, live 4663) with
  only the embedded 20-byte beacon address patched
  (`ForwarderPatch.sol` + `RawHull.sol`). The patcher mirrors the guard's
  shape-extraction algorithm (PUSH32 `0x7f` + 12 zero bytes + 20-byte address,
  `implementation()` selector within the following 16 bytes), so the guard
  resolves these tokens exactly as it resolves the genuine ones — shape-based,
  never codehash (the step-6 replicas embed a different beacon at the same
  shape).

This is a foundry (Solidity) project, not a cargo crate — the forwarder must BE
genuine-shape EVM bytecode, and delegatecall-into-wasm is unproven
(plan Revised note 4). The cargo workspace excludes this directory
(`contracts/Cargo.toml`).

## Layout

- `src/ForwarderPatch.sol` — the genuine runtime constant + shape scan +
  beacon patch. `test/MockPattern.t.sol` tripwires the constant against
  `spike/evidence/p_proxy.hex` (read-only fs permission).
- `src/RawHull.sol` — deploys a contract whose runtime is exactly the given
  bytes (EIP-5202-style hull; same trick as the spike's `RawDeploy`).
- `src/MockBeacon.sol`, `src/MockTokenImpl.sol` — the pattern pair.
- `script/DeployMockTokens.s.sol` — one shared beacon + shared impl v1
  (genuine layout: one shared impl across tokens), unwired v2 upgrade target,
  two pattern tokens (A, B), the plain token, funding + allowances +
  registration. `REGISTRY_ADDRESS`/`GUARD_ADDRESS` optional — skipped when
  unset (mock-only rehearsal on a bare anvil).
- `script/RunReceipts.s.sol` — the five red-flag receipts (byte-exact
  `Error(string)` expectations) + degraded settle + clean settle with balance
  assertions. Driven by `scripts/deploy/core/deploy.sh`, which asserts the
  ≤ 200k gas gate from the broadcast receipts and writes
  `deployments/<chain>.json`.

## Building / testing

Deps are pinned plain files (foundry.lock), never git submodules — forge
hoists submodule installs to the git root, which is wrong for a nested foundry
project (see contracts/replicas/README.md):

    cd contracts/core/mock-token
    git clone --depth 1 --branch v1.16.2 https://github.com/foundry-rs/forge-std lib/forge-std
    forge test

## Local rehearsal (no funds needed)

The mocks are plain EVM — they rehearse on a bare anvil:

    anvil &
    PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
      forge script script/DeployMockTokens.s.sol:DeployMockTokens \
      --rpc-url http://localhost:8545 --broadcast --slow

The guard/registry are Stylus wasm — the full receipts rehearsal needs a
stylus devnode (or funded testnet; `spike/DEPLOY.md` runbook).

## Verification (task 6)

`forge verify-contract` per contract: `--chain-id 421614` (Etherscan API v2 +
ETHERSCAN_API_KEY) on Arbitrum Sepolia; `--verifier blockscout --verifier-url
https://explorer.testnet.chain.robinhood.com/api` on 46630
(`.agent-workbench/knowledge/contract-verification.md`). Note: only MockBeacon
and MockTokenImpl are source-verifiable — the pattern forwarders are raw
283-byte runtime deploys (no Solidity source of their own), so they are
unverifiable by construction; their provenance is the ForwarderPatch tripwire
test against `spike/evidence/p_proxy.hex`.
