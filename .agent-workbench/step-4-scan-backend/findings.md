# Step 4 — scan backend (verdict engine, watchdog, drift-revoke)

status:     merged
branch:     step-4-scan-backend
deployed:   https://vetted-scan-backend.dujar-coding.workers.dev — main 559344c, worker version 0ccf8c68 (2026-09-20); /health + /watchdog live-green; P-VERIFIED receipt pending a calm rate window (below)
branch:     step-4-scan-backend
deployed:   PRE-FIX bundle live at https://vetted-scan-backend.dujar-coding.workers.dev (deployed 2026-09-20 from merged main ebaa61c, version 34d865dd; redeploy required after the probe-evidence fix below merges)

History: written by the continuation builder after the previous builder died post-commit; the
approved body merged to main as ebaa61c and deployed; the probe-evidence fix below is the only
unmerged work. verify.md + review.md travel in this dir.

## What was built
(Previous builder, committed; re-verified green by the continuation builder.) Stateless Workers-Rust
scan backend: rate-limit-aware RPC probe client with beacon-not-proxy resolution, depth-boundary
fingerprint matcher, pure rule engine over golden fixtures, `GET /scan` + `GET /watchdog` routes,
canonical-fetch logic ported from the spike worker, registrar EIP-1559 drift-revoke (cron trigger +
admin-gated `POST /registry/drift-check`), shared watchdog type + fixtures with both round-trip
extensions, CI wasm32 gate. Files: `scan-backend/src/{lib,routes,rpc,rules,fingerprint,canonical,drift,signer,watchdog,hexutil}.rs`,
`scan-backend/tests/golden_fixtures.rs` + bytecode fixtures, `packages/shared/{watchdog.ts,watchdog.rs,fixtures/watchdog/*}` +
wire.md/index.ts/lib.rs appends + roundtrip test extensions, ci.yml scan-backend job.
Inherited green, re-run on the branch: scan-backend 42 tests (37 unit + 5 golden), shared Rust 4,
shared TS 18, `cargo check --target wasm32-unknown-unknown` clean.

## Where the plan was wrong
- Watchdog counter (task 5): no L1 read reconciles with the published numbers. Measured live by the
  builder: SequencerInbox `batchCount()` = 258,707 vs published 6,092 cumulative; delivered-style
  events ~2,630/day vs ~150/day, with fork-specific event signatures. The plan's pre-committed escape
  was exercised correctly: `/watchdog` ships the degrade path — `runs: 0` + `provenanceUrl` (wire.md
  sentinel: 0 = unavailable, never a measured zero) + baseline 150/day. **COORDINATOR FLAG** (the
  plan's escape requires it): the counter mechanism stays unpinned; a live counter read is
  deferred until a mechanism that reconciles with the published figures is pinned.
- Nothing else. verify.md's 8 loose ends are all applied as prescribed (wrangler REGISTRY_ADDRESS_*
  vars with canonical-only degrade; revocationTx null-degrade path; L1_RPC_URL + nullable
  provenanceUrl + degrade sentinel; registrar-cli fallback unused because signing compiled in-crate;
  scope amendment honored via shared append; fixtures/watchdog + both roundtrip extensions; ci.yml
  wasm32 check; Rust selector consts with assertion tests).

## What the next step needs to know
- Registry event topic0s are pinned Rust-side (`scan-backend/src/hexutil.rs` `events::VERIFY_TOPIC0` /
  `REVOKE_TOPIC0`) and match step-3's in-progress `packages/shared/events.ts` byte-for-byte (I
  independently re-derived both via viem keccak256 — equal). hexutil.rs's comment calls itself the
  "mirror of packages/shared/events.ts"; that file arrives with step-3's merge, not this one, so the
  comment briefly references a file not yet on main. Harmless — do not "fix" it.
