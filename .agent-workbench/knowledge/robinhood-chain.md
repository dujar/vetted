---
topic: robinhood-chain
version: chain 4663 mainnet / 46630 testnet (live-probed 2026-09-11)
checked: 2026-09-11
sources:
  - https://docs.robinhood.com/chain/connecting
  - https://docs.robinhood.com/chain/bridging
  - https://docs.robinhood.com/chain/deploy-smart-contracts
  - https://docs.robinhood.com/chain/differences-from-ethereum
  - https://docs.robinhood.com/chain/protocol-contracts
  - live eth_chainId probes 2026-09-11
---

- Chain ID 4663 mainnet — verified live: `eth_chainId` on the public RPC returns 0x1237
  (= 4663). Testnet returns 0xb626 (= 46630). CORRECTION TO state.md Q7: a Robinhood
  TESTNET IS documented (chain 46630, its own RPC + explorer + bridge contracts) — "no
  Robinhood testnet exists in any evidence file" is wrong as of 2026-09-11.
- RPC: https://rpc.mainnet.chain.robinhood.com (public, rate-limited); Alchemy
  recommended: https://robinhood-mainnet.g.alchemy.com/v2/{API_KEY} (+ WSS, plus
  `wss://feed.mainnet.chain.robinhood.com` sequencer feed). Explorers (Blockscout):
  robinhoodchain.blockscout.com mainnet; explorer.testnet.chain.robinhood.com testnet.
- Gas token: ETH. Acquire via the Arbitrum canonical bridge — docs give exactly
  https://portal.arbitrum.io/bridge?destinationChain=robinhood-chain&sourceChain=ethereum
  (ETH + ERC-20 from Ethereum; deposits ~10 min; withdrawals 7-day challenge). NO faucet
  is documented anywhere; no KYC/geo restrictions documented. Whether the portal UI
  actually offers 4663 today is UNVERIFIED — this is already the week-1 verification gate;
  budget mainnet ETH on L1 for bridging.
- STYLUS: never mentioned in any docs page (deploy page says "contracts written in
  Solidity or Vyper deploy without modification"; sitemap has no stylus page). But
  ArbWasm (0x…0071) and ArbWasmCache (0x…0072) precompiles ARE in the protocol-contracts
  table. Whether Stylus is activated on 4663 is UNVERIFIED — the day-7 gate is exactly
  `cargo stylus check --endpoint https://rpc.mainnet.chain.robinhood.com`. Docs silence +
  ArbWasm presence is suggestive, not proof.
- Permissionless deploy: docs state only requirement is a funded wallet; Foundry and
  Hardhat are the documented toolchains (forge create / hardhat verify against Blockscout).
- EVM quirks that hit this codebase: `block.number` = periodic L1 estimate (use
  ArbSys.arbBlockNumber); `prevrandao`/`difficulty` are CONSTANT (no randomness);
  contract limits 96 KB runtime / 192 KB init; fees = L2 execution + L1 calldata.
- Sequencer screening is OFFICIAL, not just xroot: "any transaction associated with a
  sanctioned address will be excluded from inclusion" (differences page) — cite this for
  the watchdog widget. Ordering is first-come-first-served; fees don't reorder.
- Orbit stack (L1): Rollup 0x23A19d23e89166adedbDcB432518AB01e4272D94, SequencerInbox
  0xBd0D173EEb87D57A09521c24388a12789F33ba96, DelayedInbox 0x1A07cc4BD17E0118BdB54D70990D2158AbAD7a2D.
