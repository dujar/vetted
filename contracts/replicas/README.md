# contracts/replicas — demo asset set (step 6)

Faithful replica of the genuine Robinhood stock-token pattern (beacon proxy +
hidden per-address blocklist ON THE BEACON + global pause) plus two impostor
twins, for the 5-minute demo's beats. Deliberately isolated from the Rust
core (`contracts/core/` is a separate cargo workspace — never imports this).
Ground truth: `spike/evidence/calibration_4663.json`, `spike/findings.md`.
Pattern check list: `FINGERPRINT.md` (implemented by `test/Fingerprint.t.sol`,
mirrored by step 4's matcher). Manifest of live addresses: `demo/assets.md`.

## Layout

- `src/DemoBeacon.sol` — the GENUINE beacon shape: AccessControl + `implementation()`
  /`upgradeTo()` + `paused()`/`pause()`/`unpause()` + `isBlocked()`/`setBlocked()`.
  The hidden demo state lives HERE, not in the token.
- `src/DemoBeaconProxy.sol` — forwarder embedding the beacon as a runtime
  immediate (PUSH32, genuine `p_proxy.hex` shape), STATICCALL `implementation()`,
  DELEGATECALL result; writes the EIP-1967 beacon slot once in the constructor.
  Resolution is by slot or bytecode SHAPE, never codehash (see FINGERPRINT.md).
- `src/DemoStockToken.sol` — impl: ERC-20 (18 decimals), reads pause/blocklist
  from the resolved beacon inside the transfer hook; `isBlocked` reverts through
  the proxy (calibrated behavior). Deployed twice: v1 + upgrade target.
- `src/ImpostorPlain.sol` — twin 1: the naive anti-pattern copy (pause+blocklist
  in-token), fabricated `uid()`.
- `src/ImpostorSelfProxy.sol` — twin 2: EIP-1967 IMPL-slot proxy (wrong slot),
  `paused()` answers on the proxy, no blocklist anywhere.
- `src/CastNames.sol` — shared name/symbol; the twins' mimicry is the point.

## Building / testing

Deps are pinned by `foundry.lock` + `.gitmodules` (plain files — NOT git
submodules: forge hoists submodule installs to the git root, which is wrong
for a nested project; `forge install` is a silent no-op here). Restore with
plain clones, then `forge test`:

```sh
cd contracts/replicas
git clone --depth 1 --branch v1.16.2 https://github.com/foundry-rs/forge-std lib/forge-std
git clone --depth 1 --branch v5.4.0 https://github.com/OpenZeppelin/openzeppelin-contracts-upgradeable lib/openzeppelin-contracts-upgradeable
git clone --depth 1 --branch v5.4.0 https://github.com/OpenZeppelin/openzeppelin-contracts lib/openzeppelin-contracts-upgradeable/lib/openzeppelin-contracts
forge test
```

Never commit `lib/` (gitignored). CI runs the same clones (`.github/workflows/ci.yml`,
`replicas` job).

## Deploying

`DEPLOY_KEY` (pk) + `RPC_URL`, then `scripts/deploy/replicas/deploy.sh` —
runbook, per-chain explorer verify commands and the beat choreography live in
`demo/assets.md`.
