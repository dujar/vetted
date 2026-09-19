/**
 * Watchdog widget wire type (step 4) — the shared contract for
 * GET /watchdog?chainId= (step-4 task 5). Two numbers + provenance, no stored
 * history (spec.md:37). Step 5 declared this shape locally in
 * frontend/src/lib/api.ts (verify loose end 3); this module is the canonical
 * home — the frontend switches to it at step 8's reconciliation.
 */
export interface WatchdogStats {
  chainId: number;
  /**
   * Cumulative sequencer-filterer runs — one RPC read upstream.
   * `0` WITH a `provenanceUrl` means the live count is unavailable
   * (the degrade path), never a measured zero (wire.md, Watchdog API).
   */
  runs: number;
  /** Published 6-week baseline, as a daily rate (~150/day, measured 2026-08). */
  baselinePerDay: number;
  /**
   * Where the numbers come from: the live counter's public entry, or the
   * published-baseline source when degraded. null = no provenance at all.
   */
  provenanceUrl: string | null;
}
