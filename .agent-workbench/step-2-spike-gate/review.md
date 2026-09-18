# Step 2 — spike gate: review (round 1, 2026-09-19)

Branch `step-2-spike-gate` @ 2a9fa46, base `main` (merge-base 1b76d63). Diff: 43 files,
all under `spike/**`. `packages/shared/abi.ts` untouched — `PROBE_SELECTORS` placeholders
intact, which is correct per the plan's "merge-time append" language (plan.md:21,35).

## Verified independently (not taken from builder claims)

- **9 native tests are real and green**: ran `CARGO_TARGET_DIR=/tmp/probe-host cargo test`
  in `spike/probe` — 9 passed, 0 failed, exit 0. Assertions are concrete: exact 7-field
  roundtrip values, slice clamp at code end (596..600 of a 600-byte code, len=100 → 4),
  three returndata shapes (true / empty / revert), revert → (ZERO, false) not a panic,
  suite bit-packing incl. code-size field, empty-mirror degradation bit-by-bit. The two
  tests weakened by the TestVM 0.10.9 quirk (`probe_resolve_composition_plumbing`,
  `suite_packs_expected_bits`) disclose the quirk inline and are backed by the per-call
  tests; not vacuous.
- **Calibration is genuinely live-verified**: I re-ran read-only RPC against
  `https://rpc.mainnet.chain.robinhood.com` myself — chainId 4663; P and CRM beacon slots
  → 0xe10b6f…1b00 (shared beacon); impl slot empty; `implementation()` → 0xb35490d6…c5ae2;
  P/CRM proxy codehashes byte-identical 0x6c1fdd40…5630; impl codehash 0xdc07e86e…eec7;
  beacon codehash 0x8b465c0b…e90e; P `uid()` 0x…2c4d1ce31ec4310b2c507c921a52b70 ==
  `/rhj/assets` registry id byte-exact; `paused()` false. Every value matches
  `evidence/calibration_4663.json` and the `.codehash`/`.hex` evidence files exactly.
- **Beacon-not-proxy blocklist finding REPRODUCED LIVE**: `isBlocked(buyer)` answers on
  the beacon (false), reverts with empty data on BOTH the proxy and the impl.
  findings.md:15 states the finding and the step-3 consequence (blocklist probe → beacon,
  paused probe → proxy) unambiguously; the probe contract itself (`probe_blocklist` takes
  the beacon), the DEPLOY.md cast commands, and step-6 guidance all bake it in. Step 3's
  guard probe design change is unambiguous and evidenced.
- **Selectors keccak-verified**: `paused()` 0x5c975abb, `isBlocked(address)` 0xfbac3951,
  `implementation()` 0x5c60da1b, `extsload(address,bytes32)` 0x25a130c3.
- **Activation log content**: builds the probe, metadata hash 3ab96a63…0359, contract
  12429 bytes, ArbWasm wasm data fee 0.000092 ETH quoted on both endpoints,
  `EXIT_4663=0` and `EXIT_421614=0` (stamped 2026-09-18T23:13). See blocking finding 1 —
  the file is not committed.
- **GoPlus evidence**: exactly 16 fields, `is_proxy=1`, `is_open_source=1`, every risk
  field absent (no is_honeypot / is_blacklisted / transfer_pausable) — matches findings
  and knowledge goplus-api.md. Impostor half correctly recorded as unanswerable.
- **EAS evidence**: canonical 421614 deployment artifact (EAS 0x2521021f…1dE), no 4663
  anywhere in it — "421614 only, absent on 4663" supported.
- **Canonical fetch**: live worker hit from here — `/health` → `{"ok":true}`; `/assets` →
  `{"ok":true,"total":194}` with P/CRM rows carrying contractAddress + chainId 4663.
  Degraded branch implemented (upstream failure / parse failure → 502 `degraded: …`);
  the worker has its own 2 native tests (parse failure path included).
