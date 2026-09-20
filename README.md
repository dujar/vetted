# Vetted

Token verification + guarded swap for Robinhood Chain stock tokens. Paste any
token address on Robinhood Chain (4663) and get an evidence-linked verdict —
VERIFIED / IMPOSTOR / UNVERIFIED / REVOKED — plus a plain-language power
report (hidden blocklist, global pause, upgradeability), and a guarded swap
that refuses red-flagged tokens on-chain at execution time — including one
that was verified yesterday and beacon-upgraded since. Nothing guessed:
missing evidence degrades to UNVERIFIED, visibly, and every verdict line
links its evidence.

Built for the Arbitrum Open House Singapore Online Buildathon (deadline
2026-10-04 23:59 SGT). **Judged-criteria mapping:** `docs/criteria.md`.

## Security note — read before trusting a swap

**Disclosure: the guard's ERC-20 check decodes any `true`-ish returndata**
(`contracts/core/guard/src/lib.rs:497`). The `erc20()` helper treats *any*
return payload whose 31st byte is set as a boolean `true` — a malicious token
can return junk (e.g. 64 arbitrary bytes) that decodes as "approved", so the
commit-custody `transferFrom` can silently not move funds into escrow for a
token that lies this way. **Worst case: a phantom active order for the buyer
who chose that token — the later `execute()` then reverts whole, and no funds
move.** It cannot steal or strand a counterparty's funds; the failure is
visible as a reverted swap. Found in review, disclosed here, and deliberately
not patched at the release freeze (a decode tightening is a behavior change;
product code is frozen except blocking fixes). It is first on the Q&A list in
[`submission/qa-prep.md`](submission/qa-prep.md).

**Known limits — what the live URLs actually show:**

- **Depth boundary, stated in the UI:** full verdicts require a signature
  match on the known Robinhood stock-token pattern; every other contract gets
  UNVERIFIED plus labeled advisory heuristics. The tool never claims to scan
  any token deeply, and never guesses.
- **Watchdog widget ships the degrade sentinel by design** (`runs: 0` +
  `provenanceUrl` means "count unavailable", never a measured zero): no L1
  read reconciles with the published six-week figures yet, so the widget
  compares against the published ~150/day baseline and the measured figures
  (with provenance) live in the demo narration and `docs/threats.md`.
- **Gas actuals are PASS-PENDING-FUNDS** (`docs/gas-report.md`): sizes,
  data fees and the receipt methodology are committed (§3); the per-row
  `gasUsed` table (§4) is filled only by the first funded run's receipts —
  the receipts are never invented.
- **Registry revocation drill-in can show "not indexed yet"**: `/scan`
  attaches the revocation tx from Revoke logs; the registry table's UI data
  path does not yet — a stated indexing gap, never claimed as live evidence.
