# Step 9 review — round 1 (2026-09-21)

Branch `step-9-demo-harness` vs `main` 8648efc, diff reviewed in full (8 files, +1075/−19). Plan.md read in full incl. all five Revised notes; knowledge files `robinhood-chain.md` + `goplus-api.md` consulted; all four verify.md loose ends traced into the diff.

## Findings

```
[blocking] demo/arc.sh:213-214 — the beat-6 REVOKED scan is printed but never asserted — after the drift-check confirms the revocation, `scan_retry "$REPLICA"` prints `.verdict` and falls through unconditionally; a stale/rate-limited read returning VERIFIED or UNVERIFIED (the exact 4663 failure mode Revised 2026-09-20 note 1 documents) exits 0 with wall-clocks pasted into §5 — the flagship beat's central on-camera claim is the one verdict with no honesty gate, while beats 2/3/7 all assert (:146, :154, :238) — smallest fix: one line in the same shape as beat 2, e.g. `printf '%s' "$SCAN_JSON" | jq -e '.verdict == "REVOKED"' >/dev/null || { echo "HONESTY: replica does not read REVOKED after revocation — blocking" >&2; exit 1; }`
```

```
[blocking] README.md (demo-cast labeling paragraph, "the tool's own rules flag the impostor twins") — contradicts the honest twin beat this branch itself builds — verify LE 2 established the live engine cannot render IMPOSTOR for these twins (not on the issuer list → rules.rs:306 mimic-source search finds nothing → twins scan UNVERIFIED; e2e/specs/scan.live.spec.ts pins IMPOSTOR count 0), and the builder applied LE 2 in runbook beat 3, video beat 3, and the arc.sh:154 assertion — but this sentence copies the stale step-6 claim (demo/assets.md:9) into the submission front page — a judge reading the README then watching beat 3 ("the engine won't wear the impostor word on a guess") sees the repo contradict the demo's honesty line, in an asset the plan task 5 says feeds real-problem-solving — fix: "the tool's depth boundary marks the twins UNVERIFIED and the guard refuses them on-chain at execution"
```

## Non-blocking notes

- `demo/runbook.md:218` contains "6,092" — only as the explicit prohibition ("never the stale 6,092/~150 spec copy"). Compliant; noted so a later grep doesn't misread it.
- `demo/arc.sh:132-133` — if `$BASE_URL/health` is unreachable, `curl -fsS` dies under `set -euo pipefail` with curl's raw error; the friendly runbook-§6 message only covers `ok != true`. Cosmetic.
- Runbook go-list step 2 asks the operator to translate forge log keys (`REPLICA_PROXY`, `TWIN2_IMPOSTOR_SELFPROXY_PROXY`, …) into the `.replicas.{proxy,beacon,implV2,twin1,twin2}` JSON keys without writing the shape down. The names are pinned by `e2e/scripts/seed-scratch.sh:52-54` and all three step-9 scripts agree, and the task-6 preflight errors on a missing field — but one example JSON block in the runbook would remove the guesswork.

## Checked and clear (evidence)

