# Step 2 verify — spike-gate plan (2026-09-12)

## Hard gates

- UI without a mockup: does not trip — step builds no screen or visual component (plan: "Screens N/A"; screenshots are slide artifacts, not UI).
- Missing user journey: does not trip — verification work, no navigable surface ("User journey: N/A" is honest).

## Environment checks the dispatch asked for (run, not assumed)

- cargo-stylus was NOT installed on this box before today. `scripts/toolchain.sh` installs it correctly from crates.io (0.10.9 downloaded, compiled in 2m, "Installed package cargo-stylus v0.10.9") — but then the script FAILS: see loose end 1. cargo-stylus 0.10.9 is now installed at ~/.cargo/bin as a side effect of this verification.
- Docker: not needed for the spike. Per knowledge stylus-toolchain.md:27–30 + :34, `cargo stylus check`/`deploy` need no Docker — Docker is only the reproducible-build/verify path, which this step does not do. (Docker is usable on this box anyway: `docker info` OK.)
- wasm32-unknown-unknown target installed; forge 1.8.1 + cast present for the Solidity replica smoke.
- Gate criterion (b) is real against the pinned SDK: `Host::code()/code_size()/code_hash()` exist in stylus-sdk 0.10.9 (`~/.cargo/registry/src/.../stylus-sdk-0.10.9/src/host/mod.rs:212–220`; raw `account_code` hostio at hostio.rs:74). Remaining unknown is exactly what the gate tests: ArbWasm activation on 4663.
- PROBE_SELECTORS merge is safe: `packages/shared/tests/roundtrip.test.ts:110–113` asserts only the `/^0x[0-9a-f]{8}$/` format, not the placeholder values — calibrated bytes keep it green. Plan's CI claim (shared job round-trips + recomputes SELECTORS) matches step-1 findings.
- Reference audit — all resolve: P `0x1Cdad396…00b7D` and CRM `0xd95B4412…36D44` (knowledge robinhood-stock-tokens.md:23–25, live /rhj/assets); GoPlus underscore endpoint (goplus-api.md:11–17); portal-bridge URL (robinhood-chain.md:23); `cargo stylus check/deploy` two-tx flow (stylus-toolchain.md:27–30); state.md:32–33 (Q7 rationale + GoPlus pre-answer) and journeys.md:20 (degraded banner) are exactly the cited lines; wire.md:89–90 + frontend/src/lib/chains.ts carry the live-probed 4663/46630 RPCs; contracts/core/hello/src/lib.rs has the template header revision note 1 says to copy; scan-backend/wrangler.toml exists. No CLAUDE.md/AGENTS.md at repo root. Steps 3/4 plans correctly gate their start on step-2 findings.md; no file-scope collision (spike/** is step-2-only; abi.ts PROBE_SELECTORS block is reserved and consumers read the exported constant).

## Loose ends

1. [missing prerequisite] scripts/toolchain.sh exits 2 on the success path
   where:    scripts/toolchain.sh:14,19 — first thing a step-2 builder runs ("Stack: same as step 1")
   evidence: verified by running it 2026-09-12: install succeeds, but `cargo-stylus --version` is not a valid invocation of 0.10.9 — prints help, exit 2; only `cargo stylus -V` works (prints "stylus 0.10.9", exit 0). Line 19's bare command therefore fails under `set -euo pipefail`; line 14's grep check also never matches, so every run takes the reinstall path.
   fix:      use `cargo stylus -V` in both lines (check and final echo). One-line edit; without it a builder sees a "failed" bootstrap on a working toolchain.

2. [unhandled call site] Task 3 success criterion is unmeetable verbatim on 421614
   where:    plan task 3 — "all three return expected values on both chains"
   evidence: P and CRM exist only on 4663 (robinhood-stock-tokens.md deployments all carry chainId 4663); on 421614 the probe targets are empty accounts — paused() staticcall succeeds with empty returndata (nothing to decode), EIP-1967/beacon slot reads return 0x0, beacon.implementation() has no target.
   fix:      define the 421614 targets: either reorder task 6's replica smoke before task 3 and point the 421614 probes at the replica, or state the expected 421614 behavior (empty-code target → success with empty returndata, zero slots) in the check.

3. [orphan output] Task 4's "record read" has no producer
   where:    plan task 4 — probe suite = "record read + paused + blocklist + impl-vs-record"
   evidence: the registry ships in step 3; nothing in the spike produces a record to read, so the implementer must guess what "record read" measures.
   fix:      one sentence — stub the seven-field record (the getRecord shape in packages/shared/abi.ts) in the probe contract's sol_storage and measure reading it.

4. [deferred decision] Faucet failure has no terminal branch
   where:    plan task 1 / open questions
   evidence: knowledge arbitrum-sepolia.md:23–24 (and the plan itself) warn faucets "typically require a mainnet balance threshold or an account login". If none dispenses, 421614 — the designated fallback scratch env for 46630 AND the deploy mirror for tasks 3/6 — is unfunded and tasks 3–6 stall. The 46630 and 4663 branches each have a named fallback/escalation; this one has none.
   fix:      add the missing branch: if no faucet dispensers by day 2 (Sep 14), escalate to coordinator with the same two options as 4663 (fund from a holder / de-scope).

5. [deferred decision] The 4663-unfunded world never says how the GATE line gets written
   where:    plan open question 1 vs task 10
   evidence: spec.md:51 PASS requires deploying the probe on 4663; the open question's fallbacks ("another holder" / "de-scope to testnet-only, needs a spec amendment") don't state what findings.md writes on Sep 18 if 4663 gas never materializes. `cargo stylus check --endpoint https://rpc.mainnet.chain.robinhood.com` validates deploy AND activation on 4663 without any funds (stylus-toolchain.md:27–28) — i.e. the activation half of the gate is decidable unfunded.
   fix:      pin the degraded gate formula in the open question: GATE decided by check-passed-on-4663 (activation proof) + all probes/gas measured on the funded chain, recorded with that caveat on the GATE line.

6. [orphan output] "EAS record" screenshot has no producing task
   where:    plan line 42 check — "two screenshots (GoPlus, EAS record)"; task 8
   evidence: task 8 says "no on-chain probe needed" and the project mints no attestations (bespoke registry is the product); easscan.org DNS-fails from this environment (eas.md:20–22). Nothing in any task creates an EAS record to screenshot.
   fix:      define the subject: either mint one throwaway attestation on 421614 with the task-1 key (the optional staticcall sanity check plus a real record), or redefine the screenshot as the canonical GitHub deployments/arbitrum-sepolia/EAS.json + staticcall output.

7. [untouched sibling] Task 9 silently drops the impostor half of the GoPlus re-test
   where:    plan task 9 vs spec.md:52
   evidence: spec.md:52 re-tests "genuine stock tokens + the known impostor on 4663"; task 9 queries only P, and no impostor address exists in any evidence file (grep over product/ + knowledge/ finds only the name "Hoodrat • Robinhood Token", never an address). The narrowing is probably right — the copy branch is already decided — but it is unmarked.
   fix:      one sentence in task 9: impostor address is unrecorded anywhere, so the re-run covers the genuine token only and findings.md notes the impostor half as unanswerable at spike time (or finds one live).

8. [untouched sibling] Task 6 replica smoke moves chains without marking the divergence
   where:    plan task 6 vs spec.md:54
   evidence: spec.md:54 pins the spike replica smoke to "on 4663"; task 6 runs on 421614 "(and 46630 if funded)" — 4663 never appears. After task 3 the funded key holds 4663 gas, so covering 4663 is nearly free and it is the chain the demo beat runs on.
   fix:      add "and 4663 if the task-3 probe deploy succeeded" to task 6, or mark the 421614-only choice deliberate in the task text.

## Bottom line

Buildable direction, everything referenced is real, and the two genuinely load-bearing unknowns (ArbWasm activation on 4663, faucet reality) are correctly framed as day-1 verifications. The 8 loose ends are all one-line fixes except none — no structural rework needed. Loose end 1 should land before the builder starts or the first bootstrap "fails".
