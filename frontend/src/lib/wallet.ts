/**
 * Wallet adapter (plan task 3, J2) — wagmi 3 built-in connectors behind a
 * plain object so the swap screen's state machine is testable without a
 * wallet (tests inject a fake adapter; the default hook below is the real
 * wagmi wiring). The zero-connector property is deliberate: without
 * VITE_WALLETCONNECT_PROJECT_ID the config has NO connectors and the UI must
 * prompt for the operator's projectId instead of crashing (step-1 findings).
 */
import { useAccount, useConnect, usePublicClient, useSwitchChain, useWriteContract } from "wagmi";
import { GUARD_ABI } from "vetted-shared";

import { robinhoodChain } from "./chains";
import {
  getGuardProbeSource,
  GuardExecutionRequest,
  GuardExecutionResult,
  GuardExecutor,
  createMockGuardExecutor,
} from "./guard";
import { isLiveApi } from "./api";

export interface WalletAdapter {
  status: "disconnected" | "connecting" | "connected";
  address?: string;
  chainId?: number;
  /** Empty = no connector configured (missing VITE_WALLETCONNECT_PROJECT_ID — operator action). */
  connectorNames: string[];
  connect(): Promise<void>;
  switchToVetted(): Promise<void>;
  execute(req: GuardExecutionRequest): Promise<GuardExecutionResult>;
}

/** Live execution: simulate first (a red flag reverts in simulation — funds never move), then broadcast. */
async function liveExecute(
  writeContractAsync: ReturnType<typeof useWriteContract>["writeContractAsync"],
  publicClient: NonNullable<ReturnType<typeof usePublicClient>>,
  guardAddress: `0x${string}`,
  req: GuardExecutionRequest,
): Promise<GuardExecutionResult> {
  const amount = BigInt(req.amountIn.replace(/,/g, "") || "0");
  const args = {
    address: guardAddress,
    abi: GUARD_ABI,
    account: req.buyer as `0x${string}`,
  } as const;

  await publicClient.simulateContract({
    ...args,
    functionName: "commit",
    args: [req.tokenIn as `0x${string}`, req.tokenOut as `0x${string}`, amount, 0n],
  });
  await writeContractAsync({
    ...args,
    functionName: "commit",
    args: [req.tokenIn as `0x${string}`, req.tokenOut as `0x${string}`, amount, 0n],
  });

  await publicClient.simulateContract({ ...args, functionName: "execute", args: [] });
  const hash = await writeContractAsync({ ...args, functionName: "execute", args: [] });
  const receipt = await publicClient.waitForTransactionReceipt({ hash });
  if (receipt.status !== "success") {
    // Raced past simulation — rare, but the guard refused in-tx; funds never moved.
    throw Object.assign(new Error("guard reverted at execution"), { name: "TransactionExecutionError" });
  }
  return {
    kind: "settled",
    txHash: receipt.transactionHash,
    gasUsed: receipt.gasUsed.toString(),
    amountOut: "",
  };
}

/** Default adapter — real wagmi. Executor/probe switch on VITE_API_MODE exactly like the scan client. */
export function useWalletAdapter(): WalletAdapter {
  const { address, chainId, status: accountStatus } = useAccount();
  const { connectors, connectAsync, status: connectStatus } = useConnect();
  const { switchChainAsync } = useSwitchChain();
  const { writeContractAsync } = useWriteContract();
  const publicClient = usePublicClient();

  const execute: GuardExecutor = async (req) => {
    if (!isLiveApi()) return createMockGuardExecutor(getGuardProbeSource())(req);
    const guard = import.meta.env.VITE_GUARD_ADDRESS as `0x${string}` | undefined;
    if (!guard || !publicClient) {
      throw new Error("VITE_API_MODE=live requires VITE_GUARD_ADDRESS (deployments/4663.json, step 3/7)");
    }
    return liveExecute(writeContractAsync, publicClient, guard, req);
  };

  return {
    status: address
      ? "connected"
      : connectStatus === "pending" || accountStatus === "connecting"
        ? "connecting"
        : "disconnected",
    address: address ?? undefined,
    chainId: chainId ?? undefined,
    connectorNames: connectors.map((c) => c.name),
    connect: async () => {
      const first = connectors[0];
      if (!first) throw new Error("no wallet connector configured");
      await connectAsync({ connector: first });
    },
    switchToVetted: async () => {
      await switchChainAsync({ chainId: robinhoodChain.id });
    },
    execute,
  };
}