- **Unfunded-regime items (pending one funded operator run —**
  [`submission/post-funding-checklist.md`](submission/post-funding-checklist.md**):**
  the 4663 core contract deploy + `deployments/4663.json`, the frontend live
  env (`VITE_REGISTRY_ADDRESS` / `VITE_GUARD_ADDRESS`), the one-worker
  registrar/drift handoff, and the VERIFIED-shaped live scan receipt. The
  demo arc, contracts, engine and harness are complete and rehearsed; the
  gas report and this note carry the honest PENDING markers until the run
  lands (go/no-go 2026-09-29).

## Architecture

```
                 docs.robinhood.com/chain/contracts
                 (the issuer's canonical list — ground truth, fetched at scan time)
                               │
 browser ── GET /scan?chainId&addr ──▶ Cloudflare Worker (Rust) ──▶ Robinhood Chain RPC (4663)
   │                                    · EIP-1967 beacon/impl resolution
   │  GET /watchdog?chainId             · calibrated probes: paused() → proxy,
   │    sequencer-filterer counter        isBlocked(buyer) → resolved BEACON
   │    (one L1 RPC read)               · depth boundary: full verdicts only on a
   │                                      genuine-pattern signature match; else
   │  POST /registry/drift-check         honest UNVERIFIED + advisory heuristics
   │    the same walk the */15 cron    · drift-watch: record impl ≠ live beacon
   ▼                                    impl → revoke (cron or manual trigger)
 Canonical Registry (Stylus) ◀── registrar writes only (verify / revoke)
   ▲        │  getRecord() — the public read interface other contracts consume
   │        ▼
 Guarded Swap (Stylus) — commit/execute re-checks record + live probes in-tx;
 deterministic reverts only: GUARD_NO_RECORD / GUARD_RECORD_REVOKED /
 GUARD_PAUSED / GUARD_BLOCKLISTED / GUARD_IMPL_MISMATCH
```

- **Contracts** (`contracts/core/`, Rust/Stylus): Canonical Registry +
  Guarded Swap. Invariant/fuzz evidence + gas receipts: `docs/gas-report.md`;
  published criteria + threat model: `docs/criteria.md`, `docs/threats.md`.
- **Scan backend** (`scan-backend/`, Cloudflare Workers in Rust): stateless
  verdict engine — no database; the registry is the state.
- **Frontend** (`frontend/`, Vite 8 + React 19 + Tailwind 4 + wagmi 3):
  scan / swap / registry screens + watchdog widget. Mock mode by default;
  `VITE_API_MODE=live` + `VITE_API_URL` + `VITE_REGISTRY_ADDRESS` /
  `VITE_GUARD_ADDRESS` re-point it at the deployed contracts.
- **Wire contract** (`packages/shared/wire.md`): types + ABI + golden
  fixtures, round-tripped in TS and Rust, CI-enforced. Registry read ABI +
  selector constants: `packages/shared/abi.ts`.

## Quickstart

```bash
./scripts/toolchain.sh                                  # Rust stable + wasm32 target + cargo-stylus 0.10.9

cd contracts && cargo test --workspace                  # contracts (native stylus test host)
cd contracts && cargo build --workspace --target wasm32-unknown-unknown

cd scan-backend && cargo test                           # backend unit tests (pure route fns)
cd scan-backend && npx wrangler deploy                  # deploy worker

cd frontend && npm ci && npm run build && npm test      # frontend
cd packages/shared && npm ci && npm test && cargo test  # wire-contract round-trips, both sides

cd e2e && npm run setup && npx playwright test          # journeys (mock project is offline-capable)
```

## The demo

The 5-minute arc — docs-page problem → live scan of an address we did not
deploy → impostor twin compare → power report → guarded swap → **beacon
upgrade → registry auto-revokes → the guard refuses the formerly verified
token** → watchdog + registry-as-primitive close — is scripted start to
finish in [`demo/runbook.md`](demo/runbook.md) (per-beat URLs, budgets, and
pre-written fallbacks), driven by `demo/arc.sh` (scan retries, the upgrade
beat end-to-end, wall-clock), and narrated 1:1 in
[`demo/video-script.md`](demo/video-script.md). Demo recording (lands with
the final take; uncomment then — links must resolve at submission):

<!-- ![demo arc](demo/demo.gif) -->

**Demo-cast labeling:** the replica tokens in `contracts/replicas/` (the
"Aurelia Industries" cast, manifest in [`demo/assets.md`](demo/assets.md))
are demo artifacts — self-deployed on the public chain for the live beats,
carrying no liquidity and no holder base. The tool's depth boundary marks
the impostor twins UNVERIFIED (never a guessed IMPOSTOR), and the guard
refuses them on-chain at execution — no verification record, no settlement.
The decoys double as live evidence.

## Layout

| path | what | owner step |
|---|---|---|
| `contracts/` | Stylus contracts — cargo workspace, one `Stylus.toml` per crate | 3 (core), 6 (replicas), 7 (hardening + 4663) |
| `contracts/core/hello/` | placeholder crate proving the toolchain harness (entrypoint + native test) | 1 |
| `scan-backend/` | Cloudflare Workers in Rust — verdict engine | 1 (scaffold), 4 |
| `frontend/` | Vite 8 + React 19 + Tailwind 4 + wagmi 3 | 1 (scaffold/tokens/primitives), 5 (screens) |
| `packages/shared/` | the wire contract: types (TS + Rust), registry ABI, golden fixtures | 1 |
| `scripts/` | toolchain + deploy scripts; `scripts/seed-demo/` seeds the 4663 demo state (idempotent, registrar-signed) | 1, 3, 6, 9 |
| `e2e/` | end-to-end journeys (Playwright; mock + live projects) | 8 |
| `demo/` | demo assets + narration + the 5-minute-arc harness | 6, 9 |
| `deployments/` | append-point JSONs, one per deploy target — written only by deploy steps | deploy steps |
| `.agent-workbench/` | build plans, knowledge, step state | the build system |

Build order and current step status: `.agent-workbench/step-feature-state.md`.

## CI

`.github/workflows/ci.yml` — six jobs: contracts (wasm build + native
tests), replicas (fingerprint + behavior), mock-token, scan-backend, shared
(both round-trips), frontend (tsc + build + vitest).
