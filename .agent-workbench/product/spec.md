# Spec — Vetted (placeholder name; coordinator picks final at screens phase) — DRAFT, not approved

**Hook:** "Is this stock token real, and what can it do to me?"
**One-liner:** Paste any token address on Robinhood Chain and get an evidence-linked verdict — VERIFIED / IMPOSTOR / UNVERIFIED / REVOKED — plus a plain-language report of what the contract can do to your balance, and a guarded swap that refuses red-flagged tokens at execution time. ("Trust layer" wording is banned everywhere — Blockaid's homepage phrase, judgment H6.)

## Problem

Robinhood Chain (Arbitrum Orbit, chain 4663, public 2026-07-01) stock tokens carry powers a holder cannot see before holding: a per-address blocklist hidden in Solidity modifiers, a global pause, a sequencer filterer that ran 6,092 times in 6 weeks (~150/day) with no published criteria and no appeals, and impostor tokens in the wild ("Hoodrat • Robinhood Token").

The incumbent answers fail in three different ways — **coverage ≠ correctness ≠ enforcement**:

- **Coverage:** generic scanners now list the chain (GoPlus's supported-chains API returns `Robinhood / 4663`, verified live 2026-09-11). Coverage arrived before this hackathon ends; "no economic reason to cover us" is dead as a claim.
- **Correctness:** whether those checks catch this chain's beacon-proxy + hidden-modifier pattern is unproven — the only measurement (xroot.dev, 2026-08-25) says they miss it. The week-1 spike re-tests against the live GoPlus API; demo copy has a pre-written branch for either result.
- **Enforcement:** nobody refuses a red-flagged token at execution time. Scanners report, a docs page goes stale, and the holder is the only enforcement that exists.

The issuer's own answer is a static webpage (docs.robinhood.com/chain/contracts): "a token with a matching name/ticker but a different contract address is not a Robinhood Stock Token" — for humans only, no machine-readable feed, no power disclosure, stale on every beacon upgrade. Beating *that page*, not a vacuum, is the demo's job.

**Who has it:** retail users who hold or consider holding stock tokens on Robinhood Chain; devs integrating those tokens.
**Today:** trust whatever UI listed the token; run a generic scanner that just arrived and is unproven here; read one hand-made research post (xroot.dev).
**Why this way, not the issuer's:** the operator is a regulated broker — opacity is the design; it will never ship a self-serve risk API.
**Why now:** a verification-and-enforcement vacuum on the 10-week-old chain — analytics exist (Bubblemaps, live on 4663) but nothing issues per-token verdicts or enforces them; the tokenized-stock controversy (AMC fight escalated 2026-09-09) keeps the topic in every feed through judging; deadline 2026-10-04 forces a tight wedge.

**True when it works:** anyone pastes an address and, in under a minute, sees an evidence-linked verdict and a power report; the watchdog widget shows the chain's censorship activity at a glance; and a guarded swap contract refuses a red-flagged token on-chain, at execution time — including one that was verified yesterday and beacon-upgraded since.

## Verdict discipline (anti-false-positive rules)

1. **IMPOSTOR requires positive evidence, and ground truth is the issuer's, not ours:** the canonical list is fetched live from docs.robinhood.com/chain/contracts at scan time. Impostor = a canonical verification exists for the same name/symbol at a different address, or implementation bytecode differs from the canonical's current implementation, or metadata mimics a canonical while unverified. Otherwise UNVERIFIED — never guess.
2. **Reverts only on deterministic red flags:** no live verification record, record revoked, paused, buyer-blocklisted, implementation ≠ record. Heuristic bytecode flags warn in the report, never revert.
3. **Depth boundary, stated in the UI:** full verdicts require a signature match on the known Robinhood stock-token pattern. Every other contract gets UNVERIFIED + generic structural heuristics, labeled advisory on screen. We never claim to "scan any token deeply."
4. Every verdict line links its evidence (tx, slot, bytecode diff, probe result) on screen.

## Scope v1 (decided round 2)

1. **Scan** — address → verdict + risk report (hidden blocklist, global pause, upgradeability, transfer hooks). Engine: thin stateless Rust backend (see Tech). Depth boundary per discipline 3.
2. **Canonical Registry** (on-chain) — token → verification record {status, risk flags, verifiedAt, implementation pointer, registrar}. Writes restricted to a single registrar key held by the project backend; criteria published in the repo; `verify` / `revoke`. Revocation on beacon upgrade is the feature, not a wart. **Revocation trigger:** the backend runs a Workers cron polling each record's implementation pointer against the live beacon slot and revokes on drift — the registry itself is the state, so this adds no database; the guard's implementation≠record probe covers the window between poll and revoke. **Wording ban:** no "attestation" anywhere (Ondo's on-chain "attestations" are EIP-712 trade quotes — judges will conflate). Say: **Canonical Registry**, **verification record**, **registrar**.
3. **Guarded swap** (on-chain) — escrow-style swap; at execution: registry status + staticcall probes (paused, buyer-blocklisted) + implementation-vs-record check. Deterministic reverts only; everything else settles.
4. **Watchdog widget** (v1, promoted from roadmap) — sequencer-filterer activity: cumulative filterer runs in one RPC read, compared against the published 6-week baseline expressed as a daily rate (~150/day, measured 2026-08) — no stored history. On the scan page. This is the retention answer: recurring reason to return, with WalletGuard's sunset (2025-03-31) as the evidence that one-shot consumer checkers die.
5. **Demo asset set** — self-deployed faithful replicas on Robinhood Chain 4663 (public chain, permissionless deploy): a genuine-pattern stock token (beacon proxy, hidden modifier blocklist, global pause) verified as canonical, plus impostor twins. The scanner accepts any live address regardless of who deployed it.
6. **Demo arc (5 min):** the docs-page problem (30s) → live read-only scan of a real chain address we did **not** deploy (30s) → impostor twin scan, side-by-side vs canonical (30s) → power report on the genuine replica: hidden blocklist, global pause, beacon-upgradeable (45s) → guarded swap: impostor reverts on-chain with reason surfaced, genuine settles (45s) → **the beat no incumbent has:** beacon-upgrade the canonical replica → registry auto-revokes → the guard now refuses the formerly verified token at execution time (60s) → close on watchdog widget + registry-as-primitive + roadmap (30s).

**Out of v1:** watchlists, alerts, third-party integrations *built for specific partners* (the registry's public read interface stays in v1 — journey J3), ZeroDev gasless UX (stretch only), multi-venue expansion.

## Why not just EAS? (Q&A answer, written down)

EAS stores attestations; it does not interpret, enforce, or revoke them. This registry is load-bearing inside the swap: the guard reads the record in-tx, binds it to live probe results, and honors revocation tied to beacon upgrades — semantics EAS does not provide. EAS availability on Arbitrum Sepolia / Robinhood Chain is unverified (docs hosts broken at market time); the week-1 spike resolves it. If present, EAS becomes an integration path underneath a later version — v1 still ships the bespoke registry because revocation semantics + probe binding are the product.

## Tech (settled round 2)

- **Contracts:** Rust/Stylus for everything on-chain, on both chains; Solidity fallback only if the week-1 gate fails. One language, one toolchain, same deploy scripts.
- **Week-1 spike — committed pass/fail gate, decided day 7 (cannot slip):**
  - *Stylus gate:* deploy on Robinhood Chain 4663 a contract that (a) staticcalls an arbitrary address, (b) reads remote contract code (extcodecopy-equivalent), (c) reads the EIP-1967 implementation slot. **PASS** = all three work and the full probe suite costs ≤ 200,000 gas per swap. **FAIL** = Stylus not activated, host fns missing, or over budget → all-Solidity on both chains, decided day 7.
  - *GoPlus re-test:* live API against genuine stock tokens + the known impostor on 4663. If GoPlus catches the pattern, the scan half's copy leans on enforcement + revocation + power-disclosure and never claims "first coverage."
  - *EAS availability:* direct probe on both chains, feeds the paragraph above.
  - *Replica smoke:* beacon-proxy replica deploys and upgrades on 4663.
  - *Canonical-list fetch:* from a Worker, fetch docs.robinhood.com/chain/contracts machine-readably — the stock-token table is JS-rendered in a browser, so try the Mintlify-style `.md` endpoint or the underlying JSON. **FAIL** → the pre-committed degraded branch is the copy: all-UNVERIFIED banner as specced, rules 1/3 drop the "fetched live" phrasing, and registry records (registrar writes, by a human reading the docs page) become the only machine ground truth.
- **Backend:** thin stateless Rust on Cloudflare Workers — RPC reads (bytecode, EIP-1967 slot resolution, probe staticcalls), live issuer canonical-list fetch, rule engine. No database.
- **Frontend:** React + viem/wagmi; dark "security terminal" theme, verified-green vs risk-red duality; wallet connects only for the swap.
- **Auth:** scans are anonymous; the registrar key (backend-held, revocable writes) is the only privileged identity; user wallet signs swaps only.
- **Deploy:** Robinhood Chain 4663 itself (public chain, permissionless deploy — no Robinhood testnet is documented in evidence) + Arbitrum Sepolia mirror, same scripts, source-verified. The demo's replica-scan and guarded-swap beats run on 4663, where canonical ground truth and the journeys' network default already live.

## Competition (details in market.md)

Generic scanners: GoPlus (lists 4663; report-time only; correctness on this pattern unproven — spike decides), Blockaid (B2B enterprise, no 4663, no public verdicts), De.Fi and Token Sniffer (capabilities unverified — do not cite specifics in Q&A). Issuer pattern: Robinhood's static docs page is the incumbent to beat; Ondo (off-chain verification-agent reports) and xStocks (proof-of-reserves portal) verify nothing per-token on-chain. Infra precedent: EAS and Uniswap token lists store/format but never interpret or enforce. Enforcement precedent: Safe guards (pre-execution veto, no token-risk logic) — cite as familiar, not novel. Graveyard marker: WalletGuard. Adjacent-but-not: Bubblemaps (live on 4663; forensics, no verdicts, no enforcement).

**PMF story:** the registry is a primitive others integrate — wallets, aggregators, frontends consume the canonical record and the refusal; the watchdog widget is the recurring-audience hook. Not a consumer app people open daily.

## Constraints

- Arbitrum Open House Singapore Online Buildathon; closes **2026-10-04 23:59 SGT**; winners 2026-10-12; 614 participants. Must deploy on an Arbitrum chain (Robinhood Chain qualifies); ≥1 of 3 top prizes reserved for a Robinhood Chain project. Judged on: contract quality, PMF, innovation/creativity, real problem solving. Target: First Place Overall ($40,000 USDC).
- Solo builder + AI agents, ~3 weeks. Contract surface small and provably correct; scope ruthlessly.
- **Pre-submission checklist:** spot-check Token Sniffer and De.Fi web UIs from a real browser (their capabilities were unfetchable at research time); rehearse the EAS and GoPlus answers with spike results in hand.

## Risks

1. **Stylus unavailable on Robinhood Chain** — day-7 gate with committed fallback (all-Solidity, both chains); cannot slip past week 1.
2. **GoPlus already catches the pattern** — spike re-test decides; the copy branch is pre-written (enforcement + revocation + power disclosure carry the scan half).
3. **False accusation of a genuine token** — ground truth is the issuer's live canonical list, positive evidence only, every line links evidence.
4. **No reachable live canonical at demo time** — the read-only scan targets any live address the chain has; replicas carry the verdict/swap/upgrade beats; the docs-page fetch itself is spike-gated (fifth spike bullet), so the demo never bets on an untested fetch.
5. **Fresh official redeployment reads IMPOSTOR until verified** — equals the docs page's own update lag; defensible in Q&A.
6. **Impostor twins live on the production chain** — the replica set deploys on 4663, so lookalike demo tokens exist where real users trade. They carry no liquidity and no holder base, are labeled demo artifacts in the repo and demo narration, and the tool's own IMPOSTOR rule flags them — the decoys double as live evidence.
