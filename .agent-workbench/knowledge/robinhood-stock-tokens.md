---
topic: robinhood-stock-tokens
version: /rhj/assets v1 + docs.robinhood.com/chain, checked 2026-09-11
checked: 2026-09-11
sources:
  - https://docs.robinhood.com/chain/stock-token-apis
  - https://docs.robinhood.com/chain/contracts
  - https://docs.robinhood.com/chain/stock-tokens
  - live GET https://api.robinhood.com/rhj/assets 2026-09-11
---

- CANONICAL GROUND TRUTH IS MACHINE-READABLE — no HTML scraping needed:
  `GET https://api.robinhood.com/rhj/assets` — no auth, 60 req/s global limit, cached.
  194 assets live on 2026-09-11. This supersedes spike bullet 5's fallback planning: the
  JS-rendered contracts page ("Loading tokens…") renders FROM this same on-chain asset
  registry, and there is also /rhj/prices/{symbol} (raw unadjusted bid/ask, 15s cache) and
  /rhj/corporate-actions.
- Response shape (verified): id = on-chain `uid()`, "0x" + 66-char lowercase hex, stable
  across chains; tokenSymbol; tokenName; deployments[{contractAddress, chainId: 4663,
  networkName: "Robinhood Chain"}]; currentMultiplier; pendingMultiplier;
  status: "ASSET_STATUS_ACTIVE"; tradingCapabilities {market/extended/overnight ×
  whole/fractional}; tokenDecimals: 18; isin.
- Naming pattern: `"{Company} • Robinhood Token"` — e.g. CRM "Salesforce • Robinhood
  Token" 0xd95B44124e475743a7589e68F3D74008A5536D44; P "Everpure • Robinhood Token"
  0x1Cdad396DB64BDa184d5182A97Dd9B3C62100b7D; SPCX (SpaceX), DELL, AVGO, CRWD, SMCI…
  Impostor check: same name/ticker at a different address, per docs: "a token with a
  matching name/ticker but a different contract address is not a Robinhood Stock Token".
- Docs' static contract table (non-stock): WETH 0x0Bd7D308f8E1639FAb988df18A8011f41EAcAD73,
  USDG 0x5fc5360D0400a0Fd4f2af552ADD042D716F1d168.
- OFFICIAL architecture docs say only: standard ERC-20, 18 decimals; corporate actions via
  `uiMultiplier()` per ERC-8056 (Scaled UI Amount Extension) — balances static, multiplier
  moves; mint/burn restricted to Authorised Participants (currently BBVI); Reg S
  restrictions (no US persons). Chainlink feeds are multiplier-adjusted; /rhj/prices not.
- BEACON-PROXY + HIDDEN-MODIFIER BLOCKLIST + GLOBAL PAUSE + SEQUENCER FILTERER: present in
  NO official docs page (stock-tokens, protocol-contracts both checked 2026-09-11;
  protocol-contracts lists only Orbit infra — Rollup, SequencerInbox, gateways,
  precompiles — zero stock-token infrastructure). That pattern's source remains xroot.dev
  (third-party, 2026-08-25) → UNVERIFIED officially; the replica set + on-chain
  EIP-1967/bytecode reads are the verification path, which is the product itself.
- Corroboration: GoPlus returns is_proxy=1 for the genuine tokens (goplus-api.md) —
  consistent with upgradeable proxies, third-party sourced.
