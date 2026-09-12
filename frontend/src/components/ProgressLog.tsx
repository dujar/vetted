/**
 * Scanning progress (journeys.md:13) — the four probe lines, revealed in
 * order while the scan transport is in flight. The step cadence is display
 * theater over a single GET /scan; the last line stays "fetching" until the
 * response lands (the canonical-list fetch is the slow step upstream).
 */
import { useEffect, useState } from "react";

export const SCAN_STEPS = [
  "fetch bytecode",
  "resolve EIP-1967 implementation slot",
  "run probes: paused() · buyer blocklist",
] as const;

/** Revealed-step counter, ticking every `ms` while `active`. */
export function useStepClock(active: boolean, ms = 450): number {
  const [step, setStep] = useState(0);
  useEffect(() => {
    if (!active) {
      setStep(0);
      return;
    }
    setStep(0);
    const id = window.setInterval(() => {
      setStep((s) => Math.min(s + 1, SCAN_STEPS.length + 1));
    }, ms);
    return () => window.clearInterval(id);
  }, [active, ms]);
  return step;
}

export function ProgressLog({ step }: { step: number }) {
  return (
    <div className="panel mono" style={{ display: "grid", gap: "var(--s2)" }} role="status" aria-label="scanning">
      {SCAN_STEPS.map((label, i) => (
        <div key={label}>
          {step > i ? <span style={{ color: "var(--verified)" }}>✓</span> : <span className="faint">…</span>}{" "}
          {label}
        </div>
      ))}
      <div className="faint">
        {step > SCAN_STEPS.length ? "✓" : "…"} fetching issuer canonical list — docs.robinhood.com/chain/contracts
      </div>
    </div>
  );
}
