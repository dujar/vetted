# Market — token-security & tokenized-stock trust layer

## Round 1 — 2026-09-11, target: `.agent-workbench/product/spec.md` (fills spec §Competition)

- **Search tool:** WebFetch only. WebSearch backend hard rate-limited (resets 2026-10-08, after the deadline); web-reader MCP shares the same dead quota (confirmed: "Weekly/Monthly Limit Exhausted … reset at 2026-10-08"). All findings below were fetched 2026-09-11 from the linked URLs unless marked otherwise.
- **Method note:** every entry distinguishes *claimed* (marketing numbers, self-reported) from *verified this session* (fetched live, incl. one live API call).

---

## A. Generic token security scanners (the "GoPlus clone" objection lives here)

### 1. GoPlus Security — gopluslabs.io
- **Claim:** "Decentralized security layer for Web3" — token/NFT/dApp risk APIs, browser extension, transaction simulation, SafeToken protocol, AI-agent security (AgentGuard).
- **Mechanism (verified, docs.gopluslabs.io/reference/response-details):** static/off-chain analysis returning per-token fields: `is_proxy`, `hidden_owner`, `can_take_back_ownership`, `is_honeypot`, `transfer_pausable`, `is_blacklisted`/`is_whitelisted`, `buy_tax`/`sell_tax`, `fake_token`, and premium `b20_token` object with blacklist/whitelist admin address lists. Report-time only — nothing executes on-chain.
- **Chain support — load-bearing, verified live today:** `GET https://api.gopluslabs.io/api/v1/supported_chains` returned `{"name":"Robinhood","id":"4663"}` — **GoPlus already lists Robinhood Chain.** Arbitrum mainnet (42161) only; no Arbitrum Sepolia/Nova; testnets essentially unsupported. Their own docs pages do not mention 4663 anywhere (homepage and response docs read today) — the chain list is ahead of their surface area.
- **Buyer:** API consumers (DexScreener integration and OKX strategic investment/inegration claimed on homepage); no public pricing on page. MiCAR registration claimed.
- **Size:** homepage counters render "0+" (animation) — no usable claimed figures this session.
- **Gap vs product:** supported-chain flag ≠ correct per-token verdicts on a 10-week-old beacon-proxy chain; scout/spec evidence (xroot.dev, measured 2026-08-25) says generic scanners miss the hidden-modifier blocklist pattern — but that needs **re-testing against GoPlus specifically now that 4663 is listed**. Also: no genuine-vs-impostor attestation with revocation, no execution-time refusal.

### 2. Blockaid — blockaid.io
- **Claim:** "The trust layer for onchain finance" — the exact phrase family the spec's working name ("Vetted"/trust layer) collides with.
- **Mechanism (verified, homepage):** transaction security, dApp/token/address scanning, end-user protection, on-chain monitoring with "programmatic response", Cosigner policy enforcement for wallet admins. Pre-signing simulation + policy, B2B-embedded.
- **Chains:** no chain list published; only a Coinbase testimonial naming Ethereum "as well 6+ other EVM chains such as Base, Optimism, and Polygon." No Robinhood Chain, no 4663, no tokenized stocks.
- **Buyer:** wallets/DEXs/exchanges (logo wall: Coinbase, MetaMask, Rainbow, Ledger, Uniswap, Polymarket, Privy…). Enterprise sales; no public pricing.
- **Size (claimed, homepage):** 5.9B txs scanned, $312B assets secured, 527M attacks prevented, $13.1B theft prevented; investors Sequoia, Greylock, Cyberstarts, Ribbit, GV, Variant. All self-reported.
- **Gap vs product:** gated enterprise coverage (a new chain gets covered when a paying wallet asks), no public per-token verdicts, no issuer-legitimacy signal, no on-chain enforcement contract end users can hold.

### 3. ScamSniffer — scamsniffer.io
- **Claim:** "Real-time scam intelligence" — phishing sites, wallet drainers, scammer addresses as blocklists.
- **Mechanism (verified, homepage):** domain/URL + address blocklists; browser extension; edge API; open-source DB updated daily. **No token contract analysis at all.**
- **Buyer:** teams — Standard from **$999/month** (verified), Enterprise custom. Open-source tier free.
- **Size (claimed):** 47M URLs scanned, 705K scams identified, 258K+ blocklist entries; cited by Cointelegraph/CoinDesk/Decrypt/The Block.
- **Gap:** phishing/address intelligence, not contract-capability or legitimacy verdicts; no chain list published; no Robinhood mention.

