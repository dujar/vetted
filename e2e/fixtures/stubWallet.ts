/**
 * Stub wallet (plan task 1) — an EIP-1193 provider shim injected before the
 * app boots, so the J2 browser journeys run against a deterministic wallet
 * with zero extension flakiness. The shim announces itself BOTH ways wagmi's
 * `injected()` connector discovers providers (EIP-6963 announce + the legacy
 * `window.ethereum` fallback), answers the connect/switch surface, and fails
 * `eth_sendTransaction` on demand to force the wallet-rejection / gas-failure
 * unhappy paths that mock mode cannot reach (the mock executor never touches
 * the wallet).
 *
 * Gated scratch sends: with `rpcUrl` set, `eth_sendTransaction` is forwarded
 * through a Playwright binding to this Node process, which signs with the
 * funded e2e buyer key (fixtures/constants.ts BUYER_KEY) via viem — real
 * signatures against the real scratch deployment, no in-page signing needed.
 */
import type { Page } from "@playwright/test";
import { createPublicClient, createWalletClient, defineChain, http, parseEther } from "viem";
import { privateKeyToAccount } from "viem/accounts";

import { BUYER_KEY, SCRATCH_RPCS } from "./constants";

export interface StubWalletOptions {
  /** Chain the wallet opens on — set another chain to force the switch prompt. */
  chainId: number;
  accounts?: string[];
  /** eth_sendTransaction behavior. Default "send" (forwards to rpcUrl). */
  sendBehavior?: "send" | "reject" | "error";
  /** With sendBehavior "send" and a fixed hash (offline stubs). */
  sendHash?: string;
  /** Node-side viem send target (scratch RPC). Required for sendBehavior "send" without sendHash. */
  rpcUrl?: string;
}

const BINDING = "__vettedE2E_sendTx";

