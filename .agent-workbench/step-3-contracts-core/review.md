# Review — step-3-contracts-core (round 1, 2026-09-20)

Branch `step-3-contracts-core` @ 25e1c3f vs main 7a1c744. Reviewed per the plan
(+ Revised notes 1–8 as coordinator-pinned), verify.md loose ends, findings.md
Review handoff, and ground truth `packages/shared/abi.ts`, `spike/findings.md`,
`spike/evidence/calibration_4663.json`, `spike/evidence/p_proxy.hex`,
`.agent-workbench/step-7-contract-hardening/plan.md`.

## Verified clean

- **Tests run this session, all green:** `cargo test --workspace` → guard 21,
  registry 12, hello 2 (0 failed); `npm test` in packages/shared → 18/18
  (events 3 + roundtrip 15). Counts match the claimed 33 native.
- **Tests are not vacuous.** Registry: auth failures asserted WITH state
  unchanged after (lib.rs:310–333); Revoke event topic0 asserted against
  keccak AND full-text reason asserted in the log data (lib.rs:351–357);
  reinstatement clears revokedAt/reason (lib.rs:373–386). Guard: all five
  GUARD_* strings asserted byte-exact — the three record-family flags through
  the real `execute()` flow (lib.rs:811–863), the three probe flags through
  the pure `red_flag` table with exact (ok,value) inputs (lib.rs:925–984);
  decision ORDER asserted (revoked wins over probes, lib.rs:903–923); degrade
  table asserted (probes that cannot answer → None, lib.rs:968–984); selector
  tripwines recompute keccak against the abi.ts literals in BOTH suites
  (registry lib.rs:418–431, guard lib.rs:1034–1054); event topics recomputed
  in vitest via viem `toEventSelector` (events.test.ts:18–35).
- **Pinned decision 2 (beacon extraction) implemented as pinned:**
  shape-based, not codehash — PUSH32 `0x7f` + 12 zero bytes + 20-byte address,
  anchored by `5c60da1b` within the following 16 bytes (guard lib.rs:590–608).
  Test fixture is the genuine `p_proxy.hex` byte-for-byte (verified by diff;
  only a `0x` prefix differs) and the same-shape-different-beacon case
  (step-6 replica) plus three no-shape failures are asserted
  (lib.rs:989–1029). Impl via STATICCALL `implementation()` on the extracted
  beacon (lib.rs:336). Extraction failure ⇒ blocklist+impl checks skipped —
  `(false,_)`/None degrade to no-flag, never revert (lib.rs:333–340,
  496–523); GUARD_IMPL_MISMATCH only on successful resolution that mismatches.
  Registry staticcall failure is the deliberate exception (fail closed →
  GUARD_NO_RECORD, documented lib.rs:385–394).
- **Pinned decision 1 (escrow):** `execute()` no-arg, keyed on msg.sender's
  order; MM counterparty set at construction (zero rejected); commit escrows
  via transferFrom buyer→guard; execute pulls the fill (transferFrom
  counterparty→buyer for exactly minOut) then releases (transfer
  guard→counterparty for amountIn) — lib.rs:276–310, 357–381. All five checks
  run before any movement; every token-call error propagates (strict `false`
  → TokenTransferFailed, revert data carried, lib.rs:462–481), so EVM
  atomicity holds every revert path at zero movement. Internal invariants use
  dedicated non-GUARD_* errors (EscrowAmountZero/OrderActive/NoOrder,
  lib.rs:88–109); the five pinned strings are used only for red flags.
- **Signatures byte-identical to abi.ts:** record tuple order/status u8s
  match REGISTRY_ABI + types.ts; commit/execute/getRecord/verify/revoke
  selectors recomputed and pinned in both languages; GUARD_* strings
  byte-identical to abi.ts:40–46; PROBE_SELECTORS 0x5c975abb/0xfbac3951 used
  as constants with the calibrated targets (paused→proxy, isBlocked→beacon).
- **Pinned decision 3 (events):** `packages/shared/events.ts` new module,
  re-exported from index.ts; `record.reason = keccak256(utf8(reason))` with
  full text in Revoke (registry lib.rs:185–201); topic0 literals match the
  Rust `b256!` pins.
- **rustdoc on every public fn** in both crates, including the flow/decision
  rationale; `#[public]`-leak avoided by keeping helpers out of the ABI block.
- Wording ban respected (sole "attestation" hit is the doc explaining the
  ban). Diff is in sanctioned scope: contracts/core/{registry,guard},
  contracts manifests, packages/shared events module. The two untracked
  workbench verify.md files belong to steps 4/6 — ignored per dispatch.

## Findings

1. **[blocking] contracts/core/mock-token/ + scripts/deploy/core/ — planned
   deliverables never written.** `find` confirms zero files (git ignores the
   empty dirs); no `deployments/{421614,46630}.json` appends exist. Plan task
   4's mock pattern tokens (genuine-shape forwarder from patched p_proxy.hex,
   blocklist on the beacon, pause on the token) and task 5's `--chain` deploy
   script + deployments writers are unbuilt. The funding authorization covers
   the on-chain RUNS and receipts only — these are pure code and need no
   funds to write. Step 7 cannot own them: its plan CONSUMES them (task 5
   "4663 mainnet deploy of registry + guard via the same
   `scripts/deploy/core/`"; task 2 diffs gas against "step 3's integration
   receipts"; `learned:` expects step-3 "gas actuals, integration harness")
   while its scope is "contracts/core/** (tests + docs only)". **Ruling:**
   they block the merge; owner is step 3 — land them on this branch. Route to
   reconcile ONLY if the coordinator explicitly amends step-7's (or a
   reconcile step's) plan to name writing them; as both plans stand, nobody
   owns them and step 7 crashes into the gap mid-hardening.

2. **[non-blocking] Guard native tests cannot assert "no state change on all
   five revert paths" (plan task 4).** `execute()` deactivates the order
   before the checks (guard lib.rs:298) and the native harness does not roll
   back a top-level Err, so the zero-movement property rests on the EVM
   atomicity + never-swallowed-errors argument and on task 5's receipts —
   the sanctioned loose-end-5 split. Step 7's fuzz task re-asserts it; make
   sure the receipts eventually land.

3. **[non-blocking] Probe→target wiring has no assertion anywhere yet.**
   paused→proxy, isBlocked→beacon, implementation()→beacon and the
   tokenIn-then-tokenOut order are untested natively (stylus-test 0.10.9
   per-call-bytes quirk) and ride entirely on the deferred task-5 receipts.
   Recorded so the deferral is visible, per the same pinned split.

4. **[non-blocking] guard/src/lib.rs:588 — leftover `ponytail:` note marker**
   in a shipped doc comment; step 7's quality pass can drop it.

## Verdict

The built code (tasks 1–3, 7) is merge-quality and faithfully implements all
three pinned coordinator decisions; the tests genuinely assert the semantics.
The step is not complete: task 4 and task 5's code half are missing with no
owner under the current plans (finding 1). Fix = land the mock-token foundry
project + `scripts/deploy/core/` script + deployments writers on this branch,
or get an explicit re-scope recorded before merge.
