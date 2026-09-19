# Step 4 review — round 1 (2026-09-20)

Reviewer ran on worktree /tmp/step4-wt, branch step-4-scan-backend (66017a7 + 4bcabe7) vs BASE main 7a1c744. Diff is non-empty (31 files). All tests re-run on the branch: scan-backend cargo test 42/42 (37 unit + 5 golden), shared Rust 4/4, shared TS vitest 18/18, `cargo check --target wasm32-unknown-unknown` clean.

## Scope check (builder item 2)

Changed files = scan-backend/** + exactly the verify-prescribed shared edits (watchdog.ts/.rs, wire.md append, index.ts/src/lib.rs appends, fixtures/watchdog × 2, both round-trip extensions, ci.yml wasm32 check) + workbench docs — with ONE exception, finding B2 below. Nothing in frontend/, contracts/, scripts/deploy, or demo/.

## Builder-item judgments

1. **8 loose ends: all applied, spot-checked.** LE1 wrangler REGISTRY_ADDRESS_* + record:null degrade (wrangler.toml [vars], lib.rs config/registry_for); LE2 topic0s pinned Rust-side (hexutil.rs events) + live revocationTx extraction (lib.rs:251–262) — exceeds the prescribe-null fix, behavior verified against step-3's events.ts byte-for-byte; LE3 L1_RPC_URL + nullable provenanceUrl + degrade runs value (wrangler.toml, wire.md, watchdog.rs); LE4 registrar-cli fallback never triggered, recorded in findings; LE5 scope honored via shared append, worker.json deferral disclosed; LE6 fixtures/watchdog + both round-trip extensions; LE7 ci.yml wasm32 gate; LE8 selector consts + keccak tripwire tests (hexutil.rs tests).
3. **Fingerprint composition: SOUND.** Shipped S1–S5 (fingerprint.rs) = F2/F3/F4 of step-6's joint FINGERPRINT.md list; implements step-3's shape-based beacon extraction (PUSH32/PUSH20 scan-back, slot cross-check, slot fallback); NOT codehash byte-equality. Live P/CRM pass (fixture codehashes asserted == calibration evidence), step-6 replica passes (F3 shape embed + slot fallback), twins miss (no beacon candidates — tested). Pattern-match alone never grants a verdict (rules.rs gates VERIFIED on registry record or issuer listing + uid). See note N3 re F1/F5.
4. **Watchdog escape: pre-committed precisely enough; ships as-is.** Task 5 ("Degrade: baseline + provenance link if the count doesn't reconcile") + Open questions ("ship the degrade path (baseline + provenance) and flag the coordinator") + verify LE3 pinned the exact degrade shape (nullable provenanceUrl, degrade value of runs, L1_RPC_URL var). Shipped: runs:0 + provenanceUrl + baseline 150, sentinel documented in wire.md, unambiguous on a launched chain. COORDINATOR FLAG endorsed (see N4).
5. **Signing: recipe followed verbatim.** signer.rs flow + imports match rust-wasm-signing.md exactly; Cargo.toml pins are the verified 2.4.2 set, k256 0.13, getrandom 0.2/js. Env-only key (REGISTRAR_KEY secret, commented in wrangler.toml); the only committed key is the labeled throwaway test key. Nothing key-like committed.
6. **Wire contract: route-for-route match.** `GET /scan?chainId=&addr=` and `GET /watchdog?chainId=` (api.ts:50,72) vs routes.rs — exact, param spellings pinned by test. ScanResponse/PowerReportRow/RegistryRecord serde matches types.ts (round-trip tests green); decode_record matches abi.ts's 7-field getRecord tuple incl. bytes32 reason.

## Findings

```
[blocking] scan-backend/src/signer.rs:23 — abi_encode_revoke writes the string
  head offset as 0x60; standard ABI encoding of (address,string) is 0x40
  (offset is relative to the start of the args block: token word + offset word
  = 64 bytes, then len). viem encodeFunctionData of the pinned signature
  (abi.ts revoke "0xafd0224b") returns …token||…0040||len 0x13||data — byte-
  identical to the shipped layout EXCEPT the offset word. What breaks: a
  standard decoder seeks to args+96, reads the string bytes as the length
  (~7e22) and reverts — every drift-revoke tx reverts on-chain, killing the
  spec.md:35 auto-revocation beat (the demo's upgrade→revoke moment). The test
  asserts selector/length/padding but not the offset, so it passes either way.
  Smallest fix: U256::from(0x40u64) at signer.rs:23 + assert the offset word in
  revoke_encoding_carries_the_pinned_selector.
```

```
[blocking] packages/shared/index.ts:8 — `export * from "./events";` references
  a file this branch does not ship (events.ts exists only on
  step-3-contracts-core). `tsc` on the shared entry fails: TS2307 Cannot find
  module './events'. The branch standalone breaks the shared package entry
  point; vitest stays green only because the tests import ../abi and ../types
  directly, never ../index — inherited-green-masks-it, the exact trap LE7
  closes for wasm. Also outside the verify-prescribed shared-edit set (that
  export belongs to step-3's merge, which ships events.ts with it — findings.md
  itself says the file "arrives with step-3's merge"). Smallest fix: delete the
  line from this branch; step-3's merge re-adds it together with events.ts.
```

Non-blocking notes (do not withhold approval):

- N1 [non-blocking] scan-backend/src/watchdog.rs:17 (and findings.md "What the next step needs to know" echo) — claims "the live read stays available in tests/live_integration.rs"; that file does not exist (tests/ = fixtures/ + golden_fixtures.rs) and no live watchdog-counter code ships anywhere. Say "deferred until a mechanism reconciles" instead of pointing at a phantom file.
- N2 [non-blocking] scan-backend/src/rules.rs:84–87 and drift.rs:6–9 module doc — both still say the event signatures are "not pinned yet (verify loose end 2)" so the live path "ships revocationTx:null" / "event-log enumeration is unavailable" — while hexutil.rs pins both topic0s and lib.rs uses them (verified byte-equal to step-3's events.ts). Behavior is ahead of the comments; fix the two comments so the next reader doesn't re-implement the degrade.
- N3 [non-blocking] fingerprint.rs composition implements F2–F4 of step-6's joint FINGERPRINT.md ("ALL must pass"); F1 (solc 0.8.x metadata) and F5 (selector subset) are not implemented. The plan's three-way criteria hold on the subset and FINGERPRINT.md says "Step 4's matcher composes from THIS list" — coordinator should carry the deliberate-subset fact so step-6/step-8 consumers don't assume /scan's signature_match ≡ F1–F5.
- N4 [non-blocking → coordinator, endorse builder's flag] Watchdog ground truth is measured stale: spec.md:8's "6,092 in 6 weeks (~150/day)" vs live batchCount 258,707 / ~2,630/day with fork-specific event signatures. The pre-committed degrade shipped correctly; the 6,092 figure in the spec/power-report copy is now a product-level correction for the coordinator, outside this diff. WATCHDOG_OK.json keeping 6092 is correct as the OK-shape fixture.
- N5 [non-blocking] fingerprint.rs:90 — impl_addr non-zero test `a.chars().any(|c| c != '0')` is always true for a "0x…"-prefixed string ('x' qualifies). Dead defensive check; no live path reaches a zero impl_addr today (answers_implementation filters), fix opportunistically.
- N6 [non-blocking] lib.rs:29 FRESH_BUYER comment says "a fresh EOA the spike used" but the value is 0xd8da…6045 (a famous address). Probe semantics unaffected (answered-vs-reverted is what S4/S5 test); correct the comment.
- N7 [non-blocking] frontend/src/lib/api.ts:23–29 (step-5's local WatchdogStats, not this diff) lacks provenanceUrl — live-mode widget will render the degrade sentinel runs:0 as a measured zero until step 8's scheduled reconciliation (wire.md + watchdog.ts already schedule it).
- N8 [non-blocking] rpc.rs estimate_gas has zero callers (drift.rs deliberately fixed 400k, documented ponytail). Delete or keep knowingly for steps 7/9.

## Verdict

Two blocking findings (B1 ABI offset — every drift-revoke tx reverts on-chain; B2 dangling ./events export — shared entry breaks standalone). Both are one-line fixes with test pins; everything else — including the two hard judgment calls (fingerprint composition, watchdog degrade) — holds as built.

## Round 2 (2026-09-20) — re-check of 4bcabe7..24628f7

Both blockers verified fixed, independently re-run on the branch:

- B1 FIXED — signer.rs:26 writes 0x40, and revoke_encoding_carries_the_pinned_selector now
  asserts the offset word (data[36..68] == 0x40 word), so the regression is pinned. Post-fix
  encoder output is byte-exact vs viem encodeFunctionData of the pinned revoke(address,string)
  (266 hex chars: afd0224b | token | 0x40 | len 0x13 | padded data) — reconfirmed this round,
  not taken from the builder's report. Test passes; suite green (37+5).
- B2 FIXED — index.ts exports only abi/types/watchdog; the ./events line replaced by a comment
  naming step-3's merge as the ship point. tsc on the shared entry now clean (round 1's TS2307
  gone); vitest still 18/18.
- N1/N2/N6 taken as prescribed: phantom tests/live_integration.rs reference reworded to
  "deferred" (watchdog.rs + findings.md); stale "not pinned yet" comments in rules.rs/drift.rs
  corrected to describe the shipped pinned-topic0 behavior; FRESH_BUYER comment now accurate.
- N5/N8 knowingly kept with reasons recorded in findings.md — acceptable. N3/N4 coordinator-
  carried; N7 is step-5's file, correctly untouched.
- Round-2 diff scope: only the fix files + workbench docs — no scope creep.
- Re-run this round: scan-backend 42/42, wasm32 check clean, shared Rust 4/4, shared TS 18/18,
  tsc clean.

### Verdict

APPROVED — 0 blocking. Remaining carried items are coordinator-level, not code: N3 (matcher
implements F2–F4 of step-6's FINGERPRINT.md — deliberate subset, criteria hold), N4 (spec's
6,092/~150-per-day watchdog figures measured stale — product copy correction), N7 (step-5
api.ts WatchdogStats gains provenanceUrl at step-8 reconciliation).

## Round 3 (2026-09-20) — spot review of post-deploy live-check fix e419bc1 (vs merged main ebaa61c)

The bug (invisible to rounds 1–2: no fixture exercised a failing probe read): a rate-limited or
dropped probe read collapsed into `false` → power-report row rendered ABSENT with severity
verified — fabricated negative evidence against spec.md:30. Fix verified semantically right:

- The None/false/answered trichotomy is honest under the depth-boundary discipline. Failed read
  → `None` → `inputs.probes = None` → evaluate claims no probe rows, and `finalize_signature_match`
  receives (false,false) → no signature match → honest UNVERIFIED; under-claims, never fabricates.
  Definitive revert → `false` → ABSENT/verified row now backed by real probe evidence — exactly
  the spec.md:30 row-evidence rule. Answered → PRESENT/risk. Undecodable paused answer (Ok but
  not a bool) → None: no clean evidence either way, correctly not ABSENT. beacon:None → None
  (drops the independent paused evidence on beacon-less contracts, but those are already outside
  the pattern → UNVERIFIED + heuristics; omission is compliant where fabrication was not).
- Tests pin the three named paths with distinct mock transports: failed-to-None (429 transport),
  reverted-to-definitive-negative (revert error object → Some(false,false)), answered-to-PRESENT
  (32-byte bool → Some(true,true)). Re-ran: 3/3 pass.
- Scope: exactly the declared 4 files (fingerprint.rs fix+tests, lib.rs propagation+comment,
  rules.rs doc, findings.md). No creep. Suite re-run: 40 unit + 5 golden green, wasm32 clean.

Non-blocking notes (do not withhold approval):
- R3-N1: blocklist still accepts any `Ok(_)` as answered — an empty "0x" success would render
  PRESENT/risk without a decodable bool, while paused now requires one. Pre-existing, unchanged
  by this commit, and the conservative direction (over-warns, never fabricates absence). Fold a
  decode_abi_bool check in whenever this file is next touched.
- R3-N2: the undecodable-paused→None and beacon-None→None branches are unpinned by tests (the
  three dispatch-named paths are pinned). Trivial to add if the suite is touched again.

### Verdict

APPROVED — 0 blocking.