### 4. De.Fi — de.fi / docs.de.fi
- **Claim:** "Essentially the Robinhood of DeFi" — invest "without the fear of getting rekt" (verified quote, docs).
- **Mechanism (verified, docs.de.fi):** Dashboard, **Shield**, Smart Contract Scanner, in-house audits + audit database, Rekt hack database. Scanner checks described only generically ("reentrancy attacks, unchecked transfers … Solidity best practices to ERC standards"); "10+ supported blockchains", **chains never named** on the doc pages fetched; no Robinhood/4663 mention anywhere fetched.
- **Buyer:** retail (free scanner) → protocol teams (paid audits). No public scanner pricing found.
- **Size:** none verifiable this session (main site is JS-opaque to fetchers).
- **Gap:** heuristic, chains-unnamed, no 4663 evidence; Rekt DB is protocol-hack history, not token vetting; no on-chain enforcement.

### 5. Token Sniffer — tokensniffer.com — **NO SOURCES this session**
Root 403 (bot-block), www NXDOMAIN, Wayback snapshots 403. Existence known; **all capability claims unverified this session** — do not cite specifics against it in the demo/Q&A.

---

## B. Tokenized-stock ecosystem (how issuers communicate legitimacy today)

### 6. Robinhood Chain / Robinhood Stock Tokens — docs.robinhood.com/chain/
- **The incumbent pattern, verified today:** `docs.robinhood.com/chain/contracts` publishes canonical addresses (WETH, USDG static; stock-token table JS-loaded) with the instruction: **"Use the addresses on this page to identify the canonical Robinhood Stock Token for each underlying… a token with a matching name/ticker but a different contract address is not a Robinhood Stock Token."** That page is the entire existing verification UX: a webpage. It says nothing about proxy/upgrade patterns, blocklists, pausability, or the sequencer filterer, offers no machine-readable feed, no attestation lifecycle, no revocation when a beacon upgrade changes behavior.
- **Stock token mechanics (verified, `…/chain/stock-tokens`):** "tokenised debt securities issued by Robinhood Assets (Jersey) Limited"; standard ERC-20, 18 decimals; corporate actions via `uiMultiplier()` per **ERC-8056**; per-asset Chainlink feeds; mint/burn by Authorised Participants only (currently **BBVI**) Mon 02:00 – Sat 02:00 CET; per-asset `tradingCapabilities` exposed via `GET api.robinhood.com/rhj/assets`.
- **Ecosystem table** carries an explicit non-endorsement disclaimer — it is not a registry.
- **Read:** the product's demo must beat *this page*, not a vacuum. The page answers "which address is canonical" statically; it cannot answer "what can this token do to me" and goes stale on upgrade.

### 7. xStocks (Backed Assets (JE) Ltd) — xstocks.com / docs.xstocks.fi
- **Claim:** "1:1 backed tokenized US equities and ETFs, tradeable 24/7, across chains" — 715 tickers, **$40B+ transaction volume (claimed)**; issued by Backed (Jersey), distributed via Kraken entities (Bermuda/MiFID II). Chains include **Arbitrum**, Ethereum, Solana, BSC, Mantle, TON, Ink, X Layer, Tron, Optimism.
- **Verification UX (verified, both sites):** none on-chain. Marketing page publishes no addresses; dev docs point to a **proof-of-reserves portal** (defi.xstocks.fi), "audited smart contracts," Base Prospectus/Final Terms, and an onchain **rebasing** mechanism for corporate actions. Transfers described as permissionless; pausability/blocklist not documented on fetched pages.
- **Gap:** verification = legal paperwork + PoR portal; no per-token on-chain attestation, no address registry on the marketing surface, no risk report of token powers.

### 8. Ondo Stocks (ex-Global Markets; Ondo Global Markets (BVI) Ltd) — ondo.finance / docs.ondo.finance
- **Claim:** onchain US stocks/ETFs ("TSLAon" pattern); **$1B+ TVL, 450+ assets (claimed)**; chains Ethereum, BNB, Solana (Arbitrum not listed on fetched pages).
- **Verification UX (verified, docs.ondo.finance/ondo-stocks/trust-and-transparency.md):** **no on-chain token registry.** Trust = Ankura Trust Company as Verification/Security Agent with daily *off-chain* "attestations" of holdings (since 2025-10-01), first-priority security interest, bankruptcy-remote SPV, Spearbit + Cyfrin audits. Addresses ARE published machine-readably (`docs.ondo.finance/addresses.md` + public API for per-asset addresses across networks) — closer to good practice than Robinhood's JS table.
- **Trap for naming (verified, docs.ondo.finance/api-reference/smart-contracts.md):** Ondo's on-chain "attestations" are **EIP-712 signed trade quotes** (side, price, quantity, expiry, chain-bound, bound to a whitelisted `userId`) — trade authorization, not token legitimacy. A judge may conflate the two meanings of "attestation."
- **Gap:** off-chain trust reports; nothing tells a wallet or contract at execution time that a token is genuine or what its admin functions can do.

