/**
 * J1 live-build specs. Everything here is route-forced (plan Goal: "route-
 * intercepted where a live failure cannot be forced"): the worker's /scan
 * shape is fulfilled locally for the degraded + retry journeys, and the
 * scratch-chain replica/twin spec activates only from a funded
 * `deployments/*.json` + a worker that already serves the seeded record —
 * never from hand-forged state (plan Revised 2026-09-20 note 3).
 *
 * The live-server bundle targets the vetted chain (4663; VITE_CHAIN_ID when a
 * scratch deployment exists) with VITE_API_MODE=live.
 */
import { expect, test, type Page } from "@playwright/test";

import {
  FIXTURES,
  RETRY_BACKOFF_MS,
  RETRY_MAX_ATTEMPTS,
  WORKER_URL,
} from "../fixtures/constants";
import { SCRATCH, SERVER_CHAIN_ID } from "../fixtures/scratch";

const WORKER_HOST = new URL(WORKER_URL).hostname;

/** Intercept GET /scan + /watchdog on the worker host. Handler per-call. */
async function interceptWorkerScan(
  page: Page,
  handler: (route: import("@playwright/test").Route, url: URL) => Promise<void>,
): Promise<void> {
  await page.route(
    (url) => url.hostname === WORKER_HOST && (url.pathname === "/scan" || url.pathname === "/watchdog"),
    (route) => handler(route, new URL(route.request().url())),
  );
}

const fulfillScan = (status: number, payload: unknown) => ({
  status,
  contentType: "application/json",
  body: JSON.stringify(payload),
});

/**
 * Honest-degrade retry policy (constants: RETRY_BACKOFF_MS / RETRY_MAX_ATTEMPTS):
 * while the scan keeps landing in a retryable state, back off and retry the SAME
 * address — up to RETRY_MAX_ATTEMPTS. Deterministic routes exit on the first
 * retry; against the flaky live worker this is the real policy.
 */
async function retryScanUntilSettled(page: Page): Promise<void> {
  for (let attempt = 1; attempt <= RETRY_MAX_ATTEMPTS; attempt += 1) {
    const retry = page.getByRole("link", { name: "Retry scan" });
    if ((await retry.count()) === 0) return;
    await page.waitForTimeout(RETRY_BACKOFF_MS * attempt);
    await retry.first().click();
  }
}

test.describe("J1 unhappy paths against the live wire (route-forced)", () => {
  test("progress lines render in order while the scan is in flight; degraded payload → banner", async ({ page }) => {
    await interceptWorkerScan(page, async (route, url) => {
      if (url.pathname === "/watchdog") {
        await route.fulfill(fulfillScan(200, { chainId: 4663, runs: 0, baselinePerDay: 150, provenanceUrl: `${WORKER_URL.replace(/\/$/, "")}/provenance` }));
        return;
      }
      await new Promise((r) => setTimeout(r, 1_200)); // hold the transport open — the mock pass can't see the progress panel
      await route.fulfill(
        fulfillScan(200, {
          chainId: SERVER_CHAIN_ID,
          addr: url.searchParams.get("addr"),
          verdict: "UNVERIFIED",
          terminalState: null,
          degraded: true,
          notice: null,
          powerReport: [],
          record: null,
          revocationTx: null,
        }),
      );
    });

    await page.goto(`#/?addr=${`0x${"ab".repeat(20)}`}`);
    // ScanPage renders the progress panel twice while a fresh scan is in
    // flight (standalone + per-run) — assert against the first.
    const log = page.getByRole("status", { name: "scanning" }).first();
    // journeys.md:13, in order: bytecode → slot → probes → canonical list.
    await expect(log.getByText("fetch bytecode")).toBeVisible();
    await expect(log.getByText("resolve EIP-1967 implementation slot")).toBeVisible();
    await expect(log.getByText("run probes: paused() · buyer blocklist")).toBeVisible();
    await expect(log.getByText(/fetching issuer canonical list/)).toBeVisible();

    // Degraded:true payload (issuer list down): the banner, every verdict UNVERIFIED.
    const banner = page.getByRole("alert").filter({ hasText: "Issuer canonical list unreachable" });
    await expect(banner).toBeVisible();
    await expect(page.locator("section.panel").getByText("UNVERIFIED", { exact: true })).toBeVisible();
    // The degrade sentinel: the widget shows count-unavailable (runs: 0 WITH a
    // provenance URL), never a measured zero — scoped to the watchdog panel.
    const watchdog = page.locator(".panel", { hasText: "sequencer filterer — censorship watchdog" });
    await expect(watchdog.getByText("0", { exact: true })).toBeVisible();
  });

  test("RPC_RETRYABLE → backoff retry policy recovers the verdict (worker 503 then success)", async ({ page }) => {
    let scans = 0;
    await interceptWorkerScan(page, async (route, url) => {
      if (url.pathname === "/watchdog") {
        await route.fulfill(fulfillScan(200, { chainId: SERVER_CHAIN_ID, runs: 0, baselinePerDay: 150, provenanceUrl: "https://docs.robinhood.com/chain/differences-from-ethereum" }));
        return;
      }
      scans += 1;
      if (scans === 1) {
        await route.fulfill(fulfillScan(503, { error: "upstream rpc throttled" }));
        return;
      }
      await route.fulfill(fulfillScan(200, { ...FIXTURES.VERIFIED, chainId: SERVER_CHAIN_ID }));
    });

    await page.goto(`#/?addr=${FIXTURES.VERIFIED.addr}`);
    // First attempt fails hard (fetch throw): the retryable error state.
    await expect(page.getByText("rpc unreachable").first()).toBeVisible();

    // Honest-degrade policy: backoff, then retry the SAME address — the
    // deterministic route recovers on the first retry (request count pinned).
    await retryScanUntilSettled(page);
    await expect(page.locator("section.panel").getByText("VERIFIED", { exact: true })).toBeVisible();
    expect(scans).toBe(2);
  });
});