- Depth-boundary composition shipped (step-6 fidelity test / step 8 consume this): structural layout
  checks only — beacon slot set + impl slot empty, beacon `implementation()` STATICCALL resolves,
  `paused()` probed at the proxy, `isBlocked(buyer)` probed at the RESOLVED BEACON, fingerprint vs
  `spike/evidence/calibration_4663.json` codehashes — explicitly NOT proxy-codehash byte-equality
  (an OZ-BeaconProxy replica passes; the USDG twins miss). See `scan-backend/src/fingerprint.rs`.
- Watchdog wire sentinel: `runs: 0` with `provenanceUrl != null` means count unavailable. The
  frontend mock numbers (6092/150) stay as-is until a reconciled counter exists.
- `REGISTRY_ADDRESS_*` wrangler vars are empty until step 3/7 deploys fill them; `/scan` then serves
  `record: null` and degrades to canonical-fetch-only rules (deliberate, verify loose end 1).
- Signing recipe landed as compiled-and-tested in-crate (`signer.rs`: EIP-1559 sync-sign + recover,
  revoke encoding carries the pinned selector); the `scripts/registrar-cli` fallback never triggered.

## Live receipt outcome (2026-09-20, post round-3 merge 559344c)
Deployed version 0ccf8c68 runs the fixed engine. Live-probed: /health green; /watchdog degrade
sentinel exact (runs:0 + provenanceUrl + 150/day — the sentinel, never a measured zero); /scan
under the 4663 RPC's shared-egress rate limit behaves exactly as designed — `RPC_RETRYABLE`
transient, or completed-but-dropped-probes → honest UNVERIFIED + advisory rows with ZERO
fabricated evidence (the fix verified live in its honest direction). **Pending:** one
VERIFIED-shaped P receipt — it needs all ~6 reads inside one scan to land in a calm window.
10 spaced attempts over ~30 min: 0 full passes, 1 full pass observed pre-fix (windows are real
but rare). Finish it any time with:
`BASE_URL=https://vetted-scan-backend.dujar-coding.workers.dev ./scan-backend/scripts/live-check.sh`
FOR STEP 9 (demo harness): build scan retries into the journey scripts; if the 4663 public RPC
keeps throttling Cloudflare egress, a paid/private RPC endpoint for the demo window is the
upgrade path (ponytail: single-flight elided, 30s in-isolate TTL only — rpc.rs:5). Local
`wrangler dev` (alternative egress) crashes in workerd in this sandbox — not a code issue.

## Review round 3 (2026-09-20) — APPROVED, 0 blocking (probe-evidence fix)
Trichotomy verified honest (failed read → None → no rows + honest UNVERIFIED; definitive revert
→ real ABSENT evidence; answered → PRESENT); all three paths pinned by tests; scope exactly the
4 declared files. R3-N1 (non-blocking, pre-existing): blocklist accepts any Ok(_) as answered —
an empty success would render PRESENT without a decodable bool; conservative direction
(over-warns), fold a decode_abi_bool check in whenever fingerprint.rs is next touched.
R3-N2 (non-blocking): the undecodable-paused→None and beacon-None→None branches are unpinned by
tests; trivial to add next time the suite is touched.

## Post-merge live receipt — probe-evidence bug FOUND and FIXED (2026-09-20)
The approved code merged to main (ebaa61c) and deployed cleanly, but the plan's own live check
(task 8: P must come back VERIFIED-shaped) exposed a real bug no fixture could see — nothing
exercised `fingerprint::probe` with a failing read:
- **Bug:** `probe` collapsed "RPC read failed" (rate limit — the 4663 public RPC throttles
  Cloudflare egress; live scans oscillate between RPC_RETRYABLE and completing) into the same
  `false` as "selector reverted". A rate-limited probe then rendered a power-report row
  ABSENT with `severity: verified` — fabricated negative evidence, violating the spec.md:30
  evidence rule the step exists to enforce (the verdict side was safe: unanswered withheld
  VERIFIED → UNVERIFIED, never a wrong positive).
