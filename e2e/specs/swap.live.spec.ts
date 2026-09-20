/**
 * J2 live-build specs. Two layers:
 *
 * 1. ALWAYS-RUN (route-forced): the stub wallet forces wallet rejection
 *    (EIP-1193 4001) and gas failure (-32603) on eth_sendTransaction, and the
 *    public RPC is route-stubbed so the guard preview passes — the two
 *    nothing-sent unhappy paths mock mode cannot reach (its executor never
 *    touches the wallet). No deployments, no real funds.
 *
 * 2. GATED scratch journeys: settle + the on-chain GUARD_* reverts against the
 *    deployed guard — data-driven from `.scratch-state.json`, written only by
 *    scripts/seed-scratch.sh against a funded scratch deployment. Absent file
 *    → honest skip (plan Revised 2026-09-20 note 3). The stub wallet signs
 *    with the funded e2e buyer key through viem (Node-side), so these are real
 *    signatures against the real guard.
 */
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { expect, test, type Page } from "@playwright/test";

import { GUARD_REVERT_REASONS, SCRATCH_RPCS } from "../fixtures/constants";
import { encodeRegistryRecord, stubPublicRpc } from "../fixtures/rpc";
import { SERVER_CHAIN_ID } from "../fixtures/scratch";
import { installStubWallet } from "../fixtures/stubWallet";

/** A passing preview: a VERIFIED record; probes answer clear (rpc.ts stub). */
const PASSING_RECORD = encodeRegistryRecord({
  status: 0,
  verifiedAt: 1_788_048_000,
  impl: `0x${"77".repeat(20)}`,
  registrar: "0x5e1500000000000000000000000000000000aa11",
});

const SWAP_TARGET = `0x${"77".repeat(20)}`; // any address — the stub answers getRecord for every call

async function connectForSwap(page: Page, sendBehavior: "reject" | "error"): Promise<void> {
  await stubPublicRpc(page, { chainId: SERVER_CHAIN_ID, records: { [SWAP_TARGET]: PASSING_RECORD } });
  await installStubWallet(page, { chainId: SERVER_CHAIN_ID, sendBehavior });
  await page.goto("#/swap");
  await page.getByRole("button", { name: "Connect wallet" }).click();
  await page.getByText("guard checks — these run again on-chain at execution").waitFor();
  await page.getByLabel("receive token address").fill(SWAP_TARGET);
  // Preview all-green → the guard will not refuse pre-flight; execute reaches the wallet.
  await expect(page.getByRole("alert")).toHaveCount(0);
}

test.describe("J2 nothing-sent unhappy paths (route-forced)", () => {
  test("wallet rejection: the signature is declined → nothing was broadcast", async ({ page }) => {
    await connectForSwap(page, "reject");
    await page.getByRole("button", { name: "Swap guarded" }).click();

    await expect(page.getByText("Nothing was sent")).toBeVisible();
    await expect(page.getByText("Rejected in wallet")).toBeVisible();
    await expect(page.getByText(/the signature was declined, so no transaction was broadcast/)).toBeVisible();
    await expect(page.getByText(/never left your wallet/)).toBeVisible();
    await expect(page.getByText("Settled", { exact: true })).toHaveCount(0);
  });

  test("gas failure: the transaction never lands", async ({ page }) => {
    await connectForSwap(page, "error");
    await page.getByRole("button", { name: "Swap guarded" }).click();

    await expect(page.getByText("Nothing was sent")).toBeVisible();
    await expect(page.getByText("Gas too low")).toBeVisible();
    await expect(page.getByText(/never left your wallet/)).toBeVisible();
    await expect(page.getByText("Settled", { exact: true })).toHaveCount(0);
  });
});

// --- activation-gated: the deployed scratch guard ---------------------------

interface ScratchEntry {
  kind: "settle" | "revert";
  reason?: string;
  tokenIn?: string;
  tokenOut: string;
}
interface ScratchState {
  chainId: number;
  registry: string;
  guard: string;
  buyer: string;
  entries: ScratchEntry[];
}

const stateFile = path.resolve(import.meta.dirname ?? ".", "..", ".scratch-state.json");
const readState = (): ScratchState | null => {
  if (!existsSync(stateFile)) return null;
  try {
    return JSON.parse(readFileSync(stateFile, "utf8")) as ScratchState;
  } catch {
    return null;
  }
};
const STATE = readState();

test.describe("J2 against the deployed scratch guard (activation-gated)", () => {
  test.skip(!STATE, "no .scratch-state.json — run e2e/scripts/seed-scratch.sh after the funded deploys (e2e/README.md)");

  const rpcUrl = STATE ? SCRATCH_RPCS[STATE.chainId] : "";

  /**
   * ONE serial journey: the seeded red-flag states persist on-chain (pause,
   * beacon upgrade, blocklist), so the order is settle → the five reverts in
   * guard order, exactly as the seed script laid them down. Real reads, real
   * sends — the stub wallet forwards eth_sendTransaction to Node where viem
   * signs with the funded e2e buyer key; no route stubbing here.
   */
  test("settle, then every GUARD_* revert byte-exact against the deployed guard", async ({ page }) => {
    expect(STATE!.guard).toMatch(/^0x[0-9a-f]{40}$/);

    const { createPublicClient, defineChain, http } = await import("viem");
    const chain = defineChain({
      id: STATE!.chainId,
      name: "scratch",
      nativeCurrency: { name: "ETH", symbol: "ETH", decimals: 18 },
      rpcUrls: { default: { http: [rpcUrl] } },
    });
    const balance = await createPublicClient({ chain, transport: http(rpcUrl) }).getBalance({
      address: STATE!.buyer as `0x${string}`,
    });
    test.skip(balance === 0n, `buyer ${STATE!.buyer} unfunded on ${STATE!.chainId} — seed script step 1`);

    await installStubWallet(page, { chainId: STATE!.chainId, rpcUrl });
    await page.goto("#/swap");
    await page.getByRole("button", { name: "Connect wallet" }).click();
    await page.getByText("guard checks — these run again on-chain at execution").waitFor();

    const swap = async (entry: ScratchEntry) => {
      if (entry.tokenIn) await page.getByLabel("pay token address").fill(entry.tokenIn);
      await page.getByLabel("receive token address").fill(entry.tokenOut);
      await page.getByRole("button", { name: "Swap guarded" }).click();
    };

    for (const entry of STATE!.entries) {
      await test.step(entry.kind === "settle" ? `settle ${entry.tokenOut}` : `refused: ${entry.reason}`, async () => {
        if (entry.reason !== undefined) {
          expect(GUARD_REVERT_REASONS).toContain(entry.reason); // only the five pinned reasons, never a paraphrase
        }
        await swap(entry);
        if (entry.kind === "settle") {
          await expect(page.getByText("Settled", { exact: true })).toBeVisible({ timeout: 30_000 });
          await expect(page.getByText("All guard checks passed at execution")).toBeVisible();
        } else {
          const banner = page.locator(".banner.risk");
          await expect(banner).toContainText(`${entry.reason}:`); // verbatim on-chain revert string
          await expect(page.getByText(/never left your wallet/)).toBeVisible();
          await expect(page.getByText("Settled", { exact: true })).toHaveCount(0);
        }
        // The route change remounts SwapPage — fresh form for the next entry,
        // wallet still connected (same SPA session).
        await page.goto("#/swap");
        await page.getByText("guard checks — these run again on-chain at execution").waitFor();
      });
    }
  });
});
