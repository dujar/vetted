# Step 5 — Frontend: scan / swap / registry screens from the mockups

## Resources
- spec:    ../product/spec.md   (frontend line spec.md:57; depth-boundary notice spec.md:29; verdict + evidence discipline spec.md:25–30)
- journeys: ../product/journeys.md  (the state matrices: J1 journeys.md:6–22, J2 :24–34, J3 :36–45)
- screens: ../product/screens/scan.html, ../product/screens/swap.html, ../product/screens/registry.html
- theme:   ../product/theme.css, ../product/components.html  (tokens + primitives already coded in step 1)
- knows:   ../knowledge/frontend-stack.md, ../knowledge/robinhood-chain.md
- learned: (no dependency has findings yet — step 5 builds against step-1 fixtures. The registry read ABI is step-1-created: `packages/shared/abi.ts`; step 2's calibrated probe selectors land in that same file's `PROBE_SELECTORS` block at its merge — code against the exported constant, never against step-2 findings directly)

## Stack
Vite 8.3 + React 19.3 + Tailwind 4.3 + wagmi 3.7.7 built-in connectors (decision made in step 1) + viem 2.56.3 for all reads.

## Goal
After this step all three journeys are clickable against fixture data — and live wherever the backend/contracts already exist: every verdict state including REVOKED and every unhappy path renders per the mockups, the compare deep link works, the swap screen handles wallet connect + wrong-network + verbatim revert reasons, and the registry page reads the chain directly via viem.

## Scope
- Creates: `frontend/src/pages/**`, `frontend/src/lib/**`, verdict-specific components, frontend tests. Consumes step-1 theme/primitives/chains without editing them — if a primitive is genuinely wrong, extend in a new file and note it for the step-1 owner.
- Out of scope: backend rules (step 4), contracts (step 3), e2e (step 8); no screens or routes beyond the three mockups.

## User journey
The three journeys ARE the scope — states come from journeys.md, not invented. J1: entry `/` or deep links `?addr=0x…` and `?addr=0xA…&addr=0xB…` (journeys.md:9); network selector is a UI control defaulting to 4663, scans anonymous, wallet-free (journeys.md:12); progress lines in order (:13); verdict card in four states + power report with evidence links (:14); watchdog widget (:15); unhappy: not-a-contract (:18), non-4663 read-only notice (:19), degraded banner (:20), RPC retryable (:21), depth-boundary notice (:22). J2: connect wallet; guard panel previews the exact checks (:31–32); execute → settle with receipt, or on-chain revert with the reason surfaced verbatim (:32); wrong-network → switch-to-4663 prompt BEFORE quoting (:34); confirming / rejected / gas-failure states; wallet rejection; funds-never-move messaging. J3: records table with token, symbol, status, implementation pointer, verifiedAt, registrar — REVOKED rows red with reason + evidence (:42); criteria panel with registrar address, criteria.md link, and the on-chain read interface (:43); loading skeleton, empty state, degraded banner (:45); roadmap strip.

## Screens
Exactly the three mockups; no new surface. Scan = `/`, swap = `/swap`, registry = `/registry` (journeys.md entries).

## Tasks
1. `lib/api.ts` — typed client for `/scan` + `/watchdog` per `packages/shared` wire; **mock mode** (fixture-backed, step-1 golden fixtures) behind `VITE_API_MODE`; live mode = worker URL env. The screens render fully with no backend — that is how this step ships in parallel with step 4.
2. Scan page: network selector (default 4663), address input, progress lines in order, verdict card (four states + not-a-contract + RPC-retry + degraded banner), power report rows with evidence links, compare deep-link mode (two cards side by side), watchdog widget, depth-boundary notice — layout per `screens/scan.html`.
3. Swap page: wallet connect (wagmi), wrong-network card with switch-to-4663 before quoting, guard-preview panel (the four checks, computed client-side via viem reads of registry + staticcalls — registry ABI from `packages/shared/abi.ts`; `paused()` + blocklist probe selectors from its exported `PROBE_SELECTORS` constants, placeholders until step 2's merge-time calibration replaces them — mock-mode tests are fixture-driven and do not depend on calibration; no quote endpoint needed), token selectors accepting any address, execute → confirming / settled(receipt) / rejected(verbatim reason) / gas-failure states; mock-mode execution path over fixtures — per `screens/swap.html`.
4. Registry page: viem direct reads of the deployed registry (address from `deployments/*.json`, ABI from `packages/shared/abi.ts` — created by step 1), table per `screens/registry.html` incl. REVOKED red rows with reason + revocation-tx evidence + drill-in, criteria panel (registrar address, link to `docs/criteria.md` — path reserved by step 7, on-chain read interface snippet), loading skeleton, empty + degraded states, roadmap strip.
5. State-coverage tests (vitest, fixture-driven): one test per verdict state and per unhappy path per screen — the step is done when journeys.md's state matrix is fully covered by tests.
6. Integration smoke vs step-3/4 deployments when they exist (soft — live wiring proof is step 8's job).

Check: `npm run build` + vitest green in CI; mock-mode click-through of all three screens matches the HTML mockups state-for-state.

## Open questions
- Product name is still the "Vetted" placeholder (state.md:40 defers the pick to the coordinator at screens phase) — keep all copy behind one name constant so renaming is mechanical.