test.describe("live worker payloads (transport honesty — wire.md)", () => {
  test("/watchdog serves the degrade sentinel: runs 0 WITH provenance — count unavailable, never a measured zero", async ({ request }) => {
    let body: Record<string, unknown> | null = null;
    try {
      const res = await request.get(`${WORKER_URL}/watchdog?chainId=4663`);
      if (res.ok()) body = (await res.json()) as Record<string, unknown>;
    } catch {
      /* worker unreachable — the environmental-degrade case */
    }
    test.skip(body === null, "live worker unreachable — environmental (step-4 findings); not a product failure");
    expect(body!.runs).toBe(0);
    expect(typeof body!.provenanceUrl).toBe("string");
    expect((body!.provenanceUrl as string).length).toBeGreaterThan(0);
    expect(body!.baselinePerDay).toBe(150);
  });

  test("/scan never fabricates: a VERIFIED verdict always carries evidence rows", async ({ request }) => {
    let body: Record<string, unknown> | null = null;
    try {
      const res = await request.get(`${WORKER_URL}/scan?chainId=4663&addr=${FIXTURES.VERIFIED.addr}`);
      if (res.ok()) body = (await res.json()) as Record<string, unknown>;
    } catch {
      /* worker unreachable */
    }
    test.skip(body === null, "live worker unreachable — environmental; the designed RPC_RETRYABLE/honest-UNVERIFIED oscillation is covered by the retry policy");
    const verdict = body!.verdict as string | null;
    if (verdict === "VERIFIED") {
      const rows = body!.powerReport as unknown[];
      expect(rows.length).toBeGreaterThan(0); // no evidence-less VERIFIED, ever
    }
    // RPC_RETRYABLE and honest-UNVERIFIED are legitimate outcomes under the 4663
    // shared-egress rate limit (plan Revised 2026-09-20 note 2) — not failures.
  });
});

test.describe("scratch-chain replica vs twin (activation-gated)", () => {
  test("replica VERIFIED (with the non-4663 notice) vs twin UNVERIFIED — IMPOSTOR is 4663-only", async ({ page, request }) => {
    test.skip(!SCRATCH, "no funded scratch deployment (deployments/46630.json|421614.json) — activation runbook: e2e/README.md");
    test.skip(!SCRATCH!.tokens.replica || !SCRATCH!.tokens.twin1, "deployments JSON lacks the step-6 replicas fields — run the step-6 deploy runbook first");

    // Precondition on the worker side: the scratch registry var is filled AND
    // the seed script has verified the replica. Otherwise the scan honestly
    // serves record:null → UNVERIFIED, and this spec would assert a half-seeded world.
    const res = await request.get(`${WORKER_URL}/scan?chainId=${SCRATCH!.chainId}&addr=${SCRATCH!.tokens.replica}`);
    test.skip(!res.ok(), `worker /scan not serving (${res.status()})`);
    const payload = (await res.json()) as { record: { status: string } | null };
    test.skip(payload.record === null, "worker serves record:null — REGISTRY_ADDRESS_<scratch> unfilled or seed script not run (README steps 3-4)");

    const replicaCard = page.locator("section.panel", { hasText: SCRATCH!.tokens.replica! });
    await page.goto(`#/?addr=${SCRATCH!.tokens.replica!}`);
    await expect(replicaCard.getByText("VERIFIED", { exact: true })).toBeVisible();
    // The worker attaches the read-only notice off 4663 (rules.rs) — the UI shows it.
    await expect(page.getByText(/scanned read-only|verdicts exist only on 4663/i)).toBeVisible();

    const twinCard = page.locator("section.panel", { hasText: SCRATCH!.tokens.twin1! });
    await page.goto(`#/?addr=${SCRATCH!.tokens.twin1!}`);
    await expect(twinCard.getByText("UNVERIFIED", { exact: true })).toBeVisible();
    await expect(twinCard.getByText("IMPOSTOR", { exact: true })).toHaveCount(0); // no canonical fetch off 4663
    // Honest heuristics only: no fabricated ABSENT/verified rows on a scratch chain.
    for (const row of await twinCard.locator(".report-row").all()) {
      expect(await row.innerText()).not.toMatch(/ABSENT/);
    }
  });
});
