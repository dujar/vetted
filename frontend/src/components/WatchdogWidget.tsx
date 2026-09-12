/**
 * Watchdog widget (spec scope 5) — sequencer-filterer activity at a glance:
 * cumulative runs vs the published daily baseline. Two numbers, no history.
 */
import type { WatchdogStats } from "../lib/api";
import { formatInt } from "../lib/format";

export function WatchdogWidget({ stats }: { stats: WatchdogStats | null }) {
  return (
    <div className="panel">
      <div className="label" style={{ marginBottom: "var(--s2)" }}>
        sequencer filterer — censorship watchdog
      </div>
      <div style={{ display: "flex", gap: "var(--s4)", alignItems: "baseline" }}>
        <div>
          <span className="mono" style={{ fontSize: "var(--fs-4)" }}>
            {stats ? formatInt(stats.runs) : "…"}
          </span>
          <div className="faint mono" style={{ fontSize: "var(--fs-0)" }}>runs since launch — one RPC read</div>
        </div>
        <div>
          <span className="mono">{stats ? `~${formatInt(stats.baselinePerDay)}/day` : "…"}</span>
          <div className="faint mono" style={{ fontSize: "var(--fs-0)" }}>6-week baseline</div>
        </div>
      </div>
      <p className="faint" style={{ fontSize: "var(--fs-1)", marginBottom: 0 }}>
        Cumulative filterer executions — no published criteria, no appeals.{" "}
        <a href="https://docs.robinhood.com/chain/differences-from-ethereum" target="_blank" rel="noreferrer">
          Method
        </a>
      </p>
    </div>
  );
}
