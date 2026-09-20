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
 * Watchdog response — re-exported from `vetted-shared` (step-8 reconciliation,
 * N7 / plan Revised 2026-09-20 note 1): the endpoint shipped in step 4 and the
 * shared `WatchdogStats` (watchdog.ts) is the canonical wire type, including
 * `provenanceUrl`. `runs: 0` WITH a `provenanceUrl` is the degrade sentinel —
 * count unavailable, never a measured zero (wire.md, Watchdog API).
 */
export type { WatchdogStats } from "vetted-shared";
import type { WatchdogStats } from "vetted-shared";

/** Mock-mode provenance — the published-baseline source behind the inline numbers. */
export const MOCK_WATCHDOG_PROVENANCE_URL =
  "https://docs.robinhood.com/chain/differences-from-ethereum";

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

/** Inline constants (verify.md loose end 4) — the spec's published figures; provenance is that source. */
export class MockWatchdogSource implements WatchdogSource {
  async stats(chainId: number): Promise<WatchdogStats> {
    await Promise.resolve();
    return {
      chainId,
      runs: MOCK_WATCHDOG.runs,
      baselinePerDay: MOCK_WATCHDOG.baselinePerDay,
      provenanceUrl: MOCK_WATCHDOG_PROVENANCE_URL,
    };
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
