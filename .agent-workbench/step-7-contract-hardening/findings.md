# Step 7 — contract hardening + quality evidence

status:     merged (branch step-7-contract-hardening → main as 93201ab via --no-ff; review round 1 APPROVED 0 blocking; full suite green on merged main: contracts 30+2+14, scan-backend, shared 4+4+21, frontend 84 + build)
branch:     step-7-contract-hardening
deployed:   not deployed — operator key 0x151e9f57F31310aFeBBB60c222c14badCf938E4C reads 0 balance on 4663/46630/421614 (re-verified by this step 2026-09-21 via eth_getBalance); the 4663 deploy + verify + worker handoff are the named blockers below, with the exact runbook in this file. Everything else (fuzz, docs, gas report, warnings, audit) is done and green.

## Review round 1 (2026-09-21) — APPROVED, 0 blocking

Non-blocking notes, disposition: (1) three dead test helpers in the guard test target
(`BEACON`, `mock_record`, `install_beacon_token` — pre-existing on main, print dead-code
warnings under `cargo test -p guard`) — follow-up cleanup, deliberately NOT folded into
the reviewed diff; (2) `contracts/Cargo.lock` + hello's feature declaration outside the
letter of the file scope — no action, mechanical shadow of the plan-named proptest
dev-dep, consciously disclosed. Deferred deploy confirmed not a blocker per standing
dispatch.

## What was built

- **Fuzz + invariant suites (plan task 1)** — proptest, 10,000 cases each, fixed ChaCha
  seeds (bit-reproducible in CI; failures shrink + persist). `registry`: one model-based
  sequence property carrying both plan invariants (registrar authority = only write path;
  transitions total + monotone — exact seven-field state asserted after every op, which
  subsumes revert-leaves-zero-state) + a `getRecord` totality property. `guard`: red_flag
  priority-order totality (output is EXACTLY the first applicable condition, checked
  against a fresh transcription of the journeys.md:32 order), probe-degradation
  monotonicity (degrading a probe can never surface an earlier flag), extraction
  robustness on arbitrary bytecode (never panics, deterministic, answers only from real
  shapes in the input) + constructive acceptance at any offset, zero-storage-delta on all
  pre-write reverts, and calldata/word encoder round-trips. Suites: registry 12→14,
  guard 21→30, all green; wasm32 build unaffected (dev-dep only).
- **Gas report (task 2)** — `docs/gas-report.md`: budget definition, the exact receipts
  methodology (stage E hash-join, per-row status+gas gate, 11 rows), dry-run-measured
  sizes/data-fees for registry (11.2 KB / 0.000084 ETH) and guard (11.9 KB / 0.000090
  ETH) on BOTH 421614 and 4663 today, the PASS-PENDING actuals table, and a
  pre-registered drift policy. **Verify loose ends 4+7 applied:** PASS-PENDING-FUNDS is
  the declared unfunded-at-close end-state (step-2 precedent) and the path is
  `docs/gas-report.md`.
- **Docs (task 3)** — `docs/criteria.md` (exactly the path the registry panel links,
  `github.com/dujar/vetted/blob/main/docs/criteria.md`; record fields, positive-evidence
  rule, revocation policy, depth boundary, guard consumption, evidence discipline),
  `docs/threats.md` (trust roots, registrar-compromise blast radius — verdict lies + DoS,
  never fund theft — single-registrar rationale, revocation-as-safety, reentrancy walk,
  adversarial-token table, fail-open/fail-closed split, MM-key trust, error taxonomy,
  chain assumptions). Security-considerations rustdoc added to both contracts; all public
  surfaces documented.
- **Quality pass (task 4)** — the 8+8 unexpected-cfg warnings (registry/guard) fixed by
  declaring `contract-client-gen = []` (sdk-macro cfg gate; no codegen change; hello's
  identical 4 cleared too). Leftover `ponytail:` marker removed from
  `guard/src/lib.rs` `extract_beacon_from_code` (perf fact kept as prose). Fresh-eyes
  audit outcomes folded into threats.md §4 (reentrancy, overflow posture — no arithmetic
  on user values anywhere, taxonomy completeness).
