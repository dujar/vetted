# Step 5 review — round 1 (2026-09-12)

Reviewed diff `main...step-5-frontend-screens` (30 files, +3180/−18) against plan.md, journeys.md,
spec.md, the three mockups, and knowledge/frontend-stack.md + robinhood-chain.md.

## Verified

- **Scope clean.** `git diff main...step-5-frontend-screens -- packages/shared docs deployments` is
  empty. The only non-frontend edit is the 5-line ci.yml step (`npm ci` in packages/shared inside
  the frontend job) with the comment flagging it to the step-1 owner — exactly the sanctioned
  exception. tsconfig `resolveJsonModule` and vite `globals: true` are frontend-internal and
  documented in findings.md.
- **Verify.md loose ends: all 6 applied as prescribed.** (1) `vetted-shared: file:../packages/shared`
  + lock refresh; (2) dependency-free hash router + App.tsx nav/routes; (3) `WatchdogStats` local in
  api.ts:23, flagged for step 8; (4) `MOCK_WATCHDOG = {runs: 6092, baselinePerDay: 150}` inline
  (spec.md:8/:37 numbers); (5) `KNOWN_TOKENS` constant, mock and live sources iterate the same list
  with per-address getRecord + erc20 symbol; (6) mock renders the revocation-tx drill-in, live
  renders reason + revokedAt + "not indexed yet".
- **Tests are real, not vacuous.** 76/76 green locally (re-ran, not just CI-trusted). They assert
  fixture-derived content ("Buyer blocklist in modifier", "Metadata mimics canonical NVIDIA"),
  evidence hrefs, progress-line order, guard-check order (record→paused→blocklist→impl per
  journeys.md:31-32), first-revert-wins, probe-unavailable→advisory, executor/preview agreement,
  all five verbatim revert reasons, error classification, zero-connector graceful prompt with real
  wagmi in CI, compare deep link, retry recovery, REVOKED red row drill-in open/close.
- **Plan features present:** four verdict states + not-a-contract + RPC-retry + degraded banner;
  power report with evidence links on every line (spec.md:30); depth-boundary notice in the UI
  (VerdictCard.tsx:114-118, tested); watchdog widget reading the 6,092/~150 constants beside the
  verdict; compare mode two cards side by side; swap wrong-network card BEFORE quoting (tested: no
  guard checks shown); funds-never-move wording throughout; registry table with red REVOKED row +
  reason + drill-in evidence, criteria panel (registrar, criteria.md link, on-chain read interface
  from shared constants), skeleton/empty/degraded, roadmap strip; name behind `lib/brand.ts`.
- **Demo arc (spec §6):** every step-5 beat exists — scan, compare deep link, revert + settle,
  upgrade-revocation beat (MOCK_REVOKED_ADDR → GUARD_RECORD_REVOKED), watchdog close, registry as
  primitive. Live-wiring proof is step 8's, per plan task 6 soft-skip (documented).

## Findings

```
[blocking] frontend/src/lib/router.ts:15 (with frontend/src/pages/ScanPage.tsx:42)
  what:      parseHash reads only window.location.hash; the plan's User journey and journeys.md:9
             specify entry deep links as bare query strings — `?addr=0x…` and the compare
             `?addr=0xA…&addr=0xB…`. Those URLs have an empty hash, so getAll("addr") is [] and the
             page renders the hero: the shared-report entry point silently does nothing.
  breaks:    a link pasted exactly as journeys.md:9 specifies shows no scan, no error. The demo arc
             survives (presenter-made `#/?addr=` links work), but the planned entry format does not,
             and findings.md:27 claims "journeys.md's bare ?addr= works once served through the app
             root" — it does not (the query string is not in the hash).
  smallest:  in useHashRoute/parseHash, fall back to window.location.search when the hash carries
             no query (~2 lines, keeps `#/?addr=` working), or fix the journeys format claim AND
             make every shareable link explicitly `#/?addr=`.
```

```
[non-blocking] frontend/src/lib/wallet.ts:115-119 — `previewChecks` is a pure forwarder around
  previewGuard with zero callers, and the `classifyExecuteError` re-export has no importers
  (SwapPage and the tests import both from lib/guard). Delete both lines.
```

```
[non-blocking] frontend/src/pages/ScanPage.tsx:66-73 — the scan effect is keyed on addrKey only, so
  changing the network selector after results render leaves stale cards on screen with no rescan or
  notice. Minor (mock-mode path); consider including chainId in the key.
```

```
[non-blocking] frontend/src/components/VerdictCard.tsx:114-118 — the non-4663 mock response
  (mockData.ts:164-177) returns verdict UNVERIFIED, so the card appends the "This contract is
  off-pattern" depth-boundary banner to a scan of a genuine stock token from another chain. Copy
  imprecision only; the wrong-network notice panel above it states the real situation, and the live
  backend owns this response shape.
```

## Verdict

1 blocking (deep-link entry format + the incorrect handoff claim about it), 3 non-blocking.
Everything else — scope, the six prescribed loose-end fixes, test substance, journeys coverage,
theme/primitive reuse — is merge-quality. Fix the router fallback (or the documented share format),
then this is APPROVED on re-round.

## Round 2 (2026-09-12) — re-check of 3cd9ca6..7b8e656 (HEAD 7b8e656)

Diff touches only frontend src/tests + findings.md — no scope creep. All four round-1 findings
verified fixed, each by the prescribed smallest fix:

1. **[blocking] bare-query deep links — FIXED.** router.ts:22 now takes `search` (wired from
   `window.location.search` in currentRoute) with documented precedence: the hash's own query when
   present, else location.search. Tests: router.test.ts (4 unit cases incl. compare format and
   hash-wins-over-stale-search) + scan.test.tsx component tests proving bare `?addr=0x…` renders
   the verdict (hero absent) and bare `?addr=A&addr=B` renders both compare cards. The incorrect
   findings.md handoff claim is corrected to state both formats work.
2. **[non-blocking] dead forwarders — FIXED.** `previewChecks` and the `classifyExecuteError`
   re-export deleted from wallet.ts; now-unused guard imports cleaned with them.
3. **[non-blocking] stale cards on network switch — FIXED.** Scan effect keyed on
   `${chainId}|${addrKey}`; test rerenders with chainId 421614 and asserts the rescan produces the
   read-only notice rather than the stale 4663 verdict.
4. **[non-blocking] depth-boundary banner on non-4663 scans — FIXED.** Banner gated on
   `response.notice === null`; test asserts the non-4663 scan shows no "off-pattern" claim.

Tests re-run locally: 84/84 green (8 files; matches CI run 34667626718). No new findings.

## Verdict round 2

APPROVED — 0 blocking, 0 remaining notes.
