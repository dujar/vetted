/**
 * J1 — verify a stock token before acquiring (journeys.md:6-22), mock
 * project: every verdict state + every unhappy path the mock transport can
 * express, over the real bundle. Live-worker payload honesty lives in
 * scan.live.spec.ts (route-intercepted / forced where a live failure cannot
 * be produced deterministically).
 *
 * J1 is wallet-free by design (journeys.md:19) — the stub wallet fixture is
 * never installed here, and the specs pin that property.
 */
import { expect, test, type Page } from "@playwright/test";

import {
  FIXTURES,
  MOCK_DEGRADED_ADDR,
  MOCK_IMPOSTOR_ADDR,
  MOCK_NOT_CONTRACT_ADDR,
  MOCK_REVOKED_ADDR,
  MOCK_RPC_ERROR_ADDR,
  MOCK_VERIFIED_ADDR,
  RETRY_BACKOFF_MS,
} from "../fixtures/constants";

/** Typed-input entry (journeys.md:13 step 1) — paste an address, hit Scan. */
async function scanTyped(page: Page, addr: string): Promise<void> {
  await page.goto("#/");
  await page.getByLabel("token address").fill(addr);
  await page.getByRole("button", { name: "Scan", exact: true }).click();
}

const VERIFIED_ADDR = MOCK_VERIFIED_ADDR;

test.describe("J1 happy paths", () => {
  test("typed scan of the canonical token: VERIFIED, evidence-linked power report, watchdog", async ({ page }) => {
    await scanTyped(page, VERIFIED_ADDR);

    const card = page.locator("section.panel", { hasText: VERIFIED_ADDR });
    await expect(card.getByText("VERIFIED", { exact: true })).toBeVisible();
    await expect(card.getByText("NVIDIA")).toBeVisible();
    // Power report: every line carries its evidence link (journeys.md:15 / spec.md:30).
    const rows = card.locator(".report-row");
    await expect(rows).toHaveCount(FIXTURES.VERIFIED.powerReport.length);
    for (const row of FIXTURES.VERIFIED.powerReport) {
      const line = rows.filter({ hasText: row.check });
      await expect(line).toContainText(row.result);
      await expect(line.getByRole("link", { name: "evidence" })).toHaveAttribute("href", row.evidenceUrl);
    }
    // A VERIFIED token can still carry red power flags — legitimacy ≠ safety.
    await expect(card.getByText(/Verified ≠ safe/)).toBeVisible();

    // Watchdog widget beside the verdict (journeys.md:16): two numbers, no history.
    await expect(page.getByText("sequencer filterer — censorship watchdog")).toBeVisible();
    await expect(page.getByText("6,092", { exact: true })).toBeVisible();
    await expect(page.getByText("~150/day")).toBeVisible();
  });

  test("compare deep link: canonical replica vs impostor twin side by side, both entry formats", async ({ page }) => {
    // journeys.md:9 bare format — no hash at all.
    await page.goto(`/?addr=${VERIFIED_ADDR}&addr=${MOCK_IMPOSTOR_ADDR}`);
    const cards = page.locator("section.panel");
    const impostor = cards.filter({ hasText: "This is not a Robinhood Stock Token." });
    const verified = cards.filter({ hasText: "· Robinhood Stock Token" });
    await expect(impostor).toBeVisible();
    await expect(verified.getByText("VERIFIED", { exact: true })).toBeVisible();
    await expect(verified.getByText("IMPOSTOR", { exact: true })).toHaveCount(0);
    await expect(impostor.getByText("IMPOSTOR", { exact: true })).toBeVisible();
    // Mimicry rows with evidence links (the demo's impostor-vs-canonical beat).
    await expect(impostor.getByRole("link", { name: "evidence" }).first()).toBeVisible();
    // Two-column compare layout.
    await expect(page.locator(".cards")).toBeVisible();

    // Same beat via the in-app hash format.
    await page.goto(`#/?addr=${VERIFIED_ADDR}&addr=${MOCK_IMPOSTOR_ADDR}`);
    await expect(cards.filter({ hasText: "This is not a Robinhood Stock Token." })).toBeVisible();
  });

  test("J1 is wallet-free: no EIP-1193 provider is ever injected on the scan page", async ({ page }) => {
    await page.goto(`#/?addr=${VERIFIED_ADDR}`);
    await expect(page.getByText("VERIFIED", { exact: true })).toBeVisible();
    expect(await page.evaluate(() => (window as { ethereum?: unknown }).ethereum)).toBeUndefined();
  });
});

