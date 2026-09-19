# Wire contract — the shared language of every Vetted component

Single source of truth for the data that crosses component boundaries:
frontend (step 5) ↔ backend (step 4) ↔ contracts (step 3). `types.ts` (TS) and
`types.rs` (Rust) implement this file and must round-trip the golden fixtures
in `fixtures/` byte-identically — CI enforces both directions.

Reading order for a new builder: this file, then `types.ts`, then `abi.ts`.

## Verdicts

`VERIFIED | IMPOSTOR | UNVERIFIED | REVOKED` (spec.md, journeys.md:14).

- The engine never guesses: missing evidence degrades to `UNVERIFIED`, visibly.
- `REVOKED` = formerly verified, record revoked since; links the revocation tx.
- Colors are fixed by theme: green `VERIFIED`, red `IMPOSTOR`/`REVOKED`, amber `UNVERIFIED`.
- "Not a contract" is NOT a verdict — it is the `NOT_CONTRACT` terminal state
  (no verdict attempted, `verdict: null`).

## Scan API

Request — query params (step-4 plan):

```
GET /scan?chainId=<number>&addr=<0x-hex address>
```

Response — `ScanResponse` (`types.ts` / `types.rs`):

- `verdict` — one of the four, or `null` when `terminalState` is set.
- `terminalState` — `NOT_CONTRACT` (no code at the address) or `RPC_RETRYABLE`
  (RPC failure; nothing guessed, caller may retry).
- `degraded` — issuer canonical list unreachable: ground truth is down, every
  verdict drops to UNVERIFIED and this flag is true (journeys.md:20).
- `notice` — non-4663 chains scan read-only; stock-token verdicts exist only on
  4663, the notice says so (journeys.md:19).
- `powerReport[]` — one row per power check; every row carries `evidenceUrl`
  (tx / slot / bytecode diff / probe result — spec.md:30).
- `record` — the registry record when one exists (shape below).
- `revocationTx` — tx hash, set for REVOKED verdicts (journeys.md:14).

## Registry record

On-chain shape (step 3): `{u8 status, u256 riskFlags, u64 verifiedAt,
address impl, address registrar, u64 revokedAt, bytes32 reason}`.

Wire (JSON) encoding:

| field | on-chain | JSON | notes |
|---|---|---|---|
| `status` | u8 | string | `0 → "VERIFIED"`, `1 → "REVOKED"`; unknown values must be treated as no-record, never guessed |
| `riskFlags` | u256 | decimal string | JSON has no u256; parse with `BigInt` |
| `verifiedAt` | u64 | number | unix seconds |
| `impl` | address | 0x-hex string | implementation pointer |
| `registrar` | address | 0x-hex string | |
| `revokedAt` | u64 | number | unix seconds; `0` while VERIFIED |
| `reason` | bytes32 | 0x-hex string | zero value while VERIFIED |

## Guard revert reasons

Verbatim strings — the guard's deterministic red flags, surfaced exactly as
reverted (journeys.md:32):

```
GUARD_NO_RECORD | GUARD_RECORD_REVOKED | GUARD_PAUSED | GUARD_BLOCKLISTED | GUARD_IMPL_MISMATCH
```

Heuristic/`riskFlags` warnings never revert (spec.md:28). The strings are the
ABI-level revert data; `abi.ts` exports them as constants.

## Registry read ABI

`abi.ts` — human-readable signatures + exported 4-byte selector constants,
pinned before any contract exists; step 3 must match byte-for-byte:

- `getRecord(address token)` — the J3 primitive: callable from other contracts.
- `verify(address token, uint256 riskFlags, address impl)` — registrar-only.
- `revoke(address token, string reason)` — registrar-only.
- `commit(address tokenIn, address tokenOut, uint256 amountIn, uint256 minOut)` / `execute()` — guarded swap.

`PROBE_SELECTORS` (`paused()` + per-address blocklist probe) is a reserved
block of placeholder bytes that step 2 replaces with its live-calibrated ones
at merge — consumers read the constant, never the placeholder value.

## Chains

| id | role | RPC | verified |
|---|---|---|---|
| 4663 | Robinhood Chain mainnet — verdicts, swaps, demo | `https://rpc.mainnet.chain.robinhood.com` | `eth_chainId → 0x1237`, 2026-09-12 |
| 46630 | Robinhood Chain testnet — scratch deploys | `https://rpc.testnet.chain.robinhood.com` | URL from the official docs bundle (docs.robinhood.com/chain/connecting assets); `eth_chainId → 0xb626`, 2026-09-12 |
| 421614 | Arbitrum Sepolia — Stylus mirror | `https://sepolia-rollup.arbitrum.io/rpc` | `eth_chainId → 0x66eee`, 2026-09-12 |

4663 is the journeys' default network; scans on other chains are read-only.

## Conventions

- JSON files: 2-space pretty, keys in interface order, trailing newline — the
  fixtures ARE the canonical form; round-trip tests compare bytes.
- Addresses lowercase `0x` + 40 hex; hashes `0x` + 64 hex.
- Append points: `index.ts` / `src/lib.rs` re-export; later steps add module
  files and re-export them — never edit another step's module.
- Wording discipline: Canonical Registry / verification record / registrar —
  never "attestation" (spec.md:35).

## Watchdog API

`GET /watchdog?chainId=<number>` → `WatchdogStats` (`watchdog.ts` /
`watchdog.rs`, added step 4; step 5's local declaration in
`frontend/src/lib/api.ts` moves here at step 8's reconciliation):

- `chainId` — echoed back.
- `runs` — cumulative sequencer-filterer runs, ONE RPC read upstream
  (spec.md:37). `runs: 0` WITH a `provenanceUrl` means the live count is
  UNAVAILABLE (the degrade path) — never a measured zero on a launched chain.
- `baselinePerDay` — the published 6-week baseline as a daily rate (~150/day,
  measured 2026-08). Never recomputed, never stored.
- `provenanceUrl` — where the numbers come from: the live counter's public
  entry, or the published-baseline source when degraded; `null` = no
  provenance at all.

No stored history anywhere (spec.md:37) — the endpoint is stateless and the
registry/state carries nothing for it.