- **Replica beat**: broadcast receipts (chain 412346 = local) show 10 txs incl. the
  beacon `upgradeTo` under a fixed proxy plus the RawDeploy canary; 6 foundry tests incl.
  pinning of the calibrated selector bytes and the EIP-1967 beacon-slot layout.
- **Scope**: nothing outside `spike/**`. No secrets tracked (`spike/.env` untracked;
  64-hex grep hits only public codehashes/addresses). Funding-blocker evidence
  (7 faucets probed, PNGs, balances) consistent with the PENDING story.

## Findings

[blocking] spike/evidence/stylus_check_4663_421614.log — exists on disk but is UNTRACKED, excluded by .gitignore:21 (`*.log`) — findings.md:1 and calibration_4663.json:94 cite it as the `evidence_file` for the activation claim, DEPLOY.md:31 references it, and commit 9e902d1's message claims it was added — a clone of this branch carries a GATE line whose core PROVEN evidence is a dead reference — fix: `git add -f spike/evidence/stylus_check_4663_421614.log` (or rename to `.txt`); contents are clean (build warnings, exit codes, no secrets).

[non-blocking] Dangling verify.md citations — no step-2 verify.md exists anywhere in `.agent-workbench/` (only step-1's, whose 10 loose ends do not match the numbering used): findings.md:23 cites "verify.md:14", DEPLOY.md:14 cites "verify.md loose end 5" (the 421614-gas PASS formula), findings.md:11/35 cite loose ends 2/3/7 — the substance of each is restated inline and I verified each directly, but the operator flipping the gate will chase a dead reference — restate the PASS-formula rule inside DEPLOY.md (one line) or regenerate the verifier's verify.md. Note: the dispatched "spot-check the 8 loose ends" could not be performed against the file because the file does not exist.

[non-blocking] Plan task 6 ran on local chain 412346, not 421614 as written — funding-driven and documented (findings.md:7, funding_blockers_20260919.txt); the upgrade-beat mechanics are proven by 10 broadcast receipts + 6 foundry tests; redoing it on 421614 is optional alongside the operator run, not a gate condition.

[non-blocking] spike/probe/src/lib.rs:130-136 — `probePaused` returns (true,false) both for "success, unpaused" and "success, empty returndata", so the "ok, nothing there" reading is indistinguishable from a live unpaused token at that single probe — documented as deliberate mirror semantics and `runSuite` bit 7 (code presence) disambiguates; nit only.

[non-blocking] Gate-day slip: spec.md:50 says the decision cannot slip past 2026-09-18; the activation log is stamped 09-18T23:13 and findings.md was written 09-19 (prior builder's usage-limit death). Process matter, recorded here only; the substantive deadline evidence (activation) made 09-18.

## What remains for the PASS flip (gas receipt = known operator-blocked item; not held against this review)

1. Fix the blocking finding above (track the stylus check log).
2. Fund 0x151e9f57F31310aFeBBB60c222c14badCf938E4C (DEPLOY.md §0).
3. `cargo stylus deploy` probe → 421614 + 4663, run the three probes live (DEPLOY.md §1; completes plan task 3).
4. `forge script Deploy` + `DeployCanary` on 4663 — records the 0x5c EXTSLOAD verdict for gate criterion (c) (DEPLOY.md §2).
5. `seedRecord` + `cast send runSuite` → `cast receipt .gasUsed` ≤ 200,000 (DEPLOY.md §3; plan task 4).
6. Flip findings.md line 1 to `GATE: PASS — Stylus everywhere` and paste actuals (DEPLOY.md §4).
7. Merge-time edit: `PROBE_SELECTORS` → `{ paused: "0x5c975abb", blocklist: "0xfbac3951" }` in packages/shared/abi.ts, shared vitest suite green.

## Verdict

GATE: PASS-PENDING-GAS is honest — findings.md:1 separates PROVEN (activation exit-0 on both endpoints, byte-exact live calibration, beacon-state finding, live worker, GoPlus/EAS) from PENDING (the ≤200k receipt, blocked solely on funding), and DEPLOY.md is a complete runbook whose expected values match my own live reads. One blocking finding: the activation log must actually be on the branch.