---

## C. Registry / attestation precedents (what a judge will name)

### 9. Ethereum Attestation Service (EAS) — attest.sh (301 → attest.org)
- **Claim (verified, easscan.org):** "the global base layer for attestations… Public good… Permissionless." Explorer default view shows **14,642 total attestations, 416 schemas, 842 attestors** (appears to be a single-network view; treat as thin mainstream usage signal, not global).
- **NOT verified this session:** chain deployments. Both docs hosts failed (TLS altname mismatch on docs.eas.attest.sh; JS-shell empty on docs.attest.org; raw deployments file 404). Whether EAS is predeployed on Arbitrum Sepolia or Robinhood Chain is **unverified** — resolve during the week-1 spike, not from memory.
- **Read:** EAS is a component, not a competitor — it stores attestations but does no scanning, no verdict discipline, no revocation policy, no enforcement. It is the "why didn't you just use EAS" question in Q&A. A bespoke registry must justify itself on revocation semantics + execution-time probe integration + per-chain pattern knowledge, or the answer is "EAS underneath."

### 10. Uniswap Token Lists — tokenlists.org / github.com/Uniswap/token-lists
- **Claim/mechanism:** signed off-chain JSON catalogs of checksummed token addresses + metadata; UIs fetch lists and gate display.
- **What it does NOT do:** no on-chain enforcement, no safety verification — listing is not a verdict, lists go stale, authorship ≠ truth. (Page fetched returned title only; mechanism summary is spec-knowledge — label accordingly in Q&A.)
- **Read:** the incumbent *format*. "An attested on-chain list with revocation + risk flags + enforcement" is the delta.

### 11. Circle USDC contract addresses — developers.circle.com/stablecoins/usdc-contract-addresses
- **Verified:** static per-chain address tables (~38 mainnet, ~42 testnet) with explorer links and explicit warnings (incl. "USDC.e is not issued or backed by Circle" on X Layer). Arbitrum mainnet + Arbitrum Sepolia listed; **no Orbit chains**. No verification API on this page.
- **Read:** the stablecoin world's answer is the same webpage pattern as Robinhood's. Confirms the hole: even the best issuer lists are static, non-machine-actionable, and enforce nothing.

---

## D. Guards / execution-time enforcement precedents

### 12. Safe transaction guards — docs.safe.global/advanced/smart-account-guards
- **Verified:** since Safe v1.3.0 a guard "programmatically check[s] all the parameters of the respective transaction before execution" and has "full power to block Safe transaction execution"; post-execution check also exists; docs warn a broken guard causes DoS; reference guards: Zodiac `zodiac-guard-scope`, Yearn's.
- **Read:** the closest documented on-chain pre-execution veto pattern — but scoped to one Safe account's own transactions, with zero token-risk logic. A judge will accept "Safe-guard-style pre-flight, applied to settlement of a swap" as a familiar, auditable pattern; cite it as precedent rather than novelty.

### 13. WalletGuard — walletguard.app — **the graveyard marker**
- **Verified:** "Wallet Guard has been sunset as of March 31, 2025." Was a free open-source extension: phishing detection + transaction simulation (ETH/Polygon/Arbitrum/Optimism). Claimed 100K+ wallets protected, 10M+ txs simulated, $40M+ assets saved. No Consensys acquisition mentioned on the sunset page.
- **Read:** consumer-side transaction simulation is a proven dead-end for indie retention; the B2B layer (Blockaid) consolidated it. Answers the "retention" risk with evidence: one-time checkers die; recurring monitoring (filterer watchdog) or embedded enforcement live.

### 14. Blowfish — blowfish.xyz — **NO SOURCES this session**
Root timed out, docs host TLS-rejected. Status unknown; do not cite it in Q&A either way.

