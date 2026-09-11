# Journeys

Product: Vetted (placeholder) — token verification + guarded swap for Robinhood Chain stock tokens.
Standing rules across all journeys: every verdict line links its evidence on screen; green = verified, red = risk, amber = advisory (the theme's duality); the engine never guesses — missing evidence degrades to UNVERIFIED, visibly.

## J1 — Verify a stock token before acquiring (primary)

**User:** retail holder considering a stock token on Robinhood Chain.
**Entry:** scan page (`/`), or a deep link `?addr=0x…` shared from a report. A compare deep link `?addr=0xA…&addr=0xB…` renders two verdict cards side by side — the demo's impostor-vs-canonical beat.

**Path:**
1. Paste (or deep-link) a token address; the network selector — a UI control, not a wallet connection — defaults to Robinhood Chain · 4663. Scans are anonymous.
2. Progress lines, in order: fetch bytecode → resolve EIP-1967 implementation slot → run probes (`paused()`, buyer blocklist) → fetch the issuer's live canonical list (docs.robinhood.com/chain/contracts).
3. Verdict card: VERIFIED (green) / IMPOSTOR (red) / UNVERIFIED (amber) / REVOKED (red — formerly verified, record revoked since, linking the revocation tx; the same status J3's table shows), plus the power report — hidden blocklist, global pause, upgradeability, transfer hooks — every line with an evidence link (tx / slot / bytecode diff / probe). A VERIFIED token can still carry red power flags; legitimacy ≠ safety is the report's point.
4. Watchdog widget beside the verdict: sequencer-filterer activity (cumulative filterer runs in one RPC read vs the published 6-week baseline expressed as a daily rate — two numbers, no stored history) — the recurring reason to come back.

**Unhappy paths:**
- Address has no code (EOA / empty) → "Not a contract" state, no verdict attempted.
- Selector pointed at another chain (e.g. the Arbitrum Sepolia mirror) → the scan runs read-only against that chain's RPC and states that stock-token verdicts exist only on 4663. No wallet anywhere in J1 — connected-wallet and switch-chain handling live in J2.
- Issuer canonical list unreachable → degraded banner: ground-truth source down, all verdicts drop to UNVERIFIED, stated on screen.
- RPC failure → retryable error state; nothing guessed.
- Contract does not match the Robinhood stock-token pattern → UNVERIFIED + structural heuristics, depth-boundary notice visible.

## J2 — Swap a stock token without getting rugged (guarded swap)

**User:** holder who decided to acquire or exit and wants on-chain protection, not a report.
**Entry:** "Swap guarded" from any verdict card, or `/swap`.

**Path:**
1. Connect wallet; token selectors accept any address (same parity as the scanner).
2. Guard panel previews the exact checks the contract will re-run at execution: live verification record? paused? buyer-blocklisted? implementation matches record?
3. Execute. Genuine + live record → settles, guard receipt shown. Red flag → on-chain revert with the exact deterministic reason surfaced verbatim: `GUARD_NO_RECORD` / `GUARD_RECORD_REVOKED` / `GUARD_PAUSED` / `GUARD_BLOCKLISTED` / `GUARD_IMPL_MISMATCH`.

**Unhappy paths:** wallet on any chain other than Robinhood Chain → switch-to-4663 prompt before quoting; wallet rejection (nothing broadcast); token paused mid-quote → revert reason shown verbatim; gas failure (nothing sent); record revoked between quote and execution → revert (this is the demo's upgrade beat); funds never move on any revert.

## J3 — Consume the registry as a primitive (integrator/dev — the PMF story)

**User:** wallet / aggregator / frontend dev deciding whether to consume the Canonical Registry.
**Entry:** `/registry`.

**Path:**
1. Table of verification records: token, symbol, status, implementation pointer, verifiedAt, registrar. REVOKED rows red with reason and evidence (e.g. "beacon impl changed 2026-09-06 — record stale").
2. Criteria panel: registrar address, link to the published criteria in the repo, and the on-chain read interface a contract can call — "consume it from your contract," not just from this UI.

**Unhappy paths:** issuer canonical list unreachable → same degraded banner as J1; empty registry state; revoked-record drill-in showing the revocation tx.

The demo arc (spec §Scope 6) walks J1 (live scan of an address we did not deploy) → the J1 compare deep link (impostor side-by-side) → J2 revert + settle → the upgrade-revocation beat → J3 close.
