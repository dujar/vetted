# Step 3 — Core contracts v1: Canonical Registry + Guarded Swap (Rust/Stylus)

## Resources
- spec:    ../product/spec.md   (Scope 2–3 spec.md:35–36; verdict discipline spec.md:25–30; wording ban spec.md:35; gas gate spec.md:51)
- journeys: ../product/journeys.md  (J2 revert reasons verbatim journeys.md:32; J3 contract-callable read interface journeys.md:43)
- knows:   ../knowledge/stylus-toolchain.md, ../knowledge/robinhood-chain.md, ../knowledge/arbitrum-sepolia.md, ../knowledge/contract-verification.md  (Arbitrum Sepolia = Etherscan v2 `chainid=421614` :27–38 — the FAIL branch's `forge verify-contract` needs this; per-chain v1 endpoints are dead)
- learned: ../step-2-spike-gate/findings.md  (GATE decision, calibrated fingerprint + probe selectors, gas actuals — this step must not start until step 2's GATE line exists)

## Stack
Rust/Stylus (stylus-sdk 0.10.9) per spec.md:49.
**GATE:FAIL branch** → this step is rewritten same-day in Solidity: identical scope, identical revert strings, identical tests; foundry replaces cargo-stylus. Steps 4–6 are unaffected — the ABI and revert reasons are pinned in `packages/shared` (step 1), which is the contract every consumer builds against.

## Goal
After this step, the Canonical Registry and the Guarded Swap are deployed, tested and source-verified on Arbitrum Sepolia + scratch: registrar-only verification records the backend can revoke on beacon drift, and an escrow swap that reverts on exactly the five deterministic red flags at execution time — the enforcement no incumbent has (spec.md:14).

## Scope
- Creates: `contracts/core/registry/`, `contracts/core/guard/` (stylus crates), `scripts/deploy/core/` (one script, `--chain` flag for 4663/46630/421614), `deployments/{421614,46630}.json` (append-only; 4663 lands in step 7).
- Shared append points: `deployments/*.json`, `packages/shared/` (add a module file only if a type is genuinely missing).
- Out of scope: replica tokens (step 6), backend/frontend code, 4663 mainnet deploy (step 7 hardens first, step 10 freezes).

## User journey
N/A — on-chain contracts; consumed via J2 (swap) and J3 (registry read interface).

## Screens
N/A — no UI surface.

## Tasks
1. `contracts/core/registry/`: record {u8 status, u256 riskFlags, u64 verifiedAt, address impl, address registrar, u64 revokedAt, bytes32 reason}; **single-registrar writes** (`msg::sender()` check; registrar set once, with a registrar-transfer fn for recovery); `verify(token, …)` / `revoke(token, reason)` / `getRecord(token)` public read — the J3 primitive a wallet contract can call (journeys.md:43); Verify/Revoke events. Wording discipline everywhere: Canonical Registry / verification record / registrar — **never "attestation"** (spec.md:35).
2. Registry semantics (spec Scope 5 implies it): an ACTIVE registry record IS the product's canonical verification — the registrar writes it after checking the issuer's live list; the contract stores and exposes, it does not judge. Rule-engine use of records is step 4's business.
3. `contracts/core/guard/`: escrow-style guarded swap — `commit(tokenIn, tokenOut, amountIn, minOut)` escrows the user's tokenIn; `execute()` runs checks on BOTH tokens before any movement; the demo counterparty (project MM key) fills by transferring tokenOut, then escrow releases. Checks in fixed order, deterministic reverts, **exact strings** (journeys.md:32): record missing → `GUARD_NO_RECORD`; status REVOKED → `GUARD_RECORD_REVOKED`; staticcall `paused()` true → `GUARD_PAUSED`; per-address blocklist probe on buyer true → `GUARD_BLOCKLISTED`; resolved impl (EIP-1967 slot, else beacon slot + `beacon.implementation()`) ≠ record.impl → `GUARD_IMPL_MISMATCH`. Probe mechanics from step-2 findings (calibrated selectors). Heuristics never revert (spec.md:28); every revert path moves zero funds.
4. Unit tests in CI (stylus native test host): non-registrar writes revert; status transitions; every guard reason byte-exact; settle-path accounting; no state change on all five revert paths.
5. Integration (scripted, throwaway key): deploy registry + guard + mock pattern tokens to 421614 + 46630 via `scripts/deploy/core/`; run all five revert reasons + one settle ON-CHAIN; assert the full probe suite **≤200,000 gas** from receipts (spec.md:51; compare step-2 actuals); write `deployments/<chain>.json`.
6. `cargo stylus verify` every deploy (≥0.10.8 pinned; Docker reproducible-build path if the hash mismatches — knowledge stylus-toolchain.md:33–36; explorer endpoints/footguns in knowledge/contract-verification.md — the GATE:FAIL branch verifies with `forge verify-contract` using its Blockscout/Etherscan-v2 recipes). Check: verified pages on both Blockscouts.
7. rustdoc on every public fn — input to step 7's quality pass.

Check: `cargo test` green in CI; integration output shows five reverts + settle + gas numbers on 421614; verified pages linked from deployments/.

## Open questions
- Escrow counterparty model: project-funded MM key fills the demo swap (simplest faithful escrow). If the coordinator prefers a two-party order shape, only `execute` plumbing changes — decide before task 3.
- GATE:FAIL → rewrite in Solidity the same day and flag the coordinator in the builder report (tracker row stays step 3).
