/**
 * J3 live-build specs. The degraded state is route-forced (every registry
 * eth_call fails → the advisory banner), which works with or without a
 * scratch deployment. The populated-table spec is activation-gated: it needs
 * a funded scratch deployment in `deployments/*.json`, the worker's scratch
 * REGISTRY_ADDRESS var filled, and the seed script's records — the UI path
 * the VITE_REGISTRY_TOKENS table override opens (step-8 sanctioned fix 3).
 */
import { expect, test } from "@playwright/test";

import { WORKER_URL } from "../fixtures/constants";
import { stubPublicRpc } from "../fixtures/rpc";
import { SERVER_CHAIN_ID } from "../fixtures/scratch";
import { SCRATCH } from "../fixtures/scratch";

test.describe("J3 unhappy paths (route-forced)", () => {
  test("registry reads fail → degraded banner, records untouched, retry offered", async ({ page }) => {
    await stubPublicRpc(page, { chainId: SERVER_CHAIN_ID, failCalls: true });
    await page.goto("#/registry");

    const banner = page.getByRole("alert").filter({ hasText: "Issuer canonical list unreachable" });
    await expect(banner).toBeVisible();
    await expect(banner).toContainText("Existing records stay readable on-chain");
    await expect(page.getByText("No records could be read.")).toBeVisible();
    await expect(page.getByRole("link", { name: "Retry" })).toBeVisible();
    await expect(page.locator("table.table")).toHaveCount(0);
  });
});

test.describe("J3 live registry table (activation-gated)", () => {
  test("seeded records render in the table with the REVOKED drill-in", async ({ page, request }) => {
    test.skip(!SCRATCH, "no funded scratch deployment (deployments/46630.json|421614.json) — activation runbook: e2e/README.md");
    test.skip(!SCRATCH!.tokens.replica || !SCRATCH!.tokens.tokenB, "deployments JSON lacks replicas/mock token fields — run the step-3/6 deploy runbooks first");

    // Precondition: the worker can see the seeded replica record (its scratch
    // REGISTRY_ADDRESS var is filled). Otherwise the UI would honestly render
    // no-record rows and there would be nothing to assert.
    const res = await request.get(`${WORKER_URL}/scan?chainId=${SCRATCH!.chainId}&addr=${SCRATCH!.tokens.replica}`);
    test.skip(!res.ok(), `worker /scan not serving (${res.status()})`);
    const payload = (await res.json()) as { record: { status: string } | null };
    test.skip(payload.record === null, "worker serves record:null — REGISTRY_ADDRESS_<scratch> unfilled or seed script not run (README steps 3-4)");

    await page.goto("#/registry");
    const table = page.locator("table.table");
    await expect(table).toBeVisible();
    await expect(table.locator("tr", { hasText: SCRATCH!.tokens.replica! }).getByText("VERIFIED", { exact: true })).toBeVisible();
    await expect(table.locator("tr", { hasText: SCRATCH!.tokens.tokenB! }).getByText("REVOKED", { exact: true })).toBeVisible();
  });
});
