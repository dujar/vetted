# Step 2 — Week-1 spike gate: Stylus on 4663 (day-7 pass/fail), gas acquisition, pattern calibration, canonical fetch

## Resources
- spec:    ../product/spec.md   (the gate contract: §Tech spec.md:50–55; degraded branch spec.md:55)
- state:   ../product/state.md  (Q7 rationale + GoPlus pre-answer, lines 32–33)
- knows:   ../knowledge/robinhood-chain.md, ../knowledge/stylus-toolchain.md, ../knowledge/arbitrum-sepolia.md, ../knowledge/eas.md, ../knowledge/goplus-api.md, ../knowledge/robinhood-stock-tokens.md
- learned: (none — no built dependencies; this step produces the first findings.md, which steps 3, 4, 6 build against)

## Stack
Same as step 1. Spike code lives in `spike/` and is throwaway proof, not product.

## Goal
By **2026-09-18** (day 7 — spec.md:50 "cannot slip"): prove or kill Rust/Stylus on Robinhood Chain 4663 with the exact spec'd probe (staticcall + remote code read + EIP-1967 slot resolution, full probe suite ≤200,000 gas), verify mainnet-gas acquisition, smoke the replica upgrade mechanics, prove the canonical-list fetch, and calibrate the stock-token fingerprint that the depth boundary (spec.md:29) runs on. Output: `spike/findings.md` with the gate decision on line 1.

## Scope
- Creates: `spike/**` (probe contract, funds log, fetch script, screenshots, findings.md). Touches nothing else except one merge-time append: the calibrated probe selectors written into the `PROBE_SELECTORS` block of `packages/shared/abi.ts` (block reserved by step 1) — the bytes step 3's guard and step 5's guard-preview both need.
- Out of scope: production contracts (step 3), backend rules (step 4), final replica set (step 6) — spike versions are disposable.

## User journey
N/A — verification work.

## Screens
N/A — screenshots for slides only.

## Tasks (order matters — funds first, bridging has latency)
1. Funds, day 1: claim a Sepolia faucet for real — pick ONE from knowledge arbitrum-sepolia.md:17–24 (the 200s landing pages still often require login/mainnet balance; complete an actual claim, don't defer). Bridge ETH to 4663 via the documented portal `https://portal.arbitrum.io/bridge?destinationChain=robinhood-chain&sourceChain=ethereum` (knowledge robinhood-chain.md:22–27; deposits ~10 min; budget L1 gas). Figure out 46630 scratch gas (bridge contracts documented at docs.robinhood.com/chain/bridging); if unsolved in a day, 421614 is the scratch env. Check: funded key on 421614 + 4663, tx hashes logged in findings.md.
2. Probe contract `spike/probe/` (Rust/Stylus, sdk 0.10.9 — host access via `.vm()` on `#[storage]` contracts, knowledge stylus-toolchain.md:21–23): (a) staticcall an arbitrary address — call `paused()` on genuine token P `0x1Cdad396DB64BDa184d5182A97Dd9B3C62100b7D`; (b) remote code read (extcodecopy-equivalent host fn — its existence is itself a gate criterion); (c) EIP-1967 impl slot `0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc` + beacon slot `0xa3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50` read + `beacon.implementation()` staticcall. The gate itself: `cargo stylus check --endpoint https://rpc.mainnet.chain.robinhood.com` — Stylus activation on 4663 is UNVERIFIED (docs silent, ArbWasm precompiles present: knowledge robinhood-chain.md:29–34).
3. Deploy probe to 421614 + 4663 (`cargo stylus deploy --endpoint … --private-key …` — two txs each, knowledge stylus-toolchain.md:27–30); run all three probes live. Check: all three return expected values on both chains.
4. Gas measurement: receipt gasUsed for the full probe suite (record read + paused + blocklist + impl-vs-record). Check: **≤200,000 on both chains** — the spec'd PASS number (spec.md:51); record actuals.
5. Pattern calibration: getcode(P) → resolve impl via slots → getcode(impl); repeat for CRM `0xd95B44124e475743a7589e68F3D74008A5536D44` (knowledge robinhood-stock-tokens.md:24–26). Extract function selectors; define the stock-token fingerprint (selector set + beacon layout) — the signature the scanner's depth boundary matches and the step-6 replicas must reproduce. Check: identical fingerprint on both genuine tokens; pause + per-address-blocklist probe selectors identified. Findings entry with byte-level evidence. At merge: replace the `PROBE_SELECTORS` placeholders in `packages/shared/abi.ts` with the calibrated bytes (the only file step 2 touches outside `spike/`).
6. Replica smoke (Solidity, disposable): minimal UpgradeableBeacon + BeaconProxy + pausable/blocklisted impl deploys AND upgrades on 421614 (and 46630 if funded) — proves the demo's upgrade beat mechanics (spec.md:54).
7. Canonical fetch `spike/fetch/`: workers-rs script calling `GET https://api.robinhood.com/rhj/assets` — machine-readable, no auth, 60 req/s, 194 assets (knowledge robinhood-stock-tokens.md:12–19). This **supersedes** spec spike bullet 5's docs-page `.md`/JSON spelunking (spec.md:55) — the docs page renders from this same registry. Record the response shape; verify P and CRM rows carry contractAddress + chainId 4663. The degraded branch (fetch fail → all-UNVERIFIED banner) stays pre-committed for live-demo failure (spec.md:55, journeys.md:20).
8. EAS availability: answered by knowledge eas.md:10–24 — deployed on Arbitrum Sepolia (EAS `0x2521021fc8BF070473E1e1801D3c7B4aB701E1dE`), NOT on 4663; no on-chain probe needed (optional single staticcall sanity check). Record into findings.md — this completes the "why not EAS" paragraph (spec.md:43–45).
9. GoPlus re-run: `GET https://api.gopluslabs.io/api/v1/token_security/4663?contract_addresses=0x1Cda…` — **underscore endpoint**, the hyphen form 404s (knowledge goplus-api.md:11–17). Screenshot for slides showing the metadata-only response (is_proxy=1, every risk field omitted — knowledge :18–27). Downgraded to a screenshot task per state.md:33; the copy branch (enforcement + revocation + power disclosure, never "first coverage") is the live-verified one.
10. Gate decision → `spike/findings.md`, first line: `GATE: PASS — Stylus everywhere` or `GATE: FAIL — all-Solidity both chains`, then gas actuals, calibrated fingerprint table, /rhj/assets sample, screenshots, funds log. The decision cannot slip past 2026-09-18 (spec.md:50).

Check: findings.md exists with the GATE line, gas numbers vs the 200k budget (or the FAIL decision), fingerprint selector table, a /rhj/assets sample, two screenshots (GoPlus, EAS record).

## Open questions
- Portal-bridge 4663 availability is unverified (knowledge robinhood-chain.md:26–27). If the UI does not offer 4663 by day 3 (Sep 15): escalate to coordinator — alternatives are acquiring 4663 gas from another holder or de-scoping 4663 to testnet-only (needs a spec amendment; the demo beats target 4663, spec.md:59).
- 46630 gas source undocumented — if unsolved in a day, scratch work moves to 421614; no plan change (46630 is convenience, not a requirement).
