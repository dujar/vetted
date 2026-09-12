/**
 * Typed scan-backend client (plan task 1). Wire shapes come from
 * `vetted-shared` — the step-1 wire contract is the source of truth, so
 * step 4's real worker drops in behind the same `ScanClient` interface.
 *
 * Mock mode (default; `VITE_API_MODE=mock`) serves the step-1 golden fixtures
 * so the screens render fully with no backend — this is how the step ships in
 * parallel with step 4. Live mode (`VITE_API_MODE=live` + `VITE_API_URL`)
 * does GET /scan per packages/shared/wire.md.
 */
import type { ScanResponse } from "vetted-shared";

import { mockScanResponse, MOCK_WATCHDOG } from "./mockData";

export type { ScanResponse } from "vetted-shared";

/**
 * Watchdog response — pinned LOCALLY (verify.md loose end 3): the shared wire
 * has no watchdog type yet (endpoint exists as prose in step-4 task 5 — two
 * numbers, no stored history). Step 8 reconciles this into packages/shared
 * when the endpoint ships; until then this is the only definition.
 */
export interface WatchdogStats {
  chainId: number;
  /** Cumulative sequencer-filterer runs — one RPC read upstream. */
  runs: number;
  /** Published 6-week baseline, as a daily rate (~150/day, measured 2026-08). */
  baselinePerDay: number;
}

/** The transport seam step 4's real client satisfies. */
export interface ScanClient {
  scan(chainId: number, addr: string): Promise<ScanResponse>;
}

export interface WatchdogSource {
  stats(chainId: number): Promise<WatchdogStats>;
}

/** Live client — GET /scan?chainId=&addr= (wire.md). Throws on non-2xx; no retry logic here (the UI owns retryable states). */
export class FetchScanClient implements ScanClient {
  constructor(private readonly baseUrl: string) {}

  private get base(): string {
    return this.baseUrl.replace(/\/$/, "");
  }

  async scan(chainId: number, addr: string): Promise<ScanResponse> {
    const res = await fetch(
      `${this.base}/scan?chainId=${encodeURIComponent(String(chainId))}&addr=${encodeURIComponent(addr)}`,
    );
    if (!res.ok) {
      throw new Error(`scan backend ${res.status} for ${addr}`);
    }
    return (await res.json()) as ScanResponse;
  }
}

/** Fixture-backed client — deterministic, instant, no network. */
export class MockScanClient implements ScanClient {
  async scan(chainId: number, addr: string): Promise<ScanResponse> {
    // One microtask so callers exercise the same async paths as live mode.
    await Promise.resolve();
    return mockScanResponse(chainId, addr);
  }
}

export class FetchWatchdogSource implements WatchdogSource {
  constructor(private readonly baseUrl: string) {}

  async stats(chainId: number): Promise<WatchdogStats> {
    const res = await fetch(`${this.baseUrl.replace(/\/$/, "")}/watchdog?chainId=${encodeURIComponent(String(chainId))}`);
    if (!res.ok) throw new Error(`watchdog ${res.status}`);
    return (await res.json()) as WatchdogStats;
  }
}

/** Inline constants (verify.md loose end 4) until the /watchdog endpoint exists. */
export class MockWatchdogSource implements WatchdogSource {
  async stats(chainId: number): Promise<WatchdogStats> {
    await Promise.resolve();
    return { chainId, runs: MOCK_WATCHDOG.runs, baselinePerDay: MOCK_WATCHDOG.baselinePerDay };
  }
}

function liveBaseUrl(): string {
  const url = import.meta.env.VITE_API_URL;
  if (!url) throw new Error("VITE_API_MODE=live requires VITE_API_URL (the scan-backend worker URL)");
  return url.replace(/\/$/, "");
}

/** Mock by default so `npm run dev` works before step 4 ships; opt in with VITE_API_MODE=live. */
export function isLiveApi(): boolean {
  return import.meta.env.VITE_API_MODE === "live";
}

export function getScanClient(): ScanClient {
  return isLiveApi() ? new FetchScanClient(liveBaseUrl()) : new MockScanClient();
}

export function getWatchdogSource(): WatchdogSource {
  return isLiveApi() ? new FetchWatchdogSource(liveBaseUrl()) : new MockWatchdogSource();
}