/** Install the shim; call before the first navigation to the app. */
export async function installStubWallet(page: Page, opts: StubWalletOptions): Promise<void> {
  const accounts = opts.accounts ?? ["0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"];

  if (opts.sendBehavior !== "reject" && opts.sendBehavior !== "error") {
    // Forward real sends to this process — viem signs with the funded e2e key.
    await page.exposeBinding(BINDING, async (_source, tx: { to?: string; data?: string; value?: string; gas?: string; chainId?: number }) => {
      if (opts.sendHash) return opts.sendHash;
      if (!opts.rpcUrl) throw new Error("e2e stub: sendBehavior 'send' needs rpcUrl or sendHash");
      const chainId = Number(tx.chainId ?? opts.chainId);
      const chain = defineChain({
        id: chainId,
        name: `e2e-${chainId}`,
        nativeCurrency: { name: "ETH", symbol: "ETH", decimals: 18 },
        rpcUrls: { default: { http: [opts.rpcUrl] } },
      });
      const wallet = createWalletClient({
        account: privateKeyToAccount(BUYER_KEY as `0x${string}`),
        chain,
        transport: http(opts.rpcUrl),
      });
      return wallet.sendTransaction({
        to: tx.to as `0x${string}`,
        data: (tx.data ?? undefined) as `0x${string}` | undefined,
        value: tx.value != null ? BigInt(tx.value) : undefined,
        gas: tx.gas != null ? BigInt(tx.gas) : undefined,
      });
    });
  }

  await page.addInitScript(
    // Serialized into the page — plain ESM only, no imports.
    (config: { chainId: number; accounts: string[]; sendBehavior: string; sendHash?: string; binding: string }) => {
      const hex = (n: number): string => `0x${n.toString(16)}`;
      let chainId: number = config.chainId;
      // eth_accounts stays empty until eth_requestAccounts — otherwise wagmi's
      // reconnect treats the stub as already-authorized and the connect
      // button never renders (auto-connect skips the J2 connect journey).
      let connected = false;
      const listeners: Record<string, ((...args: unknown[]) => void)[]> = {};
      const emit = (ev: string, ...args: unknown[]) => (listeners[ev] ?? []).forEach((cb) => cb(...args));
      const reject = (code: number, message: string): never => {
        const e: Error & { code?: number } = new Error(message);
        e.code = code;
        throw e;
      };

      const provider = {
        isVettedE2EStub: true,
        request: async ({ method, params }: { method: string; params?: unknown[] }): Promise<unknown> => {
          switch (method) {
            case "eth_requestAccounts": {
              if (!connected) {
                connected = true;
                emit("accountsChanged", config.accounts);
              }
              return config.accounts;
            }
            case "eth_accounts":
              return connected ? config.accounts : [];
            case "eth_chainId":
              return hex(chainId);
            case "net_version":
              return String(chainId);
            case "wallet_switchEthereumChain": {
              const target = parseInt((params?.[0] as { chainId: string })?.chainId ?? "0x0", 16);
              if (target !== chainId) {
                chainId = target;
                emit("chainChanged", hex(chainId));
              }
              return null;
            }
            case "wallet_addEthereumChain":
              return null;
            case "wallet_getCapabilities":
              return {};
            case "eth_sendTransaction": {
              if (config.sendBehavior === "reject") {
                // EIP-1193 standard rejection — wagmi maps code 4001 to UserRejectedRequestError.
                throw reject(4001, "e2e stub: user rejected the transaction");
              }
              if (config.sendBehavior === "error") {
                throw reject(-32603, "e2e stub: insufficient funds for gas * 1000000 + value");
              }
              const tx = { ...(params?.[0] as Record<string, unknown>), chainId };
              return (window as unknown as Record<string, (t: unknown) => Promise<string>>)[config.binding](tx);
            }
            case "eth_estimateGas":
              return "0x186a0";
            case "eth_gasPrice":
              return "0x1";
            case "eth_blockNumber":
              return "0x1";
            case "eth_getBalance":
              return "0xde0b6b3a7640000";
            case "eth_getCode":
              return "0x";
            case "eth_getTransactionReceipt":
              return config.sendHash
                ? {
                    transactionHash: config.sendHash,
                    transactionIndex: "0x0",
                    blockHash: config.sendHash,
                    blockNumber: "0x1",
                    from: config.accounts[0],
                    to: config.accounts[0],
                    cumulativeGasUsed: "0x1",
                    gasUsed: "0x1",
                    contractAddress: null,
                    logs: [],
                    logsBloom: `0x${"0".repeat(512)}`,
                    status: "0x1",
                    effectiveGasPrice: "0x1",
                    type: "0x2",
                  }
                : null;
            case "personal_sign":
            case "eth_signTypedData_v4":
              throw reject(4001, "e2e stub: signature prompts are not part of the journeys");
            default:
              throw reject(4200, `e2e stub wallet: method not supported: ${method}`);
          }
        },
        on: (ev: string, cb: (...args: unknown[]) => void) => {
          (listeners[ev] ??= []).push(cb);
        },
        removeListener: (ev: string, cb: (...args: unknown[]) => void) => {
          listeners[ev] = (listeners[ev] ?? []).filter((f) => f !== cb);
        },
      };

      (window as unknown as { ethereum: unknown }).ethereum = provider;
      const info = {
        uuid: "350670db-19ef-4e2e-a0e5-64d2a0b23c33",
        name: "Vetted e2e Stub",
        icon: "data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg'/>",
        rdns: "dev.vetted.e2e",
      };
      const announce = () =>
        window.dispatchEvent(
          new CustomEvent("eip6963:announceProvider", { detail: Object.freeze({ info, provider }) }),
        );
      window.addEventListener("eip6963:requestProvider", announce);
      announce();
      // Late subscribers: wagmi's EIP-6963 store subscribes at config creation.
      setTimeout(announce, 0);
      setTimeout(announce, 50);
    },
    { chainId: opts.chainId, accounts, sendBehavior: opts.sendBehavior ?? "send", sendHash: opts.sendHash, binding: BINDING },
  );
}

/**
 * Node-side pre-flight for the gated scratch specs — honest gating per the
 * funded-runs reality (plan Revised 2026-09-20 note 3): read-only checks, no
 * hand-forged state. Returns null when the buyer is not funded yet.
 */
export async function scratchBuyerBalance(rpcUrl: string, chainId: number, address: string): Promise<bigint | null> {
  try {
    const chain = defineChain({
      id: chainId,
      name: `e2e-${chainId}`,
      nativeCurrency: { name: "ETH", symbol: "ETH", decimals: 18 },
      rpcUrls: { default: { http: [rpcUrl] } },
    });
    const client = createPublicClient({ chain, transport: http(rpcUrl) });
    return await client.getBalance({ address: address as `0x${string}` });
  } catch {
    return null;
  }
}

export { parseEther };
