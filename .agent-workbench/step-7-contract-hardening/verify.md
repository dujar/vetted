# Step 7 — verify (2026-09-20, main b889011, clean)

## Hard gates
Neither trips. Screens: N/A (no new UI surface). User journey: N/A is correct — tests, docs, a gas report, and a deploy; nothing a person navigates.

## Verified sound (checked against the tree, not the plan's word)
- `scripts/deploy/core/deploy.sh:47` accepts `--chain 4663` (RPC pinned), and stage F **creates** `deployments/<chain>.json` when absent (`deploy.sh:202-203`) with append-only shallow merge (`:198-201`). So task 5's "read `REGISTRY_ADDRESS_4663` from `deployments/4663.json` post-deploy" works even though step 6 has not executed its funded replicas deploy — the coordinator's pre-rehearsal question resolves: task 5 reads the file its own deploy run just wrote.
- Worker names match code exactly: `REGISTRAR_KEY` (`scan-backend/src/lib.rs:110`), `DRIFT_ADMIN_SECRET` (`:111`), `REGISTRY_ADDRESS_{4663,46630,421614}` (`:107-109`, empty at `wrangler.toml:13-15`). wrangler.toml's trailing comment documents both secrets as `wrangler secret put` — the dispatch's secret mechanism matches how the worker is deployed.
- `transfer_registrar` exists: `contracts/core/registry/src/lib.rs:205`.
- Criteria path: `frontend/src/pages/RegistryPage.tsx:17` links `github.com/dujar/vetted/blob/main/docs/criteria.md` — task 3's file is exactly the resolving path once merged to main.
- Guard `ponytail:` marker is at `contracts/core/guard/src/lib.rs:587` (plan's correction of step-3's `:588` is right).
- `packages/shared/abi.ts:49-63` — blocklist probe targets the resolved beacon, matching Revised note 2. 4663 RPC pinned (`wire.md:89`, `chains.ts:12`) — no re-probe, as Revised note 1 says.
- Knowledge: no contradiction with `stylus-toolchain.md`. Toolchain pinned 0.10.9 (`scripts/toolchain.sh`, step-1 findings) satisfies the ≥0.10.8 verify pin; step-3-measured sizes 11.1/12.0 KB stay under the 24 KB fragmentation threshold; the 4663 Cloudflare-challenge fallback (browser UI / sourcify) is in `contract-verification.md:19-27` as cited. (robinhood-chain.md's 96 KB/192 KB limit is at `:36`, plan cites `:38` — same fact, two lines off.)
- CI: no new job needed and none is claimed. Fuzz suites land in the existing `contracts` job (`cargo test --workspace`, `ci.yml:13-22`); a proptest dev-dependency is native-only so the wasm32 build step is unaffected; docs/ and the gas report are static files with no CI claim; the mock-token job already exists (`ci.yml:44-59`) and mirrors the foundry-deps pattern of plan note 5.
- File-scope collision with step 8: none. Step 8's scope is `e2e/**` + the one-file `frontend/src/lib/api.ts` exception; its Revised note 4 explicitly leaves `REGISTRY_ADDRESS_4663` to step 7 and takes the scratch pair itself.

## Loose ends

1. [Missing prerequisite] The new production registrar key is never funded
   where:    plan task 5 ("generate the production registrar key, set it as the backend Worker's `REGISTRAR_KEY` secret")
   evidence: the worker signs and broadcasts drift-revokes from that key (`scan-backend/src/drift.rs:190-201`, fixed gas fields in `signer.rs:41-43`); an unfunded signer cannot pay the revoke tx, so step 9's upgrade→revocation beat dies exactly as a registrar mismatch would. The only funding runbook, `spike/DEPLOY.md` §0, funds ONE address (the spike key 0x151e…8E4C) — nothing covers a second key.
   fix: add to task 5 — fund the new registrar key on 4663 (same §0 faucets/bridge), or state that step 9's beat funds it and name the amount/source.

2. [Dangling reference] "deploy with registrar = that address" is impossible via the pinned script
   where:    plan task 5 ("deploy with registrar = that address (or `registrar-transfer` to it immediately after deploy)")
   evidence: `deploy.sh:82` hardcodes `--constructor-args "$DEPLOYER"` and `:83-84` aborts unless `registrar() == DEPLOYER` — the end-to-end script cannot deploy with a different registrar unmodified.
   fix: strike the first alternative; pin the working path: run deploy.sh end-to-end first (its stage-D receipts are registrar-only writes by DEPLOY_KEY), then `cast send $REGISTRY "transfer_registrar(address)" <new-key-addr>`, then set the secrets/var. Note that post-transfer `--receipts-only` re-runs must sign with the NEW registrar — DEPLOY_KEY re-runs would revert on the registrar-only calls.

3. [Untouched sibling] `REGISTRY_ADDRESS_4663` is a `[vars]` entry, and step 8 mutates the same live worker in parallel
   where:    plan task 5 ("set ... `REGISTRY_ADDRESS_4663` wrangler var") vs step-8 plan task 5 (scratch pair)
   evidence: `wrangler.toml:13-15` — the registry addresses are vars, not secrets, so setting one means a redeploy carrying the whole `[vars]` set; `wrangler.toml` sits in NEITHER step's declared file scope, and whichever `wrangler deploy` runs last from a checkout where the other step's var is still `""` silently blanks it.
   fix: one sentence in the plan — set it with `wrangler deploy --var REGISTRY_ADDRESS_4663:0x…` passing ALL currently-known non-empty registry vars (no file edit, no clobber), and sequence the two worker reconfigurations (step 7 first; step 8 re-passes 4663).

4. [Deferred decision] No unfunded-at-close end-state
   where:    plan Revised note 1 / task 2 ("the first funded run executes them end-to-end before this diff is written")
   evidence: the plan routes its own receipts, the 4663 deploy, the verify pages, and the worker handoff all through a funded run, but never says what ships if funding still hasn't landed when step 7 closes — block, or ship pending? Step 2's precedent (PASS-PENDING-GAS + operator-runbook pointer) is cited in Revised note 2 but not adopted as this step's fallback.
   fix: one sentence — if still unfunded at close, commit `gas-report.md` with PASS-PENDING receipt markers (step-2 precedent) and record the 4663 deploy + handoff as named blockers in findings.md; never fabricate receipts.

5. [Untouched sibling] `deployments/README.md` writer pin contradicts the first-funded-run fallback for 421614
   where:    plan Revised note 1 + task 2/5 (421614 receipts and the 421614 verify mirror) vs `deployments/README.md:11`
   evidence: README pins `421614.json` writers as "3, 6 (replicas field)" — but when step 3's funded run never happened, step 7's own 421614 run executes `deploy.sh --chain 421614`, whose stage F (`deploy.sh:180-205`) writes `deployments/421614.json`. Step 7 is not a listed writer there (only `4663.json` was reconciled to "6 then 7").
   fix: one line in the plan — on the first-funded-run fallback, step 7's operator run is the writer of record for `421614.json` (append-only discipline unchanged); update the README row when the run lands.

6. [Dangling reference] Task 1's fuzz check "no path exceeds the probe gas budget" has no referent and no mechanism
   where:    plan task 1 (fuzz + invariant suite over the native host)
   evidence: no budget constant exists anywhere in contracts/core (`grep budget|ink|gas` in `guard/src/lib.rs` → only the `:588` comment); probes forward ALL gas by design (`guard lib.rs:154` "forward all gas, no ETH"); the native `stylus_sdk::testing` host does not meter EVM gas, and the ≤200k assertion already lives where it is measurable — on-chain from broadcast receipts (`deploy.sh:125-171`, task 2).
   fix: drop the clause from the fuzz suite, or redefine it as a deterministic invariant the native host can assert (probe count/ordering — largely covered by the determinism check in the same task); keep the gas budget assertion in the receipts.

7. [Deferred decision] `gas-report.md` path unpinned
   where:    plan task 2 ("commit `gas-report.md` with receipt links") / Scope ("gas report artifact")
   evidence: no directory is named anywhere in the plan; `docs/` does not exist yet and the only link a consumer holds is the criteria-panel GitHub URL, which points at `docs/criteria.md` only.
   fix: name the path — `docs/gas-report.md` sits naturally beside `criteria.md`/`threats.md` and gets judged with them.

8. [Orphan output, minor] `DRIFT_EXTRA_TOKENS` is assigned by a comment but owned by no plan
   where:    `scan-backend/wrangler.toml:21-22` ("step 6's replicas go here for the demo beat")
   evidence: grep across `.agent-workbench/` finds zero plan references (only code reads at `drift.rs:7`, `lib.rs:112`); step 9's plan creates `scripts/seed-demo/` and never mentions it. If step 9's drift-check beat needs the replica tokens inside the drift walk, the step-7 handoff redeploy (loose end 3) is the natural moment to set it.
   fix: name the owner — either fold it into task 5's var set or record in findings.md that step 9 owns it.

## Bottom line
No hard gate trips and the reconciled pins all check out against the tree (deploy.sh 4663 support, 4663.json creation by the script itself, worker secret/var names, transfer_registrar, criteria path, CI coverage). The eight items above are one-liner plan edits, not structural: the deploy-day ordering (1, 2, 3) is the cluster that would actually bite during the funded run.
