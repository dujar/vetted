/**
 * The three chains Vetted touches, with RPCs verified live 2026-09-12
 * (`eth_chainId` probes; see packages/shared/wire.md). viem's built-in
 * arbitrumSepolia is used verbatim — its default RPC is the same verified
 * sepolia-rollup endpoint.
 */
import { defineChain, type Chain } from "viem";
import { arbitrumSepolia } from "viem/chains";

/** Robinhood Chain mainnet — verdicts, swaps, the demo. Journeys' default. */
export const robinhoodChain = defineChain({
  id: 4663,
  name: "Robinhood Chain",
  nativeCurrency: { name: "Ether", symbol: "ETH", decimals: 18 },
  rpcUrls: {
    default: { http: ["https://rpc.mainnet.chain.robinhood.com"] },
  },
  blockExplorers: {
    default: { name: "Blockscout", url: "https://robinhoodchain.blockscout.com" },
  },
});

/** Robinhood Chain testnet (chain 46630) — scratch deploys. */
export const robinhoodTestnet = defineChain({
  id: 46630,
  name: "Robinhood Chain Testnet",
  nativeCurrency: { name: "Ether", symbol: "ETH", decimals: 18 },
  rpcUrls: {
    default: { http: ["https://rpc.testnet.chain.robinhood.com"] },
  },
  blockExplorers: {
    default: { name: "Robinhood Testnet Explorer", url: "https://explorer.testnet.chain.robinhood.com" },
  },
});

/** Arbitrum Sepolia — the Stylus mirror (viem ships the verified definition). */
export { arbitrumSepolia };

export const VETTED_CHAINS = [
  robinhoodChain,
  robinhoodTestnet,
] as const satisfies readonly [Chain, Chain, ...Chain[]];

/** All three, in journeys order (4663 first — the default). */
export const ALL_CHAINS = [robinhoodChain, robinhoodTestnet, arbitrumSepolia] as const;

export const DEFAULT_CHAIN = robinhoodChain;
