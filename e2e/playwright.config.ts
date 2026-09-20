/**
 * Playwright config (plan task 1) — two always-built servers over the real
 * frontend bundle:
 *   · mock  (port 4883) — fixture-backed app (VITE_API_MODE unset = mock), the
 *     default project; runs offline-capable against the golden fixtures.
 *   · live  (port 4884) — VITE_API_MODE=live + the step-4 worker URL, plus the
 *     e2e wallet seam. When a FUNDED scratch deployment exists
 *     (deployments/46630.json or 421614.json), the bundle is re-pointed at it
 *     (VITE_CHAIN_ID + VITE_REGISTRY_TOKENS — the live J2/J3 path); otherwise
 *     sentinel addresses stand in and the scratch-gated specs skip honestly.
 *
 * Both servers add VITE_E2E_STUB_WALLET=1 (wagmi injected() seam — verify
 * loose end 1). Product builds set none of these and are unchanged.
 */
import { defineConfig, devices } from "@playwright/test";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { LIVE_PORT, MOCK_PORT, SENTINEL_GUARD, SENTINEL_REGISTRY, WORKER_URL } from "./fixtures/constants";
import { SCRATCH } from "./fixtures/scratch";

const here = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(here, "..");

const scratchEnv: Record<string, string> = SCRATCH
  ? {
      VITE_CHAIN_ID: String(SCRATCH.chainId),
      VITE_REGISTRY_ADDRESS: SCRATCH.registry,
      VITE_GUARD_ADDRESS: SCRATCH.guard,
      VITE_REGISTRY_TOKENS: [
        SCRATCH.tokens.replica,
        SCRATCH.tokens.tokenB,
        SCRATCH.tokens.twin1,
        SCRATCH.tokens.twin2,
        SCRATCH.tokens.tokenPlain,
      ]
        .filter(Boolean)
        .join(","),
    }
  : {
      VITE_REGISTRY_ADDRESS: SENTINEL_REGISTRY,
      VITE_GUARD_ADDRESS: SENTINEL_GUARD,
    };

const frontend = path.join(ROOT, "frontend");

export default defineConfig({
  testDir: "./specs",
  fullyParallel: true,
  timeout: 45_000,
  reporter: "list",
  use: {
    trace: "retain-on-failure",
    // The EIP-6963 announce + route interception both assume main-world pages.
    serviceWorkers: "block",
  },
  projects: [
    {
      name: "mock",
      testIgnore: /.*\.live\.spec\.ts/,
      // VETTED_E2E_BASE_URL re-points the (wallet-free) mock specs at a
      // deployed frontend — e.g. the production Pages URL — instead of the
      // local server.
      use: {
        ...devices["Desktop Chrome"],
        baseURL: process.env.VETTED_E2E_BASE_URL ?? `http://127.0.0.1:${MOCK_PORT}`,
      },
    },
    {
      name: "live",
      testMatch: /.*\.live\.spec\.ts/,
      use: { ...devices["Desktop Chrome"], baseURL: `http://127.0.0.1:${LIVE_PORT}` },
    },
  ],
  webServer: [
    {
      command: `npm run dev -- --port ${MOCK_PORT} --strictPort`,
      cwd: frontend,
      url: `http://127.0.0.1:${MOCK_PORT}`,
      reuseExistingServer: true,
      timeout: 120_000,
      env: { VITE_E2E_STUB_WALLET: "1" },
    },
    {
      command: `npm run dev -- --port ${LIVE_PORT} --strictPort`,
      cwd: frontend,
      url: `http://127.0.0.1:${LIVE_PORT}`,
      reuseExistingServer: true,
      timeout: 120_000,
      env: { VITE_E2E_STUB_WALLET: "1", VITE_API_MODE: "live", VITE_API_URL: WORKER_URL, ...scratchEnv },
    },
  ],
});
