# Step 5 verify — frontend screens plan (2026-09-12)

Checked against main at 96b4c7a (step-1 merged at 5e2ff35). Read: plan.md, all three mockups, theme.css vs coded tokens, journeys.md, spec.md, wire.md, types.ts, abi.ts, fixtures, frontend scaffold (App/main/wagmi/chains/verdicts/components.css), packages/shared package.json + index.ts + roundtrip tests, .github/workflows/ci.yml, step-2/step-4/step-7 plans, step-1 findings, knowledge/frontend-stack.md.

## Hard gates

- **UI without a mockup: PASS.** scan.html (178 lines, empty/scanning/verified/impostor/unverified/revoked/4 error states), swap.html (137 lines, connect/wrong-network/pre-flight/rejected/confirming/reverted×2/settled), registry.html (111 lines, populated/loading/degraded/empty) — opened each; all three cover the screens the plan names.
- **Missing user journey: PASS.** journeys.md J1/J2/J3 carry entry points, step order, and unhappy paths; the plan's journey section derives every state from them, none invented.

## Verified clean (asked-for checks that passed)

- Mockup class names map: every component class the mockups use (.badge.verified/.impostor/.unverified, .report-row, .check-row, .table, .banner.risk/.advisory/.verified, .btn.ghost/.danger, .chip, .panel, .input, .label, .mono/.muted/.faint) exists in `frontend/src/theme/components.css`; product/theme.css and frontend tokens.css are token-identical (diff: zero). Remaining mockup classes are per-page layout blocks defined in each mockup's own `<style>`.
- PROBE_SELECTORS collision with step 2: clean. `packages/shared/abi.ts:57-60` reserves the block with `paused`/`blocklist` keys; step-2 plan (task 5 + Scope) replaces bytes at its merge, keys stable; plan task 3 consumes the exported constant and never inlines bytes. No other shared file is a collision surface.
- Shared export names match the plan's revised notes exactly: REGISTRY_ABI, GUARD_ABI, SELECTORS, GUARD_REVERT_REASONS, PROBE_SELECTORS (abi.ts); VERDICTS, RegistryRecord, ScanResponse, TerminalState, PowerReportRow (types.ts); `verdicts.ts` mirror is at `frontend/src/components/verdicts.ts` as the plan states, with the in-file switch comment.
- Headless tests: `getWagmiConfig()` instantiates walletConnect only when VITE_WALLETCONNECT_PROJECT_ID is set (frontend/src/lib/wagmi.ts:23-24); CI has no such var → zero-connector config, and vite.config.ts pins `test.environment: "jsdom"`. Mock-mode tests stay headless as long as they don't set the env var.
- CI: frontend job (`npm ci` → `npm run build` → `npm test`, .github/workflows/ci.yml) already runs tsc+vite build and `vitest run`; plan adds no CI surface beyond tests/dep changes.
- Line references all resolve: spec.md:57/:29/:25-30/:8/:37, journeys.md:9/:13/:14/:15/:31-32/:34/:42/:45, state.md deferred-name item, docs/criteria.md reserved by step 7 (step-7 plan:18,:30,:35 — "the registry page's criteria panel links exactly this path").
- Stack matches knowledge/frontend-stack.md (wagmi 3.7.7 built-in connectors, AppKit excluded; Tailwind 4 via @tailwindcss/vite; viem for reads; wallet only on the swap beat).

## Loose ends

```
[dangling reference] `vetted-shared` is not importable from the frontend
  where:    plan revised note 1 + task 1 ("switch its imports to vetted-shared") / frontend/package.json
  evidence: frontend/package.json dependencies list only react/react-dom/@tanstack/react-query/viem/wagmi; frontend/node_modules has no vetted-shared link; packages/shared/package.json is named "vetted-shared" (private, main: ./index.ts). CI's frontend job runs npm ci from frontend/package-lock.json — the switched import fails type-check and build.
  fix:      one plan line — add "vetted-shared": "file:../packages/shared" to frontend dependencies and refresh frontend/package-lock.json (TS-source main resolves fine under the existing moduleResolution "bundler"; viem dedupes).
```

