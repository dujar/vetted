# demo/video-script.md — the 5-minute take, 1:1 with the runbook beats

Recorded from a live run of `demo/runbook.md` after task 6's dry runs are
green — never from a staged state, never over a degraded take. **The product
name must be locked before recording** (plan open question; the wordmark is
one file, `frontend/src/lib/brand.ts`). Recording checklist: OBS 1920×1080
60fps; fresh browser profile with the funded operator wallet; console pane
visible from beat 6 (the upgrade is driven there); full take, then per-beat
clips cut at the §5 beat boundaries; `demo/demo.gif` for the README comes
from beats 4–6 of the best take.

Speaking pace ≈ 150 wpm; each block below fits its beat's budget with air.

| beat | clock | judged criterion | on screen |
|---|---|---|---|
| 1 | 0:00–0:30 | real problem solving | docs.robinhood.com/chain/contracts |
| 2 | 0:30–1:00 | real problem solving | scan of the issuer's own token |
| 3 | 1:00–1:30 | real problem solving | compare deep link: twin vs replica |
| 4 | 1:30–2:15 | PMF (power disclosure) | replica scan, red power rows |
| 5 | 2:15–3:00 | contract quality (enforcement) | guarded swap: refusal + settle |
| 6 | 3:00–4:00 | innovation (revocation) | upgrade → auto-revoke → refusal |
| 7 | 4:00–4:30 | PMF (primitive) + close | watchdog widget, registry table |

## Beat 1 (0:30) — the problem, on the issuer's own page
> "Robinhood's chain tokenizes stocks. This is the issuer's entire machine
> answer for 'is this token real' — a static web page. It tells humans that
> a matching name and ticker at a different address is not a Robinhood
> Stock Token. No feed, no API, stale on every upgrade. Meanwhile these
> tokens carry powers no holder can see: a per-address blocklist hidden in
> modifiers, a global pause, sequencer-level screening. Holders are the only
> enforcement that exists today."

## Beat 2 (0:30) — live scan, an address we did not deploy
*(open `$APP/#/?addr=0x1Cdad396DB64BDa184d5182A97Dd9B3C62100b7D` — no wallet)*
> "This scanner reads any address on Robinhood Chain. This token is the
> issuer's own — Everpure — deployed before this project existed, and I
> never touched it. VERIFIED: the on-chain registry id matches the issuer's
> live list, fetched at scan time. Every line links its evidence."

If the rate limiter bites, the runbook's retry copy IS the narration — the
scanner retries and says so; a partial scan shows honest UNVERIFIED with
only the evidence that actually landed.

## Beat 3 (0:30) — the impostor, defined by the docs page
*(compare deep link: `$TWIN1` beside `$REPLICA`)*
> "Same name, same ticker, different contract — the docs page's own
> impostor definition, and lookalikes exist in the wild here. The engine
> won't wear the impostor word on a guess: this replica-pair address isn't
> on the issuer's list, so it scans UNVERIFIED, with the depth boundary
> stated on screen. What it does get is enforcement — it has no verification
> record, and the swap contract refuses it at execution. Watch."

## Beat 4 (0:45) — verified is not safe: the power report
*(replica scan; the blocklist row reads PRESENT — pre-staged on the beacon)*
> "VERIFIED is not safe. This token verifies against the issuer's list — and
> carries a per-address blocklist hidden in a beacon modifier. The scanner
> found it where the token's own code doesn't show it: the state lives on
> the beacon, the evidence link points there. Same for the global pause and
> the beacon-proxy upgrade path: one call flips any of them. Legitimacy and
> safety are different answers, and this report gives you both."

## Beat 5 (0:45) — enforcement at execution
*(swap: tokenIn = MSA stand-in; first tokenOut = twin, then = replica)*
> "The guarded swap re-runs every check inside the transaction. First the
> twin: no verification record — reverted on-chain, GUARD_NO_RECORD, funds
> never moved. Now the genuine token: settles, receipt shown. Deterministic
> reverts only — heuristics warn in the report, they never block a swap."

## Beat 6 (1:00) — the beat no incumbent has
*(console: `cast send $BEACON "upgradeTo(address)" $IMPL_V2 …`, then the
manual drift-check POST, then the scan, then the swap)*
> "The upgradeable proxy is the standard here — and it's exactly what
> breaks every static list. Watch: same token address, new implementation,
> one transaction. The backend's drift-watch compares each record's
> implementation pointer against the live beacon — I'm triggering by hand
> what its cron runs every fifteen minutes; on production nobody is here.
> The record revoked itself. The scan now reads REVOKED — with the
> revocation transaction linked. And the swap that settled a minute ago?
> Refused at execution: GUARD_RECORD_REVOKED. Scanners report. This revokes
> and enforces — the registry is load-bearing, not decoration."

## Beat 7 (0:30) — the primitive, and the honest scope
*(watchdog widget; then the registry table and criteria panel)*
> "The chain's own screen — sanctioned-address exclusion at the sequencer —
> is a live counter, not a launch-day stat: our L1 read measured 258,707
> batches, roughly 2,630 delivery-style events per day as of 2026-09-20,
> against the published ~150/day baseline the widget on screen compares —
> provenance linked, one read, no stored history. And the registry is a
> primitive: wallets and aggregators read `getRecord` from their own
> contracts — criteria published in the repo. Scope, stated plainly:
> signature matches on the Robinhood token pattern get full verdicts;
> everything else gets a labeled UNVERIFIED and structural heuristics. It
> never guesses — and where it verified, it enforces."

## Pre-flight read-through (bind at recording)
- [ ] Product name locked (open question — recording waits).
- [ ] No "trust layer", no "first coverage" — GoPlus lists 4663; the line
  is coverage-without-enforcement vs. enforcement (knowledge/goplus-api.md).
- [ ] Twin never implied IMPOSTOR on the live engine; mock fixtures labeled
  as fixtures if shown.
- [ ] Watchdog: measured figures WITH provenance; widget narrated as
  baseline/illustrative (live counter mechanism pending).
- [ ] Deferred-deploy items never implied live: cron is real (*/15) and
  hand-triggering is said out loud; registry drill-in revocation tx is not
  claimed (it narrates from the /scan payload).
- [ ] Degraded takes are re-taken, never narrated over (runbook §4).
- [ ] Q&A ammo rehearsed: false-positive discipline (positive evidence
  only, issuer ground truth); registrar-compromise blast radius (wrong
  verdicts, never fund theft — `docs/threats.md`); EAS comparison (stores,
  doesn't interpret/enforce/revoke); GoPlus depth (risk fields absent when
  `is_proxy: 1` — structurally blind to this pattern).
