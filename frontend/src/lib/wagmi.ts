/**
 * wagmi 3 config — wallet decision made in step 1 (knowledge frontend-stack.md):
 * built-in wagmi/connectors (WalletConnect); AppKit is wagmi-2.x-only and is
 * excluded. The projectId is wired from env only (operator action); with no
 * projectId the config is created WITHOUT the walletConnect connector instead
 * of throwing at import time, so the app boots read-only and every consumer
 * (including tests) can import safely (verify.md loose end 9).
 */
import { createConfig, http, type Config } from "wagmi";
import { injected, walletConnect } from "wagmi/connectors";
import { robinhoodChain, robinhoodTestnet } from "./chains";

const PROJECT_ID = import.meta.env.VITE_WALLETCONNECT_PROJECT_ID ?? "";
/**
 * e2e-only seam (step 8, verify loose end 1): adds wagmi's `injected()`
 * connector so the Playwright stub wallet — an EIP-1193 provider shim that
 * announces itself via EIP-6963 + window.ethereum — can connect for the J2
 * browser journeys. No product build sets VITE_E2E_STUB_WALLET, so the
 * documented zero-connector state (missing projectId) is untouched.
 */
const E2E_STUB_WALLET = import.meta.env.VITE_E2E_STUB_WALLET === "1";

let cached: Config | null = null;

/** Memoized wagmi config — lazily, at first use, never at module import. */
export function getWagmiConfig(): Config {
  if (cached) return cached;

  cached = createConfig({
    chains: [robinhoodChain, robinhoodTestnet],
    connectors: [
      ...(PROJECT_ID.length > 0 ? [walletConnect({ projectId: PROJECT_ID })] : []),
      ...(E2E_STUB_WALLET ? [injected()] : []),
    ],
    transports: {
      [robinhoodChain.id]: http(robinhoodChain.rpcUrls.default.http[0]),
      [robinhoodTestnet.id]: http(robinhoodTestnet.rpcUrls.default.http[0]),
    },
    ssr: false,
  });
  return cached;
}
