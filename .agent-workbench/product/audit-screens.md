# Audit — screens (round 2)

target: `.agent-workbench/product/screens/` (scan.html, swap.html, registry.html) vs journeys.md; theme.css + components.html checked for coherence.
Round-1 check: all 10 gaps verified fixed in the files — registrar column (registry.html:39), registrar address (registry.html:52), scan wrong-network card (scan.html:169), swap confirming state (swap.html:78), rejection/gas card (swap.html:67), registry loading skeleton (registry.html:67), roadmap strip (registry.html:61), `.eviline` gone (grep empty), `chip mono` on all three headers (scan.html:30, swap.html:29, registry.html:27), input `min-width:32ch` (components.html:54).

## Gaps

- journeys.md:34 — swap.html has no wrong-chain state: J2's "wallet on any chain other than Robinhood Chain → switch-to-4663 prompt before quoting" appears nowhere among swap.html:33-123's seven states, though state.md:30 claims it was added — add one panel reusing scan.html:169's pattern: "wallet on Arbitrum Sepolia — switch to Robinhood Chain · 4663 to quote; funds never move on a revert".
- journeys.md:14 — scan.html:144 REVOKED banner shows `revocation tx 0xf4c0…71b8` as plain text; the standing rule (journeys.md:4: every verdict line links its evidence) and registry.html:43 / swap.html:110 both link this same evidence — wrap the tx hash in `<a href="#">`.
- registry.html:13,75-77 — the new loading state introduces the only raw pixel sizes outside theme.css (`.skel { height: 11px }`; skeleton widths 52–96px; the `max-width: 980px/760px` wraps stay exempt as round-1 scaffold) — smallest fix: `height: var(--fs-0)` and `ch` measures, or move `.skel` into theme.css beside `.report-row`.

## Confirmed present (asked questions, round 2)

- Required verdict states all on screen: VERIFIED (scan.html:60), IMPOSTOR (scan.html:96), UNVERIFIED (scan.html:120), REVOKED (scan.html:134, red badge + revocation line, same address as the VERIFIED card — correct, it is the canonical post-upgrade), errors ×4 (scan.html:150-173).
- Depth-boundary disclaimer verbatim in the UNVERIFIED state (scan.html:129): signature-match on the Robinhood pattern vs advisory-only heuristics.
- Demo arc beats all present: live scan of any address, impostor side-by-side (scan.html:106), power report with red flags on the genuine token (scan.html:71-76, "Verified ≠ safe"), impostor revert GUARD_IMPL_MISMATCH (swap.html:97), upgrade beat GUARD_RECORD_REVOKED (swap.html:108) + auto-revoked registry row (registry.html:43), watchdog with 6-week baseline (scan.html:82-83), consume-panel + roadmap close (registry.html:56-61).
- J1 progress lines match journeys.md:13 order (scan.html:52-55); guard pre-flight shows all four execution checks (swap.html:57-60); J3 criteria panel has registrar address, repo criteria link, and the on-chain read interface (registry.html:52-59).
- Theme: zero hardcoded colors outside theme.css (grep clean); zero `<script>`/handlers (grep clean); no lorem/Foo/xxx, no "attestation", no "trust layer" (grep clean — `class="bar"` hits are the header rule); duality holds (green=verified incl. the on-canonical network chip, red=risk, amber=advisory incl. the mirror chip at components.html:38, cyan=interactive only).
- No orphan screens: all three reached from journeys (nav + scan.html:76,115,145 → swap; registry.html:41-44,105 → scan).

VERDICT: screens: 3 gaps
