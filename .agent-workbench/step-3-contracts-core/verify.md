# Verify — step 3 contracts-core (2026-09-19)

## Hard gates

- UI without a mockup: not tripped — Screens N/A, no UI surface (plan.md:42).
- Missing user journey: not tripped — on-chain contracts consumed via J2/J3; "N/A" is the correct call for this artifact type.

## Checked clean

- All five pinned selectors recomputed with viem from the abi.ts signatures — byte-exact: getRecord 0x617fba04, verify 0x73c7cf61, revoke 0xafd0224b, commit 0x498ab631, execute 0x61461954; PROBE_SELECTORS paused 0x5c975abb / blocklist 0xfbac3951 also verify. Plan task 1/3 signatures match `packages/shared/abi.ts` and `packages/shared/wire.md` byte-for-byte; record shape matches `types.ts` RegistryRecord (7 fields, same order).
- GUARD_* strings: plan task 3 = journeys.md:32 = abi.ts:40-46 = wire.md = fixtures/guard/*.json, byte-identical.
- Referenced artefacts exist: `spike/probe/` (resolve_impl / probe_blocklist / run_suite, 9 native tests), `spike/replica/script/DeployCanary.s.sol`, knowledge lines cited (stylus-toolchain.md:33-36, contract-verification.md:27-38), merge commits 9d8dc30/5e2ff35/e37bc15 in history.
- File-scope collisions with parallel steps 4/6: none. Step 3 writes contracts/core/{registry,guard}, scripts/deploy/core/, deployments/{421614,46630}.json; step 4 = scan-backend/** only; step 6 = contracts/replicas/**, scripts/deploy/replicas/, demo/assets.md. Workspace glob `core/*` picks up the new crates without edits; `contracts/Cargo.toml` comment already excludes replicas. CI contracts job (`.github/workflows/ci.yml:13-22`) runs wasm build + native `cargo test --workspace` and will run the new crates as-is (rust-toolchain.toml pins stable + wasm32).
- Revised notes 1–6 all reconcile with merged reality (hello crate header, workspace glob, wire.md chains table, deployments writers, step-5 env seam).

## Loose ends

1. [Deferred decision] Escrow counterparty model — UNDECIDED, flag the coordinator
  where:    plan.md:56 (Open questions: "decide before task 3")
  evidence: grep for escrow/counterparty/MM-key across .agent-workbench (plans, reconciliation/, plan-judgment.md, tracker) hits nothing but this open question; no decision recorded anywhere. Blast radius is bigger than "only execute plumbing changes": merged frontend already hardcodes the no-arg two-tx flow (`frontend/src/lib/wallet.ts:49-59` — commit(...) then execute() with `args: []`), and abi.ts pins `execute()` → selector 0x61461954; a two-party order shape would touch step-1's abi.ts, step-5's shipped code, and the shared fixtures.
  fix: coordinator states the decision (project MM key fills; execute() stays no-arg) as a one-line Revised note before task 3 starts.

2. [Missing prerequisite] Guard's from-contract beacon resolution is impossible as written
  where:    plan task 3 + Revised note 3 ("impl resolution is: beacon slot + the beacon's implementation()")
  evidence: reading the TOKEN's beacon slot from-contract is a remote-storage read — impossible on stock EVMs, no EXTSLOAD (0x5c unshipped) and no remote-storage hostio in Stylus (`spike/findings.md:17`, the very findings the plan cites); the extsload-helper path exists only on chains that ship 0x5c. The spike's own on-chain suite never resolved it — it takes the beacon as a call arg (`spike/DEPLOY.md` §3 `runSuite(address,address,address,address)`). The genuine proxies carry the beacon EMBEDDED in their 283-byte forwarder bytecode (`spike/evidence/calibration_4663.json`: "embedded fixed beacon address 0xe10b6f…1b00 + STATICCALL implementation()"), and from-contract CODE reads are proven (`spike/probe` codeBytes).
  fix: before task 3, pin the from-contract beacon source — extract the embedded beacon from the calibrated forwarder bytecode (offset/shape-based, NOT codehash byte-equality: the step-6 replica embeds a different beacon, and codehash-equality extraction would also collide with the step-4/6 joint fingerprint decision) — or carry the beacon in guard config / record. Then make task 4/5 mocks mirror that mechanism. Without this, GUARD_IMPL_MISMATCH — the demo's upgrade→revocation beat — cannot be built on 421614/46630/4663.

3. [Dangling reference] Verify/Revoke event signatures named but pinned nowhere
  where:    plan task 1 ("Verify/Revoke events"); consumers in steps 4/5/8
  evidence: `packages/shared/abi.ts` — the pinned surface steps build against — contains only functions; no event signatures anywhere in packages/shared. Step 4 Revised note 5 extracts the revocation tx from "step 3's registry Revoke event logs"; step 5 renders it (step-5 findings "Out of scope": live REVOKED shows "not indexed yet" until step 3 emits Revoked events). Also unspecified: revoke() takes `string reason` but record.reason is bytes32 (wire.md) — the string→bytes32 rule (hash? truncate?) is left to the implementer, while journeys.md:42 shows a human-readable reason in J3.
  fix: pin both event signatures in abi.ts (Scope explicitly allows adding a module file) and extend task 1's byte-for-byte note to them; state the conversion rule (e.g. keccak into record.reason, full text carried in the Revoke event — which also serves J3's readable reason).

4. [Missing prerequisite] Mock pattern tokens required by tasks 4 and 5 but in no Creates entry
  where:    plan Scope (plan.md:34) vs tasks 4/5 ("Mock tokens mirror the genuine layout", "deploy registry + guard + mock pattern tokens to 421614 + 46630")
  evidence: Scope creates only registry/, guard/, scripts/deploy/core/, deployments JSONs. Task 5 deploys mock tokens on-chain, so they need a deployable crate inside the workspace glob (affecting task 6's "verify every deploy" scope); no path is named anywhere.
  fix: add the mock-token crate path (e.g. `contracts/core/mock-token/`) to Scope and say whether task 6 verifies it.

5. [No check] Task 4's native test matrix collides with the stylus-test 0.10.9 mock quirk
  where:    plan task 4 ("every guard reason byte-exact; settle-path accounting; no state change on all five revert paths" in the stylus native test host)
  evidence: in native tests every staticcall in one tx returns the BYTES of the LAST-registered mock — only the returndata LENGTH is per-call (`spike/probe/src/lib.rs:416-422` and :438-442; `spike/findings.md:19`). execute() is a ≥4-staticcall flow (registry getRecord, paused on proxy, beacon resolution, blocklist on beacon), so byte-faithful full-flow assertions cannot run natively in one tx.
  fix: task 4 should explicitly split tests per-staticcall (the spike's documented pattern) and delegate full-flow byte-exactness + settle accounting to task 5's on-chain receipts.

6. [Knowledge contradiction] Task 6 check "verified pages on both Blockscouts" is wrong for 421614
  where:    plan task 6 check line (plan.md:50)
  evidence: Arbitrum Sepolia verifies via Etherscan API v2 / Arbiscan, not Blockscout (`knowledge/contract-verification.md:27-33` — chainid=421614, ETHERSCAN_API_KEY required; :38-39 — Etherscan v2 has no Robinhood chains, Blockscout-only there). The plan's own `knows:` citation (:27-38) says exactly this.
  fix: reword to "verified pages on Arbiscan (421614, Etherscan v2) + explorer.testnet.chain.robinhood.com (46630, Blockscout)".

7. [Deferred decision] Task 5's ≤200k gas assertion doesn't say WHAT is measured
  where:    plan task 5 ("assert the full probe suite ≤200,000 gas from receipts")
  evidence: spec.md:51 budgets "the full probe suite" (spike runSuite = 4 reads + 3 staticcalls + 2 code hostios, no token transfers — `spike/findings.md:12`); the guard's execute() receipt additionally carries the escrow transferFrom + both release transfers. A 220k execute() receipt would be arguable mid-build: is the swap mechanic inside the budget or not?
  fix: one line in task 5: assert the settle execute() receipt (and each revert receipt) gasUsed ≤ 200,000, and additionally report the probes-only figure if the total exceeds budget — or get the coordinator to pick the composition.

8. [Untouched sibling] deployments/README.md writer column not updated for step 6 appends
  where:    `deployments/README.md:11-12` (writers of 421614.json / 46630.json listed as "3" only) vs plan Revised note 4 and step-6 plan Revised note 2 ("step 6 appends replica fields to the same files")
  evidence: the append-conflict policy lives in the tracker (Next §4) and both plans, but the repo's own README — the file a builder actually reads — still says sole writer 3.
  fix: one-line README note that step 6 appends replica fields to the same files (plan-agent or step 3 at merge).

## Verdict

Buildable after the pre-task-3 decisions; no hard gate tripped. Loose ends 1 and 2 need the coordinator (or a Revised note) BEFORE task 3 — 2 is the one that breaks the guard's core check as written. 3–8 are one-line plan/README fixes that can land with the build.
