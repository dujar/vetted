# Audit — scope=spec, target=spec.md (round 2 re-audit, 2026-09-11)

VERDICT: spec: 3 gaps

## Round-1 fixes — all 5 verified landed

1. Canonical-list fetch is spike bullet 5 with the pre-committed degraded branch (spec.md:55) and Risk-4 tie-in (spec.md:78). Landed as prescribed.
2. Revocation trigger named: Workers cron polls impl pointer vs live beacon slot, registry-as-state, guard probe covers the poll-to-revoke window (spec.md:35). Landed.
3. "Tooling vacuum" → "verification-and-enforcement vacuum (analytics exist — Bubblemaps, live on 4663)" (spec.md:21). Landed.
4. Out-of-v1 integrations qualified; registry's public read interface kept in v1 with the J3 pointer (spec.md:41). Landed.
5. "Trailing average" cut; watchdog reworded to baseline-constant comparison in both spec.md:37 and journeys.md:15; Workers KV declined. Landed — but see Gap 2 below: the replacement wording keeps a residue of the same defect.

## Gaps

1. spec.md:51,59 — the deploy/gate target "Robinhood Chain testnet" (also "on testnet" spec.md:38,54) is asserted nowhere verified: spec.md:8 defines Robinhood Chain as production chain 4663 (real stock tokens, live impostor, sequencer filterer), scout.md:14 confirms only permissionless deployment on the public chain, and no evidence file names a Robinhood testnet or its chain ID — while state.md:30 (round 3) and journeys.md:12,19,34 route scans and swaps to 4663 with journeys.md:19 saying stock-token verdicts exist only on 4663, so the demo's replica-scan and guarded-swap beats live on a chain the spec never IDs and the journeys never route to; smallest fix: one clause in §Tech Deploy settling it — either "registry + guard + replicas deploy on Robinhood Chain 4663 itself (permissionless, verified)" or the testnet's chain ID + RPC with the demo selector defaults aligned — and, if 4663, add the deploy-impostor-twins-on-production hazard to §Risks.

2. spec.md:37 — "today's runs in one RPC read … no stored history" has no workable mechanism: the sourced measurement is a cumulative nonce read (scout.md:15,29), so "today's" requires a day-boundary delta (stored history — exactly what round-1 gap 5 flagged) or a ranged log query that is not one read and likely exceeds RPC range caps on a ~250 ms-block Orbit chain — the same defect survives in journeys.md:15; smallest fix: reword the first number to "cumulative filterer runs in one RPC read vs the baseline expressed as a daily rate", or commit a Workers KV midnight snapshot and drop "no stored history".

3. spec.md:4 — the one-liner enumerates VERIFIED / IMPOSTOR / UNVERIFIED but omits REVOKED, the fourth verdict state added round 3 (state.md:30; journeys.md:14) and the state the demo's headline beat produces ("registry auto-revokes", spec.md:39) — the elevator pitch is stale against the product's most-novel verdict; smallest fix: add "— REVOKED" to the one-liner's verdict list.

## Holds (checked, green)

- Every state.md Answered item lands in spec.md: Q1 (spec.md:35–36,28), Q2 (spec.md:38), Q3 (spec.md:49–51), Q4 (spec.md:35), Q5 (spec.md:56–57), Q6 (spec.md:37), Q7 (spec.md:59 — modulo Gap 1), Q8 (spec.md:57), H1 (spec.md:10–14,52,76), H2 (spec.md:27,39), H3 (spec.md:39), H4 (spec.md:37,65), H5 (spec.md:29), H6 (spec.md:4,35,43–45,53), Market Q15 (spec.md:71).
- Judgment housekeeping resolved: §Competition no longer "Pending" (spec.md:61–63); Q9–Q15 all adopted or checklist-filed.
- Placeholder grep clean: only "placeholder name" (spec.md:1), waived with a decision point in state.md Deferred; banned phrase "trust layer" absent from product copy (grep: only the ban statement itself, spec.md:4).
- Success line (spec.md:23) observable; demo arc sums to 270 s inside 5 min (spec.md:39); numbers consistent across spec/state/scout/market (4663, 2026-07-01, 6,092≈145/day over 6 weeks, 614 participants, 2026-10-04 23:59 SGT, $40k, 200k gas, day-7 gate).
- No other intra-spec contradiction found: risks 1–5 each have a pre-committed branch; degraded-mode copy is pre-committed rather than silent; registrar key vs degraded human-written records compatible (same key).