- **Arc complete and honest:** all beats present in runbook §3 + video script 1:1 — live non-self-deployed scan (default `0x1Cdad3…b7D` = P Everpure; backup `0xd95B4412…D44` = CRM Salesforce, both verified against `spike/evidence/calibration_4663.json` and the issuer's own asset sample), twin compare, power report, refusal + settle, upgrade → drift-check (`X-Admin-Secret` confirmed lib.rs:287/:390, `DRIFT_ADMIN_SECRET` lib.rs:111) → REVOKED → `GUARD_RECORD_REVOKED` (guard lib.rs:160-164 literal revert strings; `commit(address,address,uint256,uint256)`/`execute()` signatures match guard lib.rs:265/:311), watchdog close.
- **Watchdog figures:** runbook beat 7 + video beat 7 quote 258,707 / ~2,630-per-day with provenance (SequencerInbox address matches knowledge robinhood-chain.md:43 and watchdog.rs:8-10/28); the live `/watchdog` degrade sentinel is narrated honestly; no stale 6,092 anywhere in demo/README copy except the runbook prohibition above.
- **arc.sh retries are real:** `scan_retry` (arc.sh:55-74) backs off 6×10s on `RPC_RETRYABLE` and on transport errors, honors wire.md's verdict-null/terminalState shape (`.verdict // .terminalState`), and aborts with the honest-fallback message. Honesty assertions at :146/:154/:238 genuinely fail the run (jq -e exit codes under `set -e`); drift-report field selectors match `drift.rs` (`skipped`, `revoked[].token`, `tx_hash`) and watchdog selectors match camelCase serde (`baselinePerDay`, `provenanceUrl`).
- **Timing/narration:** budgets 30/30/30/45/45/60/30 = 4:30 + 0:30 buffer, identical across runbook §5, video table, and arc.sh `BUDGETS`; word budgets at ~150 wpm are feasible; judged-criteria mapping present; depth boundary stated on camera (beat 3 + beat 7 close); banned wording ("trust layer", "first coverage") appears only inside the prohibitions.
- **Fallbacks:** every beat has pre-written say/click fallbacks; §6 failure appendix covers persistent throttling, canonical-fetch down (spec.md:55 degraded branch), worker down, mid-take recovery (including the monotone-REVOKED re-deploy path).
- **seed.sh:** gated on `deployments/4663.json` + `status=="deployed"`; `REGISTRAR_KEY` (post-handoff) required for verify/revoke — no anvil key anywhere outside `--anvil-rehearsal` (where it is commented as legitimate, arc.sh:96-98); buyer defaults to the operator (LE 3 applied); idempotency via raw 7-word `getRecord` shape-check — byte offsets verified against the registry Record struct (status word0, registrar word4, registry lib.rs:126-134); `verify(address,uint256,address)`/`revoke(address,string)` match registry lib.rs:182/:214; REVOKED-is-monotone recovery path present; settle plumbing two-sided and consistent with arc.sh beat 5's read-check.
- **Verify loose ends:** LE 1 → `demo/serve.sh` + runbook §1 (env names match verify.md's coded list); LE 2 → runbook/video beat 3 + arc.sh assertion; LE 3 → seed-demo buyer default, anvil key quarantined; LE 4 → rehearsal mode + 0.3s actual recorded in runbook §5, correctly labeled as the upgrade leg only.
- **Scope:** diff touches only `README.md`, `demo/**`, `scripts/seed-demo/seed.sh`, and the step-9 workbench files. No frontend/e2e/contracts/scan-backend changes.
- **Incident:** disclosed in findings.md; verified no-damage — worktree clean, step-6 tracked receipt `run-1789794873744.json` present in the tree, rehearsal artifacts untracked.
- **README claims:** CI is six jobs (verified: contracts, replicas, mock-token, scan-backend, shared, frontend); all linked files exist (`docs/criteria.md`, `docs/threats.md`, `docs/gas-report.md`, `demo/assets.md`, `packages/shared/abi.ts`); `demo.gif` commented until the take.

## Rehearsal coverage (dispatch question)

`arc.sh --anvil-rehearsal` exercises a **subset**: only the upgrade leg on-chain (`upgradeTo` broadcast→receipt + the `implementation()` read, measured 0.3s) against the step-6 replica cast, built against the foundry.lock-pinned lib revs (forge-std v1.16.2, OZ v5.4.0 — arc.sh's clones match). NOT exercised unfunded: both scan beats, the registry verify/revoke legs and the guard refusal (Stylus — need ArbWasm), the drift-check worker leg, and seed.sh's sends (DRY_RUN plan only, validated against a /tmp file). This is disclosed accurately in arc.sh:14-18, runbook §0/§5, and findings.md — the rehearsal is honestly labeled, not oversold.