test.describe("J1 verdict states", () => {
  test("REVOKED: red verdict, decoded reason, revocation tx evidence link", async ({ page }) => {
    await page.goto(`#/?addr=${MOCK_REVOKED_ADDR}`);
    const card = page.locator("section.panel", { hasText: MOCK_REVOKED_ADDR });
    await expect(card.getByText("REVOKED", { exact: true })).toBeVisible();
    await expect(card.getByText(/Verification revoked — this token changed/)).toBeVisible();
    await expect(card.getByText(/beacon impl changed/)).toBeVisible(); // decodeBytes32(record.reason)
    const txLink = card.locator("a", { hasText: "0xab12…ab12" });
    await expect(txLink).toHaveAttribute("href", `https://robinhoodchain.blockscout.com/tx/${FIXTURES.REVOKED.revocationTx}`);
  });

  test("address has no code → not-a-contract state, no verdict attempted", async ({ page }) => {
    await page.goto(`#/?addr=${MOCK_NOT_CONTRACT_ADDR}`);
    await expect(page.getByText("not a contract")).toBeVisible();
    await expect(page.getByText(/has no code — a wallet address, not a token/)).toBeVisible();
    await expect(page.locator(".badge")).toHaveCount(0);
  });

  test("off-pattern contract → UNVERIFIED + advisory heuristics + depth-boundary notice", async ({ page }) => {
    const unknown = `0x${"11".repeat(20)}`;
    await page.goto(`#/?addr=${unknown}`);
    const card = page.locator("section.panel", { hasText: unknown });
    await expect(card.getByText("UNVERIFIED", { exact: true })).toBeVisible();
    await expect(card.locator(".report-row", { hasText: "Structural heuristics (advisory — no signature match)" })).toContainText("OFF_PATTERN");
    // Depth-boundary copy is visible when the notice is null (journeys.md:22).
    await expect(card.getByText(/Depth boundary: full verdicts cover the Robinhood stock-token pattern/)).toBeVisible();
  });

  test("degraded payload → ground-truth-down banner, every verdict UNVERIFIED", async ({ page }) => {
    await page.goto(`#/?addr=${MOCK_DEGRADED_ADDR}`);
    const banner = page.getByRole("alert").filter({ hasText: "Issuer canonical list unreachable" });
    await expect(banner).toBeVisible();
    await expect(banner).toContainText("All verdicts drop to UNVERIFIED until it responds");
    await expect(page.locator("section.panel").getByText("UNVERIFIED", { exact: true })).toBeVisible();
  });
});

test.describe("J1 unhappy paths", () => {
  test("RPC failure → retryable error state, nothing guessed; retry re-runs the scan", async ({ page }) => {
    await page.goto(`#/?addr=${MOCK_RPC_ERROR_ADDR}`);
    await expect(page.getByText("rpc unreachable").first()).toBeVisible();
    await expect(page.getByText(/No partial verdicts are shown/)).toBeVisible();

    // Retry (honest-degrade policy: RPC_RETRYABLE → backoff retry) re-runs —
    // deterministic sentinel errors again, so the panel is the re-attempt proof.
    await page.waitForTimeout(RETRY_BACKOFF_MS);
    await page.getByRole("link", { name: "Retry scan" }).click();
    await expect(page.getByText("rpc unreachable").first()).toBeVisible();
  });

  test("selector on another chain → read-only notice; stock-token verdicts exist only on 4663", async ({ page }) => {
    await page.goto("#/");
    await page.getByLabel("network selector").selectOption("421614");
    await page.getByLabel("token address").fill(VERIFIED_ADDR);
    await page.getByRole("button", { name: "Scan", exact: true }).click();

    const notice = page.getByText("wrong network", { exact: true });
    await expect(notice).toBeVisible();
    await expect(page.getByText(/is scanned read-only/)).toBeVisible();
    await expect(page.locator("section.panel").getByText("UNVERIFIED", { exact: true })).toBeVisible();
    await expect(page.locator("section.panel").getByText("VERIFIED", { exact: true })).toHaveCount(0);

    // The notice's switch control returns the scanner to 4663 and rescans.
    await page.getByRole("link", { name: "Switch to 4663" }).click();
    await expect(page.locator("section.panel").getByText("VERIFIED", { exact: true })).toBeVisible();
  });
});
