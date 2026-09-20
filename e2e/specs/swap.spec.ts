/**
 * J2 — swap a stock token without getting rugged (journeys.md:24-34), mock
 * project, driven through the REAL bundle + the stub wallet (EIP-1193 shim
 * behind wagmi's injected() connector — the step-8 e2e seam). The five
 * deterministic red flags are asserted byte-exact in the UI against the
 * shared GUARD_REVERT_REASONS constants (never retranscribed); the on-chain
 * versions against the deployed guard are the gated live specs.
 */
import { expect, test, type Page } from "@playwright/test";

import { GUARD_REVERT_REASONS, GUARD_SENTINELS, MOCK_USDG_ADDR, MOCK_VERIFIED_ADDR } from "../fixtures/constants";
import { installStubWallet } from "../fixtures/stubWallet";
import type { GuardRevertReason } from "vetted-shared";

const VETTED_CHAIN_ID = 4663;
const OTHER_CHAIN_ID = 46630;

/** Connect the stub wallet and land on the swap form. */
async function connectWallet(page: Page, chainId: number): Promise<void> {
  await installStubWallet(page, { chainId });
  await page.goto("#/swap");
  await page.getByRole("button", { name: "Connect wallet" }).click();
}

test.describe("J2 connect + wrong network", () => {
  test("stub wallet connects (EIP-6963) and the pre-flight form appears", async ({ page }) => {
    await connectWallet(page, VETTED_CHAIN_ID);
    await expect(page.getByText("guard checks — these run again on-chain at execution")).toBeVisible();
    await expect(page.getByLabel("pay token address")).toHaveValue(MOCK_USDG_ADDR);
  });

  test("wallet on another chain → switch prompt BEFORE any quoting; switch proceeds", async ({ page }) => {
    await connectWallet(page, OTHER_CHAIN_ID);
    await expect(page.getByText(/Robinhood Chain Testnet/)).toBeVisible();
    await expect(page.getByText(/switch to Robinhood Chain · 4663 to quote/)).toBeVisible();
    // Before quoting: no guard checks anywhere (journeys.md:34).
    await expect(page.getByText("guard checks — these run again on-chain at execution")).toHaveCount(0);

    await page.getByRole("button", { name: "Switch to Robinhood Chain · 4663" }).click();
    await expect(page.getByText("guard checks — these run again on-chain at execution")).toBeVisible();
  });
});

test.describe("J2 settle", () => {
  test("genuine token + live record → settles; guard receipt shown", async ({ page }) => {
    await connectWallet(page, VETTED_CHAIN_ID);
    await page.getByLabel("receive token address").fill(MOCK_VERIFIED_ADDR);

    // Pre-flight: the four checks the guard re-runs at execution, all green.
    const checks = page.getByText("guard checks — these run again on-chain at execution");
    await expect(checks).toBeVisible();
    await expect(page.locator(".check-row").getByText("✓", { exact: true })).toHaveCount(4);
    await expect(page.getByRole("alert")).toHaveCount(0);

    await page.getByRole("button", { name: "Swap guarded" }).click();
    await expect(page.getByText("Settled", { exact: true })).toBeVisible();
    await expect(page.getByText("All guard checks passed at execution")).toBeVisible();
    await expect(page.getByText(/guard receipt/)).toBeVisible();
    await expect(page.getByText(/incl\. probes/)).toBeVisible();
  });
});

test.describe("J2 deterministic red flags — verbatim revert reasons (journeys.md:32)", () => {
  for (const reason of GUARD_REVERT_REASONS) {
    test(`refused: ${reason} surfaced byte-exact, funds never move`, async ({ page }) => {
      await connectWallet(page, VETTED_CHAIN_ID);
      const token = GUARD_SENTINELS[reason as GuardRevertReason];
      await page.getByLabel("receive token address").fill(token);

      // Preview names the exact deterministic red flag before execution.
      const alert = page.getByRole("alert");
      await expect(alert).toContainText(`the guard will refuse this swap on-chain: ${reason} — preview is advisory; the guard decides`);

      await page.getByRole("button", { name: "Swap guarded" }).click();
      const banner = page.locator(".banner.risk");
      await expect(banner).toContainText(`${reason}:`);
      // The funds-never-move wording is load-bearing (journeys.md:34).
      await expect(page.getByText(/never left your wallet/)).toBeVisible();
      await expect(page.getByText("Settled", { exact: true })).toHaveCount(0);

      if (reason === "GUARD_RECORD_REVOKED") {
        // The demo's upgrade beat: formerly verified, refused at execution time.
        await expect(page.getByText("Swap refused — verification revoked")).toBeVisible();
        await expect(page.getByText(/implementation changed, the registry revoked the stale record/)).toBeVisible();
      }
    });
  }
});
