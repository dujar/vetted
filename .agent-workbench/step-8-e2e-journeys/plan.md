# Step 8 — End-to-end journeys: one per journey, unhappy paths included

## Resources
- spec:    ../product/spec.md   (demo arc beats spec.md:39)
- journeys: ../product/journeys.md  (the contract: one e2e per journey + every listed unhappy path)
- knows:   ../knowledge/frontend-stack.md, ../knowledge/robinhood-chain.md
- learned: ../step-3-contracts-core/findings.md, ../step-4-scan-backend/findings.md, ../step-5-frontend-screens/findings.md, ../step-6-replica-assets/findings.md

## Stack
Playwright + an injected EIP-1193 stub wallet (provider shim backed by a funded demo key on the scratch chain — 46630, or 421614 if step-2 findings say 46630 gas never materialized) — real signatures against real deployments, no browser-extension flakiness.

## Goal
After this step the merged product is proven as journeys, not units: J1/J2/J3 green against the deployed frontend + live worker + the live step-3/step-6 scratch deployments (46630/421614 — replicas too, from step 6's rehearsal deploys), with every unhappy path in journeys.md exercised (route-intercepted where a live failure cannot be forced). **This step does NOT target 4663** — the first 4663 registry+guard deploy is step 7, which runs in parallel; the live-4663 journey pass is step 9's dry runs + step 10's production smoke. Waiting on 4663 would serialize 8 behind 7 for no behavioral gain.

## Scope
- Creates: `e2e/**` only.
- Out of scope: fixing product bugs found here (they go back to the owning step as blocked/fix tasks); demo narration (step 9).

## User journey
The journeys are the test matrix. J1 (journeys.md:6–22): happy path typed + deep link; compare deep link — deployed impostor twin vs canonical replica side by side; REVOKED (pre-seeded record, revocation tx evidence linked); not-a-contract; non-4663 read-only notice; degraded banner (intercept `/rhj/assets`); RPC-failure retry (intercept RPC); depth-boundary UNVERIFIED on a non-matching contract. J2 (:24–34): happy settle + each `GUARD_*` revert with the reason asserted byte-exact in the UI against the deployed guard (each reason triggered by its seeded fixture token); wrong-network prompt before quoting; wallet rejection; gas failure. J3 (:36–45): table rows incl. REVOKED red + drill-in; criteria panel; empty + degraded states.

## Screens
N/A — tests over the existing screens.

## Tasks
1. Harness: playwright config (base URL = deployed Pages; env: worker URL, chain), stub-wallet fixture (EIP-1193 shim + funded demo key on the scratch chain), fixture loader reading `deployments/*.json` + `demo/assets.md`.
2. `scan.spec.ts` — J1 per journeys.md:6–22 incl. all five unhappy paths.
3. `swap.spec.ts` — J2 per journeys.md:24–34; revert-reason strings asserted verbatim (:32).
4. `registry.spec.ts` — J3 per journeys.md:36–45.
5. Seeded demo state on the scratch chain (the deployments this suite runs against): canonical replica verified, one pre-revoked record, twins unregistered — seeded ad hoc here via a small cast/script with the registrar key from step 3's integration deploys (the polished, idempotent 4663 version lands as step 9's `scripts/seed-demo/`; hand the working version over).
6. Run mode: locally / on-demand first — live chains + rate limits make per-push CI e2e a flakiness tax; a nightly-or-manual workflow is enough inside the hackathon window.

Check: all three specs green against the deployed stack; every demo-arc beat (spec.md:39) is covered by at least one assertion across the three specs.

## Open questions
- Seeding ownership shared with step 9 as in task 5 — no extra tracker edge (step 9 depends on this step anyway).