```
[missing prerequisite] No router and no shell/nav task — /swap and /registry are unreachable
  where:    plan Screens section ("swap = /swap, registry = /registry") vs scaffold
  evidence: frontend/package.json has no router dep; App.tsx is the placeholder shell ("NOT a screen. Screens arrive in step 5"); main.tsx has no router; all three mockups carry a shared header.bar nav (Scan/Swap/Registry) that no plan task builds. Scope's "Creates" list (pages/lib/components/tests) doesn't cover the App.tsx shell edit either.
  fix:      add a task line: pick routing (react-router-dom, or a small hash router in src/lib) and replace App.tsx with the shared nav + three routes, noting App.tsx/main.tsx as deliberate step-5 edits to the step-1 shell.
```

```
[dangling reference] `/watchdog` typed "per packages/shared wire" — the wire has no watchdog type
  where:    plan task 1 vs packages/shared/types.ts + wire.md
  evidence: types.ts defines ScanResponse only; wire.md documents only GET /scan. The endpoint exists solely as prose in step-4 plan task 5 ("cumulative runs vs baseline constant — two numbers, no stored history"). wire.md's append discipline ("add module files, never edit others'") plus step-5's frontend-only file scope mean step 5 cannot add it to packages/shared either.
  fix:      pin the response shape in the plan (e.g. {chainId, runs, baselinePerDay} per step-4 task 5 + journeys.md:15) and declare that type locally in frontend/src/lib/api.ts, flagged for step-8 reconciliation.
```

```
[orphan input] No watchdog fixture exists for mock mode
  where:    plan task 1 ("mock mode … step-1 golden fixtures") vs packages/shared/fixtures/
  evidence: fixtures/ holds only verdict ScanResponses (4) and guard reverts (5); no watchdog JSON. Adding one to packages/shared/fixtures would break the shared CI job — tests/roundtrip.test.ts readdirSync's every fixture and parses it as ScanResponse/GuardRevert.
  fix:      state in task 1/2 that mock watchdog numbers are inline constants in frontend/src/lib (6,092 runs / ~150/day per spec.md:8 and :37).
```

```
[orphan input] Registry table rows have no data source (token list, symbol, and the page's mock mode)
  where:    plan task 4 / journeys.md:42 / screens/registry.html (4 tokens incl. REVOKED row)
  evidence: REGISTRY_ABI is point-lookup only — getRecord(address), no enumerate, no events (abi.ts:10-14) — so nothing on chain produces the list of tokens to table. No registry fixture exists (fixtures/ are ScanResponse + guard-revert shapes); deployments/ has only infra JSONs until step 3 (421614/46630) and step 7 (4663) per deployments/README.md. RegistryRecord also has no symbol field, and task 4 — unlike tasks 1/3 — never states a mock-mode path, though task 5 requires fixture-driven registry tests.
  fix:      extend task 4: registry renders a known-token list constant in frontend/src/lib (mock + live identical), driving per-address getRecord reads plus viem erc20Abi symbol reads; VITE_API_MODE=mock serves it from that list so task 5's tests are fixture-driven without step 3.
```

```
[dangling reference] Registry revocation-tx evidence has no source in the pinned ABI
  where:    plan task 4 ("REVOKED red rows with reason + revocation-tx evidence + drill-in") / journeys.md:45
  evidence: RegistryRecord (types.ts:28-36) carries reason + revokedAt but no tx hash; REGISTRY_ABI_SIGNATURES has no events, so a viem read cannot derive the revoking tx. Only ScanResponse.revocationTx carries it (fixtures/verdict/REVOKED.json:34).
  fix:      pick one in the plan: mock mode renders the fixture's revocationTx and live mode links reason + revokedAt without a tx hash until step 3 emits a Revoked event (their file — coordinate now), or step 5 links the explorer address page instead. Currently the implementer must guess.
```

## Notes

- The product-name deferral is handled (name constant per the plan's open question) — not counted.
- Deep-link compare (`?addr=A&addr=B`) needs `URLSearchParams.getAll`, not `.get` — implementation detail, journeys.md:9 already specifies the behavior; not counted.
```