- Verify.md's 8 loose ends: 1, 2, 3, 5, 8 → applied below in the runbook; 4 + 7 →
  applied in `docs/gas-report.md`; 6 → applied (gas clause dropped from the fuzz suite,
  kept where measurable — the receipts).

## Where the plan was wrong

Nothing structural. Two notes: (1) plan task 1's "no path exceeds the probe gas budget"
had no native-host referent (verify loose end 6) — dropped there, kept on-chain. (2)
proptest's `RngAlgorithm::PassThrough` seed mode is pathological for 10k fresh cases
(minutes per property); the fixed-seed mode that is actually documented for
cross-run reproducibility is ChaCha — both crates' `runner()` folds a readable label into
the 32-byte ChaCha key. A first draft of the zero-config property also accepted
(false,false) = valid config as input; fixed to always generate ≥1 zero binding.

## The funded-run runbook (exact — executes the deferred task 5)

Precondition: DEPLOY_KEY funded (key material in `spike/.env`, never commit). 421614 via
faucet (`spike/DEPLOY.md` §0), 4663 via the Arbitrum portal bridge (~10 min). Budget: two
Stylus deploys ≈ 0.0005 ETH + foundry set + 13 receipt txs per chain — tenths of an ETH
cover everything.

1. **First funded run = END-TO-END `deploy.sh`** (step-3 round-2 routing rule):
   `source spike/.env && DEPLOY_KEY=$SPIKE_DEPLOY_KEY ./scripts/deploy/core/deploy.sh --chain 421614`
   → stages A–F: registry+guard deploys (validated by cast-call), mock tokens, the 11
   on-chain receipts (five byte-exact reverts + degraded + settles), the per-row ≤200k
   gate, and `deployments/421614.json`. **Writer-of-record note (loose end 5):** step 3's
   funded run never happened, so THIS run writes `421614.json` — append-only discipline
   unchanged; update the `deployments/README.md` row to add 7 when it lands. If only one
   chain gets funded, spend it on 4663 (the script is end-to-end there too).
2. **Registrar key first (loose end 1)** — the production registrar must be FUNDED or
   step 9's revokes die exactly like a key mismatch: `cast wallet new`; fund the new
   address on 4663 via the same §0 bridge (0.005 ETH is generous — revokes are single
   storage writes).
3. **4663 core deploy:** `DEPLOY_KEY=$SPIKE_DEPLOY_KEY ./scripts/deploy/core/deploy.sh --chain 4663`.
   The pinned script deploys with registrar = DEPLOYER (`deploy.sh:82-84` hardcodes +
   validates it) — **deploy-with-different-registrar is impossible unmodified (loose end
   2)**; the receipts run in this same pass, signed by DEPLOYER while it is still
   registrar. Then hand over:
   `cast send $REGISTRY "transfer_registrar(address)" $NEW_REGISTRAR_ADDR --private-key $SPIKE_DEPLOY_KEY --rpc-url https://rpc.mainnet.chain.robinhood.com`
   and validate with `cast call $REGISTRY 'registrar()(address)'`. **Post-transfer
   `--receipts-only` re-runs must sign with the NEW registrar** (registrar-only writes
   revert under the old key).
4. **deployments/4663.json:** the script's stage F creates the file if absent (step 6 has
   not executed a funded replicas deploy) or shallow-merges append-only over its
   `replicas` field — never rewrite step 6's entries. Record BOTH addresses for the
   frontend env handoff (`VITE_REGISTRY_ADDRESS`, `VITE_GUARD_ADDRESS`, Pages env at
   steps 8/10) — **PENDING until this run**.
5. **Verify:** `cargo stylus verify --endpoint https://rpc.mainnet.chain.robinhood.com $REGISTRY` (+ guard).
   Mainnet explorer API may sit behind a Cloudflare challenge server-side
   (`knowledge/contract-verification.md:19-27`) — fallbacks: browser-UI verification on
   robinhoodchain.blockscout.com (browsers pass CF) or `--verifier sourcify`. 421614
   mirror: `cargo stylus verify` + Etherscan v2 (`--chain-id 421614`, ETHERSCAN_API_KEY).
