# Step 8 — e2e journeys (J1/J2/J3, unhappy paths included)

status:     ready-to-merge
branch:     step-8-e2e-journeys
reconciled:
deployed:   pages https://vetted-1un.pages.dev (step-5 bundle redeployed 2026-09-21, preview 91d1d61e.vetted-1un.pages.dev) · live worker https://vetted-scan-backend.dujar-coding.workers.dev (step-4's, unchanged)

## What was built

The journey suite in `e2e/` — Playwright, two servers over the real bundle: **mock** (fixture-backed, offline-capable, 21 specs green) and **live** (VITE_API_MODE=live + step-4 worker; 7 specs green, 3 activation-gated skips). J1 full matrix (typed + deep link, both entry formats; VERIFIED/IMPOSTOR/REVOKED/UNVERIFIED; not-a-contract; wrong-network read-only notice + switch-back; degraded banner; RPC_RETRYABLE retry; depth-boundary copy; wallet-free pinned via `window.ethereum === undefined`), J2 (stub EIP-1193 wallet through wagmi's `injected()` seam; wrong-network switch BEFORE quoting; settle + all five `GUARD_*` reasons byte-exact from `vetted-shared`'s `GUARD_REVERT_REASONS`; route-forced wallet-rejection + gas-failure), J3 (table + REVOKED drill-in, criteria panel with `SELECTORS.getRecord`, degraded forced via RPC-failure interception; empty state covered by unit tests). Live-worker payload honesty asserted (watchdog degrade sentinel `runs:0 + provenanceUrl`, never a fabricated VERIFIED without rows). Files: `e2e/{playwright.config.ts,package.json,tsconfig.json,README.md,.gitignore}`, `e2e/fixtures/{constants,deployments,scratch,rpc,stubWallet}.ts`, `e2e/specs/{scan,swap,registry}.spec.ts + {scan,swap,registry}.live.spec.ts`, `e2e/scripts/seed-scratch.sh`. Frontend deltas (sanctioned, below) + the N7 fold-in.

## Where the plan was wrong / the 4 sanctioned extensions (all applied)

1. **Wallet seam** (verify LE 1): `frontend/src/lib/wagmi.ts` adds wagmi's `injected()` under `VITE_E2E_STUB_WALLET=1` — no product build sets it; the documented zero-connector state is untouched.
2. **De-hardcoded 4663** (verify LE 2): `frontend/src/lib/chains.ts` exports `VETTED_CHAIN` (`VITE_CHAIN_ID` override, default = robinhoodChain, byte-identical product). Used by wallet.ts `switchToVetted`, guard.ts + registry.ts live clients, SwapPage wrong-network gate/copy, RegistryPage `rows(VETTED_CHAIN.id)`, App default chain. "Robinhood Chain · 4663" copy is now dynamic — identical when unset.
3. **Live J3 UI path** (verify LE 3): `registry.ts` live source reads `VITE_REGISTRY_TOKENS` (comma-separated addresses, names fall back to live symbol reads); unset → product `KNOWN_TOKENS` (behavior preserved). Scratch bundle gets the seeded tokens baked at serve time from the config's `deployments/*.json` read.
4. **Pages redeploy** (verify LE 4): step-5 bundle built and deployed live; `deployments/pages.json` carries the writer pin + history (also its README row).

Plus **N7** (Revised 2026-09-20 note 1): `api.ts` now re-exports `WatchdogStats` from `vetted-shared` (the canonical home watchdog.ts named); mock provenanceUrl = the published-baseline docs URL; `frontend/tests/api.test.ts` expectations updated (verify LE 5 named it).

## What the next step needs to know

- **verify LE 6 fix honored:** degraded journeys are route-forced — mock uses the `degraded:true` UNVERIFIED fixture addr; live fulfills `/scan` with a `degraded:true` body; J3 degraded forces registry `eth_call` failures (or a bogus registry address would do).
- **verify LE 7 honored:** IMPOSTOR-red is asserted ONLY in mock mode; the gated live-scratch spec asserts replica VERIFIED + twin UNVERIFIED and that twin never shows IMPOSTOR off-4663.
- **verify LE 8 honored:** `e2e/scripts/seed-scratch.sh` wires the J2 settle plumbing (buyer tokenIn balance+allowance, MM tokenOut balance+allowance) — not just registry records.
- **verify LE 9 decision:** task 6 is LOCAL/ON-DEMAND ONLY — no workflow file written (`.github/` outside this step's scope); `npm run setup && npx playwright test` in `e2e/`.
- **verify LE 10 honored:** the deployments loader skips missing/PENDING/partial files; scratch-gated specs `test.skip` with runbook pointers. Activation runbook: `e2e/README.md` (5 steps: funded core deploy → replicas deploy → fill worker `REGISTRY_ADDRESS_46630/421614` + redeploy worker → seed → `--project=live`).
- **Real product defect found + fixed here (owners should know):** pages' default-param sources (`getScanClient()`/`getWatchdogSource()`/`getGuardProbeSource()`/`getRegistrySource()`) returned a NEW instance every render; sitting in `useEffect` deps they re-fired fetch/effects in an infinite loop in the browser (unit tests inject stable props, so 84 tests stayed green while the live app churned). Fixed with lazy module singletons in ScanPage/SwapPage/RegistryPage. Step-5 owner may want to re-home that pattern in lib.
- **Live scratch is double-gated by design:** deployments JSON + worker `record != null` precondition checked at runtime; skips carry the reason. The seed script's `GUARD_*` state ordering respects guard order (mismatch before pause; buyer blocklist on the replica's beacon LAST).
- **Well-known e2e buyer key:** anvil #0 (`0xf39F…92266`, env `VETTED_E2E_BUYER_KEY`) — funds only ever scratch-chain test ETH; never a production key.
- **Registry UI live drill-in still shows "not indexed yet":** the registry UI's live rows keep `revocationTx: null` (step-5 behavior); only `/scan` attaches `revocation_tx` from Revoke logs. Giving the registry table the worker's revocation tx needs a frontend data path — left out of scope.

## Out of scope, left broken

- The worker scratch wrangler vars (`REGISTRY_ADDRESS_46630/421614`) are still empty — filling them requires the funded deploy addresses + a worker redeploy (`scan-backend/` out of scope); runbook step 3.
- Scratch J2 settle/guard-reverts and scratch J1/J3 UI specs remain skipped until funding + seeding (the honest-degrade reality, Revised note 3).
- Mock-mode J2 cannot force wallet-rejection/gas-failure (its executor never touches the wallet) — those are the always-run live specs (route-forced), which is the journeys' "route-intercepted where a live failure cannot be forced" clause.
- App header chip on swap/registry routes shows the static "● Robinhood Chain · 4663" even on a VITE_CHAIN_ID-re-targeted bundle — cosmetic, product default correct.

## Full touched-file list (for the reconcile sweep)

Frontend: `src/lib/{api,chains,wagmi,wallet,guard,registry}.ts`, `src/pages/{ScanPage,SwapPage,RegistryPage}.tsx`, `src/App.tsx`, `src/vite-env.d.ts`, `tests/api.test.ts`. Deployments: `deployments/pages.json`, `deployments/README.md` (writer row). New: `e2e/**` (11 files + lock). NOT touched: contracts/**, scan-backend/**, packages/shared/*, demo/**, .github/**.
