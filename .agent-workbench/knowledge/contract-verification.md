---
topic: contract-verification
version: foundry book master (2026-09-11) / Etherscan API v2 / Blockscout v1-compat API
checked: 2026-09-11
sources:
  - https://docs.robinhood.com/chain/deploy-smart-contracts (fetched 2026-09-11)
  - https://raw.githubusercontent.com/foundry-rs/book/master/src/pages/reference/common/verifier-options.mdx and .../config/reference/etherscan.mdx
  - https://info.etherscan.com/etherscan-api-v2-multichain/
  - live probes: api.etherscan.io, api.arbiscan.io, both Robinhood explorers (2026-09-11)
---

- Robinhood mainnet 4663, the OFFICIAL recipe (docs.robinhood.com/chain/deploy-smart-contracts):
  `forge verify-contract <addr> src/File.sol:Name --chain-id 4663 --rpc-url $RH_RPC_URL
  --verifier blockscout --verifier-url https://robinhoodchain.blockscout.com/api/`
  Testnet (46630): same shape, `--verifier-url https://explorer.testnet.chain.robinhood.com/api/`.
  Hardhat gets `apiKey: { robinhood: "empty" }` + `apiURL: .../api`. Sourcify is not mentioned.
- Foundry book confirms the verifier shapes: options are etherscan | sourcify | blockscout, and
  for blockscout "make sure you add '/api?' to the end of the Blockscout homepage explorer URL".
- LIVE-PROBE footgun: the mainnet explorer robinhoodchain.blockscout.com sits behind a Cloudflare
  bot challenge for server-side clients — curl (with or without browser UA) gets "Just a
  moment…" HTML on /api and /api/v2/*; WebFetch got 403/404 on the same paths. The TESTNET
  explorer (explorer.testnet.chain.robinhood.com) answers cleanly: /api?module=block&action=
  eth_block_number returned the Etherscan v1-compat JSON and /api/v2/main-page/blocks returned
  JSON. Whether forge verify passes the mainnet challenge from a dev machine is UNVERIFIED —
  make it a week-1 gate. Fallbacks: Blockscout's browser UI verification (browsers pass CF), or
  `forge ... --verifier sourcify`.
- Arbitrum Sepolia: Etherscan API v2. Base https://api.etherscan.io/v2/api, `chainid` parameter
  REQUIRED (omit it → error pointing to https://api.etherscan.io/v2/chainlist, which lists
  421614 = "Arbitrum Sepolia Testnet", apiurl …/v2/api?chainid=421614, explorer
  sepolia.arbiscan.io). Use `forge verify-contract --chain-id 421614` (etherscan is the default
  verifier) + ETHERSCAN_API_KEY — no --verifier-url needed for known chains. Foundry book: "With
  Etherscan API V2, only Etherscan keys are valid, which can be used to access all similar
  explorers." A key is required (bogus key → "Invalid API Key (#err2)").
- CORRECTION to training memory: per-chain Etherscan-v1 endpoints are DEAD — api.arbiscan.io/api
  now returns "You are using a deprecated V1 endpoint, switch to Etherscan API V2 using
  https://docs.etherscan.io/v2-migration" (live probe); v1 was fully deprecated 2025-08-15
  (info.etherscan.com). The separate ARBISCAN_API_KEY habit is gone — one Etherscan key.
- Etherscan's v2 chainlist (61 chains) contains NO 4663/46630 entries — Robinhood verification
  stays Blockscout-only; there is no Etherscan-v2 path for it.
- Unchanged since the cutoff: forge verify-contract's flags/flow itself and Blockscout's browser
  verification UI flow.
