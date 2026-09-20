# Step 8 — End-to-end journeys: one per journey, unhappy paths included

## Resources
- spec:    ../product/spec.md   (demo arc beats spec.md:39)
- journeys: ../product/journeys.md  (the contract: one e2e per journey + every listed unhappy path)
- knows:   ../knowledge/frontend-stack.md, ../knowledge/robinhood-chain.md
- learned: ../step-3-contracts-core/findings.md, ../step-4-scan-backend/findings.md, ../step-5-frontend-screens/findings.md, ../step-6-replica-assets/findings.md
- learned: ../step-1-repo-scaffold/findings.md  (scaffold MERGED at 5e2ff35 — pinned selectors/revert-reason literals, live-probed RPCs, e2e/ reserved)
- learned: ../../spike/findings.md  (step-2 findings — MERGED at 9d8dc30; file lives in the REPO at `spike/findings.md`: calibrated probe targets — blocklist evidence is a BEACON probe — and the live canonical-fetch worker)

> **Revised** — step-1 findings reconciliation (2026-09-11):
> 1. Task 3's byte-exact revert assertions: `packages/shared/abi.ts` exports SELECTORS and GUARD_REVERT_REASONS as pinned literals (kept literal precisely so e2e can use them without a runtime) — import them in the specs; never retranscribe strings or selectors.
> 2. Task 1's fixture loader reads `deployments/*.json` — writers are step 3 (`421614.json`/`46630.json`) and step 6 (replica fields appended to the same files); the scratch-chain RPC (46630 = `https://rpc.testnet.chain.robinhood.com`) is live-probed and pinned in `packages/shared/wire.md` + `frontend/src/lib/chains.ts`.
> 3. `e2e/` exists in the tree (`.gitkeep` added in review round 1) — survives clean checkout; CI will not see these specs per-push (step task 6's on-demand mode is deliberate).

> **Revised** — step-2 + step-5 findings reconciliation (2026-09-19):
> 1. **The frontend has a mock/live env contract** (step 5, MERGED): mock is the default; `VITE_API_MODE=live` + `VITE_API_URL` (→ step 4's worker) + `VITE_REGISTRY_ADDRESS`/`VITE_GUARD_ADDRESS` (→ the step-3/6 scratch deployments in `deployments/*.json`) flips every client live (`frontend/src/lib/api.ts`, `guard.ts`, `wallet.ts`, `registry.ts`). Use it: CI-capable specs can run the whole stack against mock fixtures, and the scratch-chain pass (task 5) runs the same specs with live env — no page edits either way.
> 2. **Known defect inherited from step 5** (its findings' "out of scope, left broken" understates it): `ViemGuardProbeSource.isBlocked` probes the TOKEN with the blocklist selector (`frontend/src/lib/guard.ts:186-188`) instead of the resolved beacon — on genuine-pattern tokens that reverts and degrades to a "probe unavailable" advisory row. Safe (advisory rows never block, spec.md:28), but it means the J1 blocklist evidence row is only real via step 4's server-side beacon probe. Assert blocklist power-report evidence against the `/scan` payload, NOT the frontend preview row; file the frontend fix (probe the beacon, like `resolvedImpl` already does) back to the step-5 owner per this step's scope rule — do not let a green preview assertion canonize the wrong target.

> **Revised** — step-3 + step-4 + step-6 findings reconciliation (2026-09-20):
> 1. **N7 lands here:** the frontend `WatchdogStats` (`frontend/src/lib/api.ts`) still lacks `provenanceUrl` (verified — the field exists in `packages/shared/watchdog.ts` but not the frontend copy). Fold it in as part of the J3 work and assert the sentinel shape: `runs: 0` + `provenanceUrl != null` = count unavailable, never a measured zero — exactly what the live worker serves today.
> 2. **Honest degrade is an outcome, not a flake:** the public 4663 RPC rate-limits Cloudflare's shared egress — live scans oscillate between the DESIGNED `RPC_RETRYABLE` terminal and completed-but-dropped-probes → honest UNVERIFIED with zero fabricated evidence (live-verified 2026-09-20). The scan specs treat `RPC_RETRYABLE`/honest-UNVERIFIED as legitimate: retry per step-4's guidance instead of failing, and when the degrade path IS exercised, assert no ABSENT row with `severity: verified` was fabricated.
> 3. **The live scratch pass is gated on funded runs:** the addresses this suite seeds against are written by funded deploy.sh runs — step 3's core run (421614/46630) AND step 6's replica run (46630), neither executed as of 2026-09-20 (operator key 0 wei on all three chains; first-funded-run routing pinned in step 7's plan). If funding still hasn't landed at step-8 start, the live scratch pass waits on it; mock-mode specs run regardless. Do not hand-forge deployment entries.
> 4. **(judge round-4 gap 1)** The worker's `REGISTRY_ADDRESS_{4663,46630,421614}` wrangler vars (read at `scan-backend/src/lib.rs:107–109`, empty in `wrangler.toml:13–15`) are filled at the deploys: the scratch pair (`46630`/`421614`) from the funded `deployments/*.json` in task 5 below, `4663` by step 7's task 5. While empty, `/scan` serves `record: null` and degrades to canonical-fetch-only — the live J1 REVOKED spec cannot pass and step-9's drift-check beat has no records to poll.

## Stack
Playwright + an injected EIP-1193 stub wallet (provider shim backed by a funded demo key on the scratch chain — 46630, or 421614 if step-2 findings say 46630 gas never materialized) — real signatures against real deployments, no browser-extension flakiness.

## Goal
After this step the merged product is proven as journeys, not units: J1/J2/J3 green against the deployed frontend + live worker + the live step-3/step-6 scratch deployments (46630/421614 — replicas too, from step 6's rehearsal deploys), with every unhappy path in journeys.md exercised (route-intercepted where a live failure cannot be forced). **This step does NOT target 4663** — the first 4663 registry+guard deploy is step 7, which runs in parallel; the live-4663 journey pass is step 9's dry runs + step 10's production smoke. Waiting on 4663 would serialize 8 behind 7 for no behavioral gain.

## Scope
- Creates: `e2e/**`, plus ONE deliberate exception (judge round-4 gap 2, from the 2026-09-20 reconciliation): the one-file `provenanceUrl` fold-in to `frontend/src/lib/api.ts`'s `WatchdogStats` (Revised note 1, task 4) — type field only, no behavior change; no other frontend file is touched.
- Out of scope: fixing product bugs found here (they go back to the owning step as blocked/fix tasks); demo narration (step 9).

## User journey
The journeys are the test matrix. J1 (journeys.md:6–22): happy path typed + deep link; compare deep link — deployed impostor twin vs canonical replica side by side; REVOKED (pre-seeded record, revocation tx evidence linked); not-a-contract; non-4663 read-only notice; degraded banner (intercept `/rhj/assets`); RPC-failure retry (intercept RPC); depth-boundary UNVERIFIED on a non-matching contract. J2 (:24–34): happy settle + each `GUARD_*` revert with the reason asserted byte-exact in the UI against the deployed guard (each reason triggered by its seeded fixture token); wrong-network prompt before quoting; wallet rejection; gas failure. J3 (:36–45): table rows incl. REVOKED red + drill-in; criteria panel; empty + degraded states.

## Screens
N/A — tests over the existing screens.

## Tasks
1. Harness: playwright config (base URL = deployed Pages; env: worker URL, chain), stub-wallet fixture (EIP-1193 shim + funded demo key on the scratch chain), fixture loader reading `deployments/*.json` + `demo/assets.md`.
2. `scan.spec.ts` — J1 per journeys.md:6–22 incl. all five unhappy paths; `RPC_RETRYABLE`/honest-UNVERIFIED handled as legitimate outcomes with a retry policy (Revised 2026-09-20 note 2).
3. `swap.spec.ts` — J2 per journeys.md:24–34; revert-reason strings asserted verbatim (:32).
4. `registry.spec.ts` — J3 per journeys.md:36–45; watchdog rows assert the `provenanceUrl` sentinel shape, folding the field into the frontend type here (Revised 2026-09-20 note 1).
5. Seeded demo state on the scratch chain (the deployments this suite runs against): canonical replica verified, one pre-revoked record, twins unregistered — seeded ad hoc here via a small cast/script with the registrar key from step 3's integration deploys (the polished, idempotent 4663 version lands as step 9's `scripts/seed-demo/`; hand the working version over). Gated on step 3's and step 6's funded runs having produced the scratch deployments (Revised 2026-09-20 note 3). Before the live pass, fill the worker's scratch wrangler vars `REGISTRY_ADDRESS_46630`/`REGISTRY_ADDRESS_421614` from the funded `deployments/*.json` (judge round-4 gap 1 / Revised note 4).
6. Run mode: locally / on-demand first — live chains + rate limits make per-push CI e2e a flakiness tax; a nightly-or-manual workflow is enough inside the hackathon window.

Check: all three specs green against the deployed stack; every demo-arc beat (spec.md:39) is covered by at least one assertion across the three specs.

## Open questions
- Seeding ownership shared with step 9 as in task 5 — no extra tracker edge (step 9 depends on this step anyway).
