# submission/post-funding-checklist.md — the funded-window completion run

Executable top-to-bottom by any operator once the operator key is funded.
Everything before step 1 is already done and merged. Steps are the plan's
task 2/3/4/5 completion + the go-list; nothing else is missing.

**Go/no-go: 2026-09-29.** If step 0 still reads unfunded on that date, STOP
and escalate to the coordinator: same-day funded wallet, or the declared
unfunded variant ships (anvil-rehearsed wall-clocks, PASS-PENDING-FUNDS
honesty copy, no contract-address section — `submission/README.md`).

## 0. Precondition check (run first, run again at the end)

```bash
KEY=0x151e9f57F31310aFeBBB60c222c14badCf938E4C
for RPC in https://rpc.mainnet.chain.robinhood.com https://sepolia-rollup.arbitrum.io/rpc; do
  curl -sS -X POST "$RPC" -H 'Content-Type: application/json' \
    -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"eth_getBalance\",\"params\":[\"$KEY\",\"latest\"]}"
done   # "0x0" on both => still unfunded
```

Funding sources: 421614 via faucet (`spike/DEPLOY.md` §0), 4663 via the
Arbitrum portal bridge (~10 min). Budget: tenths of an ETH cover the whole
session (`docs/gas-report.md` §3).

Anytime (no funding needed): the **Token Sniffer + De.Fi browser
spot-check** on `0x1Cdad396DB64BDa184d5182A97Dd9B3C62100b7D` — record what
each shows about the beacon/hidden-modifier pattern; if either catches it,
add the one-sentence adjustment to `submission/qa-prep.md` Q7. No specifics
may be cited about either tool until this has run.

## 1. Execute the go-list (`demo/runbook.md` §0, steps 1–9, in order)

The repo-side go-list is authoritative; shape summary (details + the
replicas JSON key-mapping live in the runbook):

1. Fund the operator key; `cast wallet new` for the registrar + fund it on
   4663 (0.005 ETH generous).
2. 4663 **replicas deploy BEFORE the core deploy's stage F**:
   `DEPLOY_KEY=… RPC_URL=https://rpc.mainnet.chain.robinhood.com ./scripts/deploy/replicas/deploy.sh`
   → append the logged addresses under `.replicas` in `deployments/4663.json`
   (create file; writer 6/9) + `demo/assets.md`.
3. `DEPLOY_KEY=… ./scripts/deploy/core/deploy.sh --chain 4663` — stage F
   shallow-merges `deployments/4663.json` append-only.
4. Registrar handoff (`transfer_registrar`), validate `registrar()`.
5. `cargo stylus verify` registry + guard on 4663 (fallbacks: Blockscout
   browser UI / `--verifier sourcify`); re-check the explorer pages resolve.
6. ONE worker reconfig re-passing ALL non-empty vars on the command line
   (never wrangler.toml + stale redeploy — it silently blanks live vars):
   `cd scan-backend && npx wrangler deploy --var REGISTRY_ADDRESS_4663:0x… --var DRIFT_EXTRA_TOKENS:0x<replica>,0x<tokenB>`
   (+ the scratch pair only if the e2e scratch activation ran), then
   `wrangler secret put REGISTRAR_KEY` / `wrangler secret put DRIFT_ADMIN_SECRET`.
7. `REGISTRAR_KEY=… OPERATOR_KEY=… bash scripts/seed-demo/seed.sh`
   (idempotent). Fold the 11 receipt actuals into `docs/gas-report.md` §4
   and flip the PASS-PENDING-FUNDS header (runbook §5 drift policy).
8. `npx playwright test --project=live` in `e2e/` once — the honest skips
   flip green = journey-wide regression.
9. `VITE_WALLETCONNECT_PROJECT_ID=… bash demo/serve.sh`; fresh browser
   profile + imported operator wallet (runbook §2 checklist — the name-lock
   item is already checked).

## 2. Task 2 — production state audit (verify, don't rebuild)

```bash
bash scripts/release/production-check.sh | tee submission/production-state-audit-$(date +%F).md
```

Every line must read PASS or a justified N/A (e.g. 421614 mirror N/A if
only 4663 got funded — record it as such, never force it). PENDING-FUNDS
lines remaining here are the audit failing its own gate.

## 3. Task 3 — full demo-arc smoke, one pass, timed

`bash demo/arc.sh` (live) against production URLs; paste the per-beat
wall-clocks into `demo/runbook.md` §5. A failed pass recovers via runbook
§6 (a REVOKED record never re-verifies — re-deploy the replica cast for a
fresh proxy; a smoke re-run is NOT a re-seed). Any 4663-only divergence is
a blocking fix BEFORE recording.

## 4. P receipt (VERIFIED-shaped live scan)

Between takes: `BASE_URL=https://vetted-scan-backend.dujar-coding.workers.dev ./scan-backend/scripts/live-check.sh`
with spaced attempts. When one lands, record it in **step-9's findings**
(plan note 6 item 3 — that file is its home) and add the link to
`submission/README.md`'s package table. Repo-visibility caveat: if the
public-repo decision ever flips to scrub, COPY the receipt into
`submission/` instead of linking the workbench path. If the public RPC
still throttles Cloudflare egress into the demo window, provision the
paid/private endpoint (worker env swap), re-smoke `/scan`, and capture the
receipt there (the RPC decision, plan Revised 2026-09-20 note 2).

## 5. Final Pages deploy (the step-5 bundle, frozen tree, live mode)

Rebuild + deploy from the tagged/frozen commit with the production env:

```
VITE_API_MODE=live  VITE_API_URL=https://vetted-scan-backend.dujar-coding.workers.dev
VITE_REGISTRY_ADDRESS=<4663.json .registry>  VITE_GUARD_ADDRESS=<4663.json .guard>
VITE_WALLETCONNECT_PROJECT_ID=<project id>
```

(`wrangler pages deploy frontend/dist` after `npm run build`.) Append the
writer entry to `deployments/pages.json` — never rewrite step-8's. Note:
live demo beats still run from `demo/serve.sh` locally (the production
bundle ships without the e2e wallet seam by design); the audit's frontend
check verifies the env contract on the deployed URL.

## 6. Recording

Runbook §2 checklist → two ≤5:00 dry runs (`demo/arc.sh`) → the final take
→ cut `demo/demo.gif` from beats 4–6 of the best take, **uncomment the
README demo link** (`README.md`, the `<!-- ![demo arc](demo/demo.gif) -->`
line), commit. Honesty rules bind (runbook §4): degraded takes are
re-taken, never narrated over.

## 7. Freeze + tag

On main, at the final submitted tree (after the gif commit):

```bash
git switch main && git pull
bash scripts/release/cut-release-tag.sh          # clean tree + CI green + clean-checkout build → v1.0.0
git push origin main v1.0.0
```

Blocking fix discovered after this point → fix, review, **v1.0.1** via the
same script (never re-cut v1.0.0), update the submission links.

## 8. Upload (target 2026-10-03 EOD SGT)

`submission/README.md`'s upload checklist: form fields, video hosting,
repo link, live URLs, contract addresses (from `deployments/4663.json`),
confirmation receipt saved into `submission/`.

## 9. Post-submit (task 6)

Findings archived in this step's `findings.md`; residual risks for judges
live in `submission/qa-prep.md` (Q1 carries the erc20() decode; Q2 the
fresh-redeployment lag; Q3 the twins-on-production mitigation). Re-run
step 0's balance check once more and note the remainder in findings.
