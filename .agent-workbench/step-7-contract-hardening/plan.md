# Step 7 — Contract hardening + quality evidence (the judged criterion)

## Resources
- spec:    ../product/spec.md   (judging weights spec.md:69; criteria-published requirement spec.md:35; ≤200k budget spec.md:51; risks spec.md:73–80)
- knows:   ../knowledge/stylus-toolchain.md (verify + reproducible builds :33–36), ../knowledge/robinhood-chain.md (96 KB limits :38), ../knowledge/contract-verification.md (explorer recipes + the 4663 Cloudflare-challenge footgun :19–27 — applies to any verify path, not just forge)
- learned: ../step-3-contracts-core/findings.md  (build notes, gas actuals, integration harness)
- learned: ../step-1-repo-scaffold/findings.md  (scaffold MERGED at 5e2ff35 — deployments writer pins, live-probed 4663 RPC)
- learned: ../../spike/findings.md  (step-2 findings — MERGED at 9d8dc30; file lives in the REPO at `spike/findings.md`: Stylus ACTIVATION proven on 4663, ≤200k gas receipt still pending on funding (`spike/DEPLOY.md`); calibrated probe targets)

> **Revised** — step-1 findings reconciliation (2026-09-11): this step is the pinned writer of `deployments/4663.json` per `deployments/README.md` (append-only — never rewrite step 3's scratch entries), and the 4663 RPC endpoint is already live-probed and pinned in `packages/shared/wire.md` chains table + `frontend/src/lib/chains.ts` — use it for the deploy `--endpoint`; no re-probing.

> **Revised** — step-2 + step-5 findings reconciliation (2026-09-19):
> 1. **The 4663 deploy outputs feed the frontend env contract**: `VITE_REGISTRY_ADDRESS`/`VITE_GUARD_ADDRESS` are set from `deployments/4663.json`'s addresses (Pages env at steps 8/10; the frontend's live mode errors already point there — frontend `src/lib/guard.ts`/`wallet.ts`/`registry.ts`). Record both addresses in findings explicitly so the env handoff is one lookup.
> 2. **Gas report comparison base** (task 2): step 2's spike receipt may still not exist (PASS-PENDING-GAS — blocked on testnet funding, operator runbook `spike/DEPLOY.md`; activation itself IS proven, live `cargo stylus check` exit 0 on 4663). Step 3's integration receipts (421614 + 46630) are the committed live actuals to diff against; if the spike receipt lands via DEPLOY.md in the meantime, fold it in. Include the guard's beacon staticcall in the measured path — the blocklist probe targets the resolved beacon, not the token (calibrated, `packages/shared/abi.ts:49-63`).

## Stack
Same as step 3 (or the Solidity branch of it).

## Goal
After this step, the contract surface is defensible to a judge in five minutes: fuzz/invariant-tested, gas-budgeted with a committed report, documented, trust-modeled, deployed on 4663 and source-verified — the "smart contract quality" criterion answered with artifacts, not prose.

## Scope
- Touches: `contracts/core/**` (tests + docs only — no behavior change without a coordinator flag), `docs/` (`criteria.md`, `threats.md`), gas report artifact, the 4663 mainnet deploy of core + verification.
- Out of scope: any feature change; backend; frontend; replicas.

## User journey
N/A.

## Screens
N/A.

## Tasks
1. Fuzz + invariant tests (proptest/arbitrary over the native host): registrar authority is the only write path; record transitions are total and monotone per the rules; guard — every revert path leaves zero state change and zero balance delta; check order is deterministic (an earlier reason is never masked by a later one); no path exceeds the probe gas budget.
2. Gas report: re-measure full guard execution + probes (incl. the beacon blocklist probe — Revised note 2) on 421614 and 4663 receipts; commit `gas-report.md` with the ≤200,000 assertion (spec.md:51) and drift vs the step-3 integration actuals explained (the step-2 spike receipt if the DEPLOY.md operator run has landed by then; Revised note 2).
3. Docs: rustdoc complete on public surfaces; `docs/threats.md` — trust model (registrar key compromise blast radius, why single-registrar is acceptable for v1, revocation-as-safety framing); `docs/criteria.md` — the published verification criteria (spec.md:35 requirement; the registry page's criteria panel links exactly this path).
4. Fresh-eyes review pass: reentrancy posture (external calls are staticcalls during checks; effects ordering), 96 KB runtime / 192 KB init headroom (knowledge robinhood-chain.md:38), overflow posture, error-taxonomy completeness vs the five guard reasons.
5. 4663 mainnet deploy of registry + guard via the same `scripts/deploy/core/`; **registrar-key handoff first:** generate the production registrar key, set it as the backend Worker's `REGISTRAR_KEY` secret, and deploy with registrar = that address (or `registrar-transfer` to it immediately after deploy) — step 4's drift-revokes sign with `REGISTRAR_KEY`, and a mismatch makes every revoke revert as a non-registrar write, killing the demo's upgrade→revocation beat (scratch/integration deploys keep throwaway keys). `cargo stylus verify` on 4663 and the 421614 mirror (Blockscout source-verified; explorer behaviors and fallbacks — mainnet `/api` may sit behind a Cloudflare bot challenge server-side, browser-UI verification passes — in knowledge/contract-verification.md:19–27); append `deployments/4663.json`.
6. `findings.md`: what hardened, what was found, what was consciously accepted.

Check: fuzz suites green in CI (seeded ≥10k cases each); `gas-report.md` committed with receipt links; verified contract pages live on the 4663 explorer; `docs/criteria.md` resolves at the path step 5's criteria panel links.

## Open questions
- None blocking. If fuzzing surfaces a design change rather than a fix (e.g. check reordering), stop and flag the coordinator before touching shared semantics.
