# Step 3 — contracts-core (Canonical Registry + Guarded Swap, Rust/Stylus)

status:     blocked
branch:     step-3-contracts-core
deployed:   not deployed — operator key 0x151e9f57F31310aFeBBB60c222c14badCf938E4C has balance 0 on 421614, 46630 and 4663 (cast balance, 2026-09-20); task 5/6 integration receipts stay deferred per the standing dispatch

## What was built
Two Stylus crates + the pinned events module, all committed by the previous builder (abccca2..c6b4ef9, re-grounded on main 7a1c744); this session committed the Cargo.lock crate entries (c37f4ad) and this file.

- `contracts/core/registry/` — single-registrar verification records (registrar set once + transfer fn), `verify`/`revoke`/`getRecord` (0x73c7cf61 / 0xafd0224b / 0x617fba04), `record.reason = keccak256(utf8(reason))` with full text in the Revoke event, Verify/Revoke events, 12 native tests.
- `contracts/core/guard/` — MM-fill escrow swap: `commit(tokenIn, tokenOut, amountIn, minOut)` escrows via transferFrom, no-arg `execute()` runs all five checks before any movement, then pulls the counterparty's fill (`transferFrom counterparty→buyer` for exactly `minOut`) and releases escrow (`amountIn`→counterparty). Beacon extracted from the forwarder bytecode SHAPE (PUSH32 `0x7f` + 12 zero bytes + 20-byte address, `implementation()` selector 0x5c60da1b within the next 16 bytes), impl resolved by STATICCALL to that beacon; extraction/probe failure DEGRADES (skips blocklist+impl checks, never reverts); `GUARD_IMPL_MISMATCH` only on successful resolution that mismatches. Pure `red_flag` decision table; 21 native tests, split per-staticcall. Five byte-exact strings `GUARD_NO_RECORD / GUARD_RECORD_REVOKED / GUARD_PAUSED / GUARD_BLOCKLISTED / GUARD_IMPL_MISMATCH` match abi.ts/journeys.md.
- `packages/shared/events.ts` (+ index.ts re-export + 3-test vitest tripwire) — REGISTRY_EVENT_SIGNATURES/TOPICS pinned; consumed by steps 4 (Revoke log extraction) and 5 (J3 reason rendering).
- Plan Revised notes (coordinator decisions on verify.md loose ends 1–8) and verify.md checked in at c6b4ef9.

Green locally, this session: `cargo test --workspace` in contracts/ (guard 21, registry 12, hello 2 — all pass), `cargo build --workspace --target wasm32-unknown-unknown` clean, `npm test` in packages/shared 18/18. No review has run yet.

## Where the plan was wrong
Nothing material. Verify.md loose ends 1–8 were all applied as pinned (MM-key no-arg execute; shape-based beacon extraction; events module in packages/shared; mock-token in scope; per-staticcall test split; Arbiscan/Etherscan-v2 for 421614 vs Blockscout for 46630; ≤200k asserted on settle AND each of the five revert receipts; README writer note landed with step 6). Revised note 4 makes mock-token a foundry project excluded from the cargo workspace — `contracts/Cargo.toml` carries the exclude.

## What the next step needs to know
- **Unfinished on-disk skeletons:** `contracts/core/mock-token/` (script/src/test) and `scripts/deploy/core/` exist as EMPTY untracked directories — zero files. Task 4's mock pattern tokens (genuine-shape forwarder from patched `spike/evidence/p_proxy.hex`, blocklist on the beacon, pause on the token) and task 5's deploy script + `deployments/{421614,46630}.json` are still to be written. Nothing is deployed anywhere from this step.
- **Integration runbook:** `spike/DEPLOY.md` — the single manual step is funding `0x151e9f57F…8E4C` (faucet links there; headless 46630 funding is documented impossible, `spike/evidence/funding_blockers_20260919.txt`); balance was 0 on all three chains on 2026-09-20. Task 5 (deploy to 421614+46630, five reverts + settle on-chain, ≤200k receipts, `cargo stylus verify` / `forge verify-contract`) runs only when funded; step 7 (hardening, 4663 deploy) and step 8 (e2e against scratch deployments) both need those addresses — the frontend live seam (`VITE_REGISTRY_ADDRESS`/`VITE_GUARD_ADDRESS`) stays dark until then.
- **Review handoff (the caller's dispatch must state):** branch `step-3-contracts-core`, ROOT `/home/bitslicer/projects/hackathon_arbitrum_singpapore`, step dir `.agent-workbench/step-3-contracts-core/`. Reviewer must check against the pinned decisions: (1) MM-key counterparty, `execute()` stays NO-ARG with dedicated non-GUARD_* internal errors; (2) beacon extracted from forwarder bytecode shape, impl via STATICCALL `implementation()`, extraction failure = skip never revert; (3) verify.md loose ends 3–8 as listed above; (4) sanctioned file scope = `contracts/core/**` (registry+guard; mock-token crate not yet written), `scripts/deploy/core/` (empty skeleton), `packages/shared/{events.ts,index.ts,tests/events.test.ts}` — no deployments appends exist yet.
- Escrow internal invariants (amountIn==0, double commit, execute without order) revert with dedicated non-`GUARD_*` errors — the five pinned strings stay reserved for the deterministic red flags.

## Out of scope, left broken
- `cargo build` emits warnings (8 in registry/guard lib targets, mostly dead-code/style) — step 7's quality pass owns cleanup; noted, not fixed here.
- `contracts/core/mock-token/` + `scripts/deploy/core/` skeletons are tracked by nobody — they vanish on a clean checkout until task 4/5 lands files in them.
