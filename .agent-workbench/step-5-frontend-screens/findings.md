# Step 5 — frontend screens (scan / swap / registry)

status:     ready-to-merge (review pending — see below)
branch:     step-5-frontend-screens (pushed; CI frontend job expected green — tsc + build + 76 tests pass locally, shared suite 15/15 untouched)
deployed:   not deployed (pages redeploy is step 8/10's job)

## What was built

All three mockup screens are clickable end-to-end against fixture data, and live wherever the backend/contracts already exist. `frontend/src/lib/` is the seam layer: `api.ts` (typed `ScanClient`/`WatchdogSource`, `Mock*` fixture-backed vs `Fetch*` live clients, env switch `VITE_API_MODE`/`VITE_API_URL`, mock by default), `mockData.ts` (golden fixtures imported verbatim from `packages/shared/fixtures/` + mock-only sentinels), `guard.ts` (four-check preview in guard order via a `GuardProbeSource` transport seam; verbatim revert reasons from the shared `GUARD_REVERT_REASONS` constant; error classifier for rejected/gas/reverted), `registry.ts` (getRecord decode + known-token table source), `wallet.ts` (wagmi adapter: connect / switch-to-4663 / simulate-then-send execute), `router.ts` (dependency-free hash router), `brand.ts` (name constant). `frontend/src/pages/` + `App.tsx` shell render the mockups' header/nav plus Scan `/`, Swap `#/swap`, Registry `#/registry`; `VerdictCard`/`WatchdogWidget`/`ProgressLog` are new components; step-1 primitives consumed unedited. `components/verdicts.ts` now re-exports the union from `vetted-shared` (mirror deleted, per plan revised note 1). Tests: `frontend/tests/*` — one test per journeys.md state per screen (scan 14, swap 12, registry 7, api 14, guard 21, app 5, plus step-1's 3) — all 76 green; `npm run build` green.

## Where the plan was wrong

- No router existed (verify loose end 2): picked the hash-router option (no new dep; survives static Pages hosting without SPA-fallback config). `App.tsx` is a deliberate step-5 edit; `main.tsx` needed no change.
- `vetted-shared` was not importable (verify loose end 1): added `file:../packages/shared` dep + lock refresh; also had to add `resolveJsonModule` to frontend tsconfig for the fixture imports (not in the fix as written, same seam).
- `/watchdog` had no wire type (verify loose end 3): `WatchdogStats {chainId, runs, baselinePerDay}` declared locally in `api.ts`, flagged for step-8 reconciliation. Mock numbers inline (loose end 4).
- Registry had no data source (verify loose end 5): known-token list constant in `mockData.ts` (mock + live identical) drives per-address getRecord + erc20 symbol reads.
- Revocation-tx evidence (verify loose end 6): mock mode renders the fixture-style tx (drill-in); live mode shows reason + revokedAt and "not indexed yet" until step 3 emits a Revoked event — step 3/8 should coordinate there.
- Registry/guard live addresses: plan said `deployments/*.json`, but those files don't exist until steps 3/7 and importing a missing file breaks the build — used `VITE_REGISTRY_ADDRESS`/`VITE_GUARD_ADDRESS` env instead (documented in vite-env.d.ts).
- Impostor fixture shares its address with the REVOKED fixture; mock mode serves IMPOSTOR there and defines a mock-only revoked address (`MOCK_REVOKED_ADDR`), fixtures untouched.

## What the next step needs to know

- The scan-transport seam is `ScanClient` in `lib/api.ts` — step 4's real backend drops in behind it; nothing else changes. Same for `GuardProbeSource`/executor (step 2's calibrated `PROBE_SELECTORS` bytes already flow through `ViemGuardProbeSource` via the exported constant).
- Mock sentinels for demo/tests: `MOCK_VERIFIED_ADDR` (NVIDIA), `MOCK_IMPOSTOR_ADDR` (twin), `MOCK_REVOKED_ADDR` (upgrade beat), `MOCK_NOT_CONTRACT_ADDR`, `MOCK_RPC_ERROR_ADDR`, plus guard-outcome tokens `MOCK_PAUSED_ADDR`/`MOCK_BLOCKLISTED_ADDR`/`MOCK_IMPL_MISMATCH_ADDR` in `lib/mockData.ts`.
- Registry no-record rule: zero registrar or status u8 outside {0,1} decodes to no-record (wire.md; guard order then yields `GUARD_NO_RECORD`).
- Live probe failures degrade to "probe unavailable" advisory rows — heuristics never revert or block (spec.md:28); only real guard reverts refuse.
- Deep links are hash-based: `#/?addr=0x…`, compare `#/?addr=A&addr=B` (URLSearchParams.getAll); journeys.md's bare `?addr=` works once served through the app root.
- vitest config got `globals: true` (testing-library auto-cleanup) — shared-package tests unaffected (own config).
- Renaming the product: edit `lib/brand.ts` only.

## Out of scope, left broken

- The impostor verdict card renders its comparison as check/result rows from `powerReport` (wire-faithful), not the mockup's 4-row Canonical-vs-Suspect table — the wire has no canonical-name/symbol/impl-hash fields; the demo beat is the two-card compare deep link.
- Network selector sits in the header on scan route (mockup shows it as a chip there); swap/registry show the static chip per mockup.
- `MockGuardProbeSource.isBlocked` ignores the buyer (per-token sentinel) — fine for fixtures; live source passes the buyer through.
- Live REVOKED scan without a `revocationTx` renders "not indexed yet" — closes automatically when step 3 emits Revoked events.
- Integration smoke vs step-3/4 deployments (plan task 6): soft-skip — neither deployment exists yet; this is step 8's live-wiring proof.
