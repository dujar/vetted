---
topic: arbitrum-sepolia
version: chain 421614 (0x66eee, live-probed 2026-09-11)
checked: 2026-09-11
sources:
  - live eth_chainId probe https://sepolia-rollup.arbitrum.io/rpc 2026-09-11
  - https://docs.arbitrum.io/stylus/quickstart.md
  - faucet landing-page checks (curl) 2026-09-11
---

- Chain ID 421614 — verified live: `eth_chainId` returns 0x66eee on
  https://sepolia-rollup.arbitrum.io/rpc (Offchain Labs' public endpoint, rate-limited).
- Stylus on Sepolia is explicit and current: "Stylus is available on Arbitrum Sepolia"
  (docs.arbitrum.io/stylus/quickstart.md, fetched 2026-09-11). The Sepolia mirror needs no
  fallback plan — only the 4663 side is gated.
- Faucet reality-check (HTTP status of landing page via curl, 2026-09-11 — a 200 means the
  page loads, NOT that it dispenses without login/balance requirements):
  - faucet.arbitrum.io → connection failed entirely (000) — treat as DEAD/unreachable.
  - bridge.arbitrum.io → 403 to curl (bot-blocked; may still work in a real browser).
  - HTTP 200: www.alchemy.com/faucets/arbitrum-sepolia, faucets.chain.link/arbitrum-sepolia,
    arbitrum.faucet.dev, faucet.quicknode.com/arbitrum/sepolia, l2faucet.com,
    www.infura.io/faucet/sepolia.
  - Alchemy/QuickNode/Infura/Chainlink faucets typically require a mainnet balance
    threshold or an account login. Pick ONE in week 1 and do a real claim; don't defer.
- Sepolia gas is cheap; budget for Stylus deploy + activation txs (activation is a second
  transaction — see stylus-toolchain.md).
- Unchanged since cutoff: Sepolia is the canonical Stylus testnet target in all current
  Arbitrum docs and examples.
