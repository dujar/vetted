# Audit — journeys (target: journeys.md), round 2, 2026-09-11

Round-1 gaps — all four verified fixed:
1. FIXED — REVOKED verdict added to J1 step 3 (`journeys.md:14`), red, links the revocation tx, cross-referenced to J3's table (`journeys.md:42`); no more VERIFIED-rescan vs `GUARD_RECORD_REVOKED` contradiction.
2. FIXED — network chip is a UI selector defaulting to 4663, scans anonymous (`journeys.md:12`); J1 is wallet-free with wrong-chain handling explicitly moved to J2 (`journeys.md:19`), matching spec.md:57–58.
3. FIXED — J2 wrong-network unhappy path added: switch-to-4663 prompt before quoting (`journeys.md:34`).
4. FIXED — compare deep link `?addr=0xA…&addr=0xB…` added to J1's entry (`journeys.md:9`), giving the demo's impostor-vs-canonical beat a journey mandate, reflected in the arc line (`journeys.md:47`).

Full re-check, round 2:
- Goals → journeys: all v1 scope items map — scan→J1, guarded swap→J2, Canonical Registry→J3, watchdog widget→J1 step 4 (`journeys.md:15`, matches spec.md:37 "on the scan page"); demo arc beats 2–7 each land in a journey (`journeys.md:47` vs spec.md:39); retail and integrator/dev personas both covered. No dropped goals.
- Entry / ordered path / unhappy path: J1 (`journeys.md:9,11–15,17–22`), J2 (`journeys.md:27,29–32,34`), J3 (`journeys.md:39,41–43,45`). All present.
- Dead ends: no-code address (`journeys.md:18`), wrong chain in J1 and J2 (`journeys.md:19,34`), canonical-list outage → all-UNVERIFIED banner, matching the pre-committed degraded branch (`journeys.md:20,45` vs spec.md:55), RPC failure (`journeys.md:21`), off-pattern contract → UNVERIFIED + heuristics with depth-boundary notice (`journeys.md:22`, matches spec.md:29), wallet rejection / gas failure / mid-quote pause / revoke-between-quote-and-execution, funds never move (`journeys.md:34`). Unknown-token and empty-registry states stated (`journeys.md:22,45`).
- Revert taxonomy is 1:1 with spec's deterministic flags: `GUARD_NO_RECORD`/`GUARD_RECORD_REVOKED`/`GUARD_PAUSED`/`GUARD_BLOCKLISTED`/`GUARD_IMPL_MISMATCH` (`journeys.md:32`) vs spec.md:28 — five for five.
- Screens: `/`→`screens/scan.html`, `/swap`→`screens/swap.html`, `/registry`→`screens/registry.html` — all exist and open; no orphan screens (screens/ contains exactly these three, each reached by a journey).
- Landing points: J1 ends on the scan page with verdict card + watchdog; J2 ends "guard receipt shown" or revert reason surfaced verbatim; J3 ends on the criteria panel with the on-chain read interface. No journey ends nowhere.
- Consistency with spec: verdict color duality (`journeys.md:4`) matches theme discipline; REVOKED is sanctioned by spec.md:35 (`revoke`) and the demo beat (spec.md:23, spec.md:39 beat 6) — not a contradiction of the spec.md:4 headline three; "trust layer" and "attestation" absent from journeys.md (grep clean); "placeholder" at `journeys.md:3` is the sanctioned name placeholder (state.md Deferred).
- Judgment round-1 holes with journey surface: H2 live non-self-deployed scan, H3 upgrade-revocation beat, H4 watchdog retention, H5 depth-boundary notice — all present in journeys.md (`journeys.md:47,34,15,4,22`). Nothing unanswered.

Verdict: GREEN — journeys. Round 1's four gaps are fixed as ordered; no new gaps, dead ends, or spec contradictions introduced by the round-3 edits.