6. **Worker handoff** (worker `vetted-scan-backend.dujar-coding.workers.dev`, wrangler
   authenticated on this box; two secrets ship deliberately unset):
   a. `cd scan-backend && wrangler secret put REGISTRAR_KEY` ← the new registrar key.
   b. `wrangler secret put DRIFT_ADMIN_SECRET` ← generate one (step 9's admin-gated
      `POST /registry/drift-check` beat).
   c. **Registry-address var (loose end 3):** `REGISTRY_ADDRESS_4663` is a `[vars]`
      entry — do NOT edit `wrangler.toml` + redeploy (step 8 mutates the same live
      worker in parallel; a redeploy from a stale checkout silently blanks its var).
      Pass vars on the command line with ALL currently-known non-empty registry vars:
      `wrangler deploy --var REGISTRY_ADDRESS_4663:0x…` (here: only 4663). Sequence:
      step 7 first, step 8 re-passes 4663 alongside its scratch pair (its plan already
      routes that).
   d. **`DRIFT_EXTRA_TOKENS` (loose end 8): step 9 owns it** — when the demo beat needs
      the replica tokens inside the drift walk, step 9 passes it with the same `--var`
      technique in its own reconfiguration.
   e. Validate: `/scan?chainId=4663&addr=<replica token>` returns a non-null record
      (ends the canonical-fetch-only degrade); drift-check with the admin secret returns
      a real report.
7. **Gas actuals:** fold the 11 rows from `deployments/<chain>.json` `receipts` into
   `docs/gas-report.md` §4 with the §5 drift policy; flip the PASS-PENDING-FUNDS header.
   The spike gate receipt (`spike/DEPLOY.md` §1–4) can ride the same funding session.

## What the next step needs to know

- **Frontend env contract (plan Revised 2026-09-19 note 1):** `VITE_REGISTRY_ADDRESS` /
  `VITE_GUARD_ADDRESS` = `registry` / `guard` fields of `deployments/4663.json` — the
  file does not exist yet; it is created by the runbook's step 3/4. Steps 8/10 set Pages
  env from it.
- **Worker state is unchanged by this step** (still version 0ccf8c68, secrets unset,
  registry vars empty, degrade paths live-green). Steps 8 (scratch pair) and 9 (beats)
  depend on the runbook above having run.
- **Contracts compile warning-free now** (all 20 unexpected-cfg warnings cleared);
  `contract-client-gen` is declared-but-never-enabled in all three core crates — do not
  "simplify" it away, it pins the sdk macro's cfg gate for rustc's unexpected_cfgs check.
- **Fuzz harness facts:** proptest runners are seeded (fixed ChaCha keys, `runner(seed)`
  folds any-length label to 32 bytes); the registry model property is the authoritative
  spec of write semantics — if you change verify/revoke/transfer behavior, the MODEL is
  what must change first, then the contract, or the property is no longer an
  independent check. Guard red-flag ordering is pinned against a fresh transcription of
  the journeys order, not against the implementation.
- **Step 9 owns `DRIFT_EXTRA_TOKENS`** (recorded per loose end 8).

## Out of scope, left broken

- `guard` `erc20()` bool-decode accepts ANY returndata whose byte 31 is 1 (e.g. 64 junk
  bytes decode "true") — a lying token could silently fail commit custody. Worst case is
  a phantom active order for the buyer who chose that token (execute then reverts whole;
  funds safe). Tightening is a behavior change → out of this step's mandate
  ("tests + docs only"); flagged for a coordinator decision.
- `contracts/core/hello` was outside the pinned warning ownership (step 3 named
  registry/guard) but got the identical 3-line feature fix to zero the whole build.
- Native suite cannot meter EVM gas nor roll back post-deactivation writes (no revert
  primitive in stylus-test) — those invariants live in the receipts by design
  (`docs/gas-report.md` §2/§4, threats.md §4).
- The 4663 deploy, verify pages, worker handoff, and `deployments/4663.json` — BLOCKED on
  funding, runbook above. Unblock: fund the operator key (§0), then any builder can
  execute the runbook; nothing else is missing.
