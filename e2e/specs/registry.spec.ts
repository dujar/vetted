/**
 * J3 — consume the registry as a primitive (journeys.md:36-45), mock project:
 * the populated table with the REVOKED row red + drill-in, the criteria panel
 * (registrar, published criteria, on-chain read interface). The degraded and
 * empty states are forced in registry.live.spec.ts (route-level RPC failure)
 * — mock mode's source never fails — and pinned by the frontend unit suite.
 */
import { expect, test } from "@playwright/test";

import { FIXTURES, SELECTORS } from "../fixtures/constants";

const registrar = FIXTURES.VERIFIED.record!.registrar;

test.beforeEach(async ({ page }) => {
  await page.goto("#/registry");
  await expect(page.getByRole("heading", { name: "Canonical Registry" })).toBeVisible();
});

test("table lists the known tokens with statuses, impl pointers and dates", async ({ page }) => {
  const table = page.locator("table.table");
  for (const symbol of ["NVDA", "AMC", "OPENAI", "SPACEX"]) {
    await expect(table.getByRole("cell", { name: symbol, exact: true })).toBeVisible();
  }
  // Three VERIFIED badges + one REVOKED badge (red).
  await expect(table.getByText("VERIFIED", { exact: true })).toHaveCount(3);
  await expect(table.getByText("REVOKED", { exact: true })).toHaveCount(1);
  // Implementation pointer + verifiedAt columns carry the record data.
  await expect(table.getByText("0x77be…41af").first()).toBeVisible();
});

test("REVOKED row is red with reason and a drill-in", async ({ page }) => {
  const row = page.locator("tr", { hasText: "OPENAI" });
  await expect(row.getByText("REVOKED", { exact: true })).toBeVisible();
  await expect(row.getByText(/record stale/)).toBeVisible();

  await row.getByRole("link", { name: "evidence" }).click();
  const drillIn = page.locator("td", { hasText: "revocation tx" });
  await expect(drillIn).toContainText("reason: beacon impl changed");
  await expect(drillIn.getByRole("link", { name: "0xf4c0…71b8" })).toBeVisible();
});

test("criteria panel: registrar, published criteria link, on-chain read interface", async ({ page }) => {
  await expect(page.getByText(`registrar: 0x5e15…aa11`)).toBeVisible();
  const criteria = page.getByRole("link", { name: /Verification criteria, published in the repo/ });
  await expect(criteria).toHaveAttribute("href", "https://github.com/dujar/vetted/blob/main/docs/criteria.md");

  // The read interface — the primitive pitch (journeys.md:43): the pinned
  // signature + selector, straight from vetted-shared.
  await expect(page.getByText("function getRecord(address token) view returns (")).toBeVisible();
  await expect(page.getByText(`selector ${SELECTORS.getRecord}`)).toBeVisible();
});

test("rows deep-link into the scanner", async ({ page }) => {
  await page.locator("tr", { hasText: "NVDA" }).getByRole("link", { name: "scan" }).click();
  await expect(page.locator("section.panel", { hasText: FIXTURES.VERIFIED.addr }).getByText("VERIFIED", { exact: true })).toBeVisible();
});