### 15. Bubblemaps — bubblemaps.io — the adjacent tool people use *instead*
- **Claim:** "The Onchain Intelligence Layer" — wallet clustering, fund tracing, token activity maps.
- **Chain support (verified, footer):** 14 chains including **Arbitrum AND Robinhood** — already deployed on the target chain.
- **Mechanism:** distribution visualization + Intel Desk investigations + embeddable iframe/API. **No automated security verdicts, no legitimacy check, no enforcement** (their HAWK case study is analyst-driven forensics).
- **Pricing:** unstated on site (API plans at pro.bubblemaps.io; $BMT token exists).
- **Read:** kills the literal "tooling vacuum" framing — analytics exist on-chain-adjacent; verification + enforcement do not. Expect a judge to name it; the answer is a scan verdict and an on-chain refusal, which bubbles cannot produce.

---

## Cross-cutting read — what nobody is doing

**Nobody verifies token legitimacy on-chain for tokenized stocks, nobody publishes what a stock token's contract can do to a holder, and nobody enforces either at execution time.** The issuer's answer is a staleable docs webpage (Robinhood), a PoR portal (xStocks), and off-chain daily PDFs (Ondo); the scanners' answer is off-chain heuristics that only now got around to listing chain 4663; the infra answer (EAS, token lists) stores or formats attestations but never interprets or enforces them. That intersection is the opening, and it is an opening rather than a graveyard: WalletGuard's death marks the consumer *simulation* app category, not the issuer-verification category, which has no consumer product at all yet.

**The two facts that most change the spec (both verified today):**
1. GoPlus's live supported-chains API includes `Robinhood / 4663` — the spec's "generic scanners have no economic reason to build per-chain logic for a 10-week-old chain" is now **half-false as written**; coverage exists, correctness on the beacon-proxy pattern is the open question.
2. Robinhood's own docs page already answers "is this address canonical" for humans — the wedge is what the page can't do: machine-readable, on-chain, revocable on upgrade, power-disclosing, execution-enforced.

## Sharpening questions (Q-numbers continue the spec's queue)

- **Q9 (GoPlus reality check):** Have you run GoPlus's Token Security API against the live/genuine Robinhood stock tokens and the known impostor *this week*, on chain 4663? If it already flags `is_blacklisted`/`transfer_pausable`, the scanner half's "first coverage" claim dies and the differentiator must be stated as: genuine-vs-impostor attestation with revocation lifecycle + hidden-modifier detection depth + on-chain refusal. Should spec §Problem and §Risks-5 be rewritten around "coverage ≠ correctness ≠ enforcement"?
- **Q10 (beat the docs page, explicitly):** Is the registry designed to be a *machine-readable, revocable, upgrade-aware* replacement for `docs.robinhood.com/chain/contracts` — with the demo explicitly showing a beacon upgrade staling the webpage while the registry revokes? And is the IMPOSTOR verdict's ground truth wired to Robinhood's canonical list (fetched live), so the product never contradicts the issuer on a genuine token?
- **Q11 (naming collision):** Blockaid brands itself "the trust layer for onchain finance" — should the one-liner avoid "trust layer" and lead with "issuer-independent token verification + on-chain refusal"?
- **Q12 (EAS or bespoke):** EAS chain support on Arbitrum Sepolia/Robinhood was unverifiable this session (docs hosts broken). Can you pre-decide and rehearse the Q&A answer for "why not just EAS?" — either "EAS underneath" or a justified bespoke registry with revocation semantics? The week-1 spike should resolve EAS availability, not just Stylus.
- **Q13 (attestation word-collision):** Ondo already uses on-chain EIP-712 "attestations" (trade quotes) and off-chain Verification-Agent attestations; xStocks ships PoR. Should the registry fields be positioned as "canonical verification / token clearance" to avoid judges conflating them with backing attestations?
- **Q14 (retention evidence):** WalletGuard (sunset 2025-03-31) and possibly Blowfish mark the dead consumer-scanner path. Is the filterer-watchdog (scout candidate 6) promoted from roadmap to the retention answer's first slide, since it is the only sourced recurring-audience hook?
- **Q15 (comparative-live risk):** Token Sniffer and De.Fi's exact current capabilities could not be fetched (bot-blocked; De.Fi chains unnamed). Do all five demo risk-report lines (hidden blocklist probe, global pause, beacon-impl diff, impostor side-by-side, execution-time refusal) demonstrably not exist in their UIs at judging time — i.e., has someone spot-checked their web UIs from a browser?

## Source failures (returned as NO SOURCES, not guessed)

- Token Sniffer capabilities: 403 on root and www, 403 on Wayback — unverified.
- Blowfish operational status: timeouts/TLS — unverified.
- EAS chain deployments (Arbitrum Sepolia/Orbit/Robinhood) and pricing: docs hosts broken — unverified.
- De.Fi supported-chain list and pricing: unnamed in docs, main site JS-opaque — unverified.