- **Fix (this branch, needs one review round):** `probe` now returns `Option<Probes>` — a failed
  read yields None: no probe rows claimed at all, no signature match (honest UNVERIFIED);
  `false` is only ever a definitive revert (which IS evidence: the mechanism is absent).
  lib.rs propagates None; rules.rs renders rows only from real evidence; 3 new unit tests pin
  failed→None, reverted→definitive-negative, answered→PRESENT. Suite: scan-backend 40+5, wasm32 clean.
- **Environmental (coordinator should know, not a code fix):** the public 4663 RPC rate-limits
  Cloudflare's shared egress — whole scans intermittently return the DESIGNED `RPC_RETRYABLE`
  terminal (correct behavior: it tells the UI to retry; it never guesses). A green live P
  receipt needs a calm rate window plus retries; step 9's demo harness may want scan retries
  built into the journey scripts. The spike canonical-fetch mirror is unaffected.

## Review round 2 (2026-09-20) — APPROVED, 0 blocking
Both blockers independently re-verified fixed (reviewer re-ran the suite and the viem byte-exact
check this round, not taken from my report); round-2 diff scoped to fix files + workbench docs,
no scope creep. Carried items are coordinator-level: N3 (matcher = F2–F4 subset of step-6's
FINGERPRINT.md — deliberate, criteria hold), N4 (spec's 6,092/~150-per-day watchdog figures
measured stale — product-copy correction), N7 (step-5 api.ts WatchdogStats gains provenanceUrl
at step-8 reconciliation).

## Review round 1 (2026-09-20) — 2 blocking, both fixed this commit
- B1 signer.rs ABI offset: `abi_encode_revoke` wrote the (address,string) string head offset as
  0x60; standard ABI is 0x40 — every drift-revoke tx would revert on abi.decode. Fixed to 0x40
  AND the test now asserts the offset word (regression pinned). Reviewer verified byte-exact
  against viem's encoding of the pinned `revoke` selector; fix changes only that word.
  Independently reconfirmed post-fix: the encoder's output is byte-identical (266 hex chars,
  incl. the 0x40 offset word) to viem `encodeFunctionData` for the same args.
- B2 packages/shared/index.ts: dropped `export * from "./events";` — events.ts exists only on
  step-3's branch; the export ships with step-3's merge (the line is replaced by a comment saying
  exactly that).
- Non-blocking: N1 phantom tests/live_integration.rs reference — comment + this file reworded to
  "deferred"; N2 stale "not pinned yet (verify loose end 2)" comments in rules.rs/drift.rs —
  corrected (behavior already uses the pinned topic0s); N6 FRESH_BUYER comment corrected.
  Knowingly kept: N5 fingerprint.rs:90 always-true guard (no live path reaches zero impl_addr;
  a real fix is a zero-address comparison, out of round-1 scope); N8 rpc.rs estimate_gas kept
  for steps 7/9 (zero callers today, deliberate per drift.rs's fixed-400k ponytail). N3
  (matcher = F2–F4 subset of FINGERPRINT.md) and N4 (spec's 6,092/~150-per-day measured stale)
  are coordinator-carried, not code. N7 is step-5's api.ts, outside this diff's scope.
- Suite re-run after fixes: scan-backend cargo test green, wasm32 check clean, shared Rust +
  TS green, tsc on the shared entry green.

## Out of scope, left broken
- `.agent-workbench/step-6-replica-assets/findings.md` on main still has an empty `reconciled:` —
  the caller's Next-1 reconcile sweep should cover it (its substance already reached this build: the
  branch sits on top of the step-6 merge and FINGERPRINT.md was in tree while building).
- `deployments/worker.json` not yet written — step 4's writer pin lands with the post-merge deploy.
- Watchdog live counter mechanism unpinned (coordinator flag above); spike canonical-fetch worker
  stays up as the fallback mirror per plan Revised note 2.
