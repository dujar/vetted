/**
 * Scan screen (plan task 2, J1) — states per screens/scan.html, all derived
 * from journeys.md's state matrix: empty hero, in-order progress lines, four
 * verdict cards (compare deep link renders two side by side), watchdog
 * widget, and the unhappy paths (not-a-contract, RPC-retryable, degraded,
 * non-4663 notice). Wallet-free by design — the network selector is a UI
 * control (journeys.md:12).
 */
import { useEffect, useRef, useState } from "react";
import type { ScanResponse } from "vetted-shared";

import { VerdictCard, DOCS_CANONICAL_URL } from "../components/VerdictCard";
import { ProgressLog, useStepClock } from "../components/ProgressLog";
import { WatchdogWidget } from "../components/WatchdogWidget";
import { getScanClient, getWatchdogSource, type ScanClient, type WatchdogSource, type WatchdogStats } from "../lib/api";
import { robinhoodChain } from "../lib/chains";
import { MOCK_IMPOSTOR_ADDR, MOCK_REVOKED_ADDR, MOCK_VERIFIED_ADDR } from "../lib/mockData";
import { navigate, useHashRoute } from "../lib/router";

const ADDR_RE = /^0x[0-9a-fA-F]{40}$/;

interface Run {
  addr: string;
  phase: "loading" | "done" | "error";
  response?: ScanResponse;
}

export interface ScanPageProps {
  chainId: number;
  onChainChange?: (chainId: number) => void;
  client?: ScanClient;
  watchdog?: WatchdogSource;
}

// Default sources are lazy module singletons: a fresh instance per render
// (what a default parameter evaluates to) would re-fire the watchdog effect
// every cycle — an infinite fetch→set→render loop in the browser (found by
// the e2e journeys; unit tests inject stable props so they never saw it).
let defaultScanClient: ScanClient | null = null;
let defaultWatchdog: WatchdogSource | null = null;
const scanClient = (): ScanClient => (defaultScanClient ??= getScanClient());
const watchdogSource = (): WatchdogSource => (defaultWatchdog ??= getWatchdogSource());

export function ScanPage({
  chainId,
  onChainChange,
  client = scanClient(),
  watchdog = watchdogSource(),
}: ScanPageProps) {
  const route = useHashRoute();
  const addrs = route.path === "/" ? route.query.getAll("addr") : [];
  const [runs, setRuns] = useState<Run[]>([]);
  const [input, setInput] = useState("");
  const [inputError, setInputError] = useState<string | null>(null);
  const [watchdogStats, setWatchdogStats] = useState<WatchdogStats | null>(null);

  const running = runs.some((r) => r.phase === "loading");
  const step = useStepClock(running);

  useEffect(() => {
    let alive = true;
    watchdog
      .stats(chainId)
      .then((s) => {
        if (alive) setWatchdogStats(s);
      })
      .catch(() => {
        if (alive) setWatchdogStats(null);
      });
    return () => {
      alive = false;
    };
  }, [chainId, watchdog]);

  const addrKey = addrs.join(",");
  // Key on chain too — switching the network selector rescans instead of
  // leaving stale cards from the previous chain (review round 1).
  const scanKey = `${chainId}|${addrKey}`;
  const scannedKey = useRef("");
  useEffect(() => {
    if (addrKey === "" || scannedKey.current === scanKey) return;
    scannedKey.current = scanKey;
    void runScan(addrs);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [scanKey]);

  async function runScan(list: string[]) {
    setRuns(list.map((addr) => ({ addr, phase: "loading" as const })));
    await Promise.all(
      list.map(async (addr, i) => {
        try {
          const response = await client.scan(chainId, addr);
          setRuns((prev) => prev.map((r, j) => (j === i ? { addr, phase: "done" as const, response } : r)));
        } catch {
          setRuns((prev) => prev.map((r, j) => (j === i ? { addr, phase: "error" as const } : r)));
        }
      }),
    );
  }

  function submit() {
    const addr = input.trim();
    if (!ADDR_RE.test(addr)) {
      setInputError("Enter a 0x… contract address (40 hex characters).");
      return;
    }
    setInputError(null);
    navigate(`#/?addr=${addr}`);
  }

  const compare = runs.length > 1;
  const degraded = runs.some((r) => r.response?.degraded);
  const notice = runs.find((r) => r.response?.notice)?.response?.notice ?? null;

  return (
    <>
      {runs.length === 0 ? (
        <div className="hero" style={{ textAlign: "center", margin: "var(--s6) 0" }}>
          <h1 style={{ fontSize: "var(--fs-5)", margin: "0 0 var(--s2)" }}>
            Is this stock token real — and what can it do to you?
          </h1>
          <p className="muted" style={{ marginTop: 0 }}>
            Paste any token address on Robinhood Chain. Evidence-linked verdicts, no listing required,
            nothing guessed.
          </p>
          <div className="scanline" style={{ display: "flex", gap: "var(--s3)" }}>
            <input
              className="input"
              style={{ flex: 1 }}
              type="text"
              placeholder="0x… token address"
              spellCheck={false}
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && submit()}
              aria-label="token address"
            />
            <button className="btn" onClick={submit}>Scan</button>
          </div>
          {inputError ? (
            <p className="mono" style={{ color: "var(--risk)", fontSize: "var(--fs-1)" }} role="alert">
              {inputError}
            </p>
          ) : null}
          <p className="faint mono" style={{ fontSize: "var(--fs-0)" }}>
            bytecode · EIP-1967 slot · execution probes · issuer canonical list, fetched live
          </p>
          <p className="faint" style={{ fontSize: "var(--fs-1)" }}>
            Try:{" "}
            <a
              href={`#/?addr=${MOCK_VERIFIED_ADDR}`}
              onClick={(e) => {
                e.preventDefault();
                navigate(`#/?addr=${MOCK_VERIFIED_ADDR}`);
              }}
            >
              canonical NVIDIA (NVDA)
            </a>{" "}
            ·{" "}
            <a
              href={`#/?addr=${MOCK_VERIFIED_ADDR}&addr=${MOCK_IMPOSTOR_ADDR}`}
              onClick={(e) => {
                e.preventDefault();
                navigate(`#/?addr=${MOCK_VERIFIED_ADDR}&addr=${MOCK_IMPOSTOR_ADDR}`);
              }}
            >
              impostor twin — side-by-side compare
            </a>{" "}
            ·{" "}
            <a
              href={`#/?addr=${MOCK_REVOKED_ADDR}`}
              onClick={(e) => {
                e.preventDefault();
                navigate(`#/?addr=${MOCK_REVOKED_ADDR}`);
              }}
            >
              revoked after beacon upgrade
            </a>
          </p>
        </div>
      ) : null}

      {running && !runs.some((r) => r.phase !== "loading") ? <ProgressLog step={step} /> : null}

      {degraded ? (
        <div className="banner advisory" style={{ marginBottom: "var(--s4)" }} role="alert">
          Issuer canonical list unreachable. All verdicts drop to UNVERIFIED until it responds.
        </div>
      ) : null}

      {notice ? (
        <div className="panel" style={{ marginBottom: "var(--s4)" }}>
          <div className="label" style={{ marginBottom: "var(--s2)" }}>wrong network</div>
          <p className="muted" style={{ fontSize: "var(--fs-1)", margin: "0 0 var(--s2)" }}>
            {notice} Scanning stays read-only; no wallet involved.
          </p>
          <a
            href="#/"
            onClick={(e) => {
              e.preventDefault();
              onChainChange?.(robinhoodChain.id);
            }}
            style={{ fontSize: "var(--fs-1)" }}
          >
            Switch to 4663
          </a>
        </div>
      ) : null}

      <div className={compare ? "cards" : undefined} style={compare ? { display: "grid", gridTemplateColumns: "repeat(2, 1fr)", gap: "var(--s3)" } : undefined}>
        {runs.map((run) => (
          <RunResult key={run.addr} run={run} onRetry={() => void runScan([run.addr])} />
        ))}
      </div>

      {!compare && runs.some((r) => r.phase === "done" && r.response?.verdict !== null) ? (
        <div style={{ display: "grid", gridTemplateColumns: "1.6fr 1fr", gap: "var(--s4)", marginTop: "var(--s4)" }}>
          <div />
          <div>
            <div style={{ marginBottom: "var(--s3)" }}>
              <WatchdogWidget stats={watchdogStats} />
            </div>
            <div className="panel">
              <div className="label" style={{ marginBottom: "var(--s2)" }}>ground truth</div>
              <p className="muted" style={{ fontSize: "var(--fs-1)", margin: 0 }}>
                Issuer canonical list fetched live at scan time —{" "}
                <a href={DOCS_CANONICAL_URL} target="_blank" rel="noreferrer">docs.robinhood.com/chain/contracts</a>.{" "}
                <a href={DOCS_CANONICAL_URL} target="_blank" rel="noreferrer">Fetch receipt</a>
              </p>
            </div>
          </div>
        </div>
      ) : null}
    </>
  );
}

function RunResult({ run, onRetry }: { run: Run; onRetry: () => void }) {
  if (run.phase === "loading") return <ProgressLog step={1} />;
  if (run.phase === "error") {
    return (
      <div className="panel">
        <div className="label" style={{ marginBottom: "var(--s2)" }}>rpc unreachable</div>
        <p className="muted" style={{ fontSize: "var(--fs-1)", margin: "0 0 var(--s2)" }}>
          Chain RPC failed mid-probe. No partial verdicts are shown.
        </p>
        <a href="#/" onClick={(e) => { e.preventDefault(); onRetry(); }} style={{ fontSize: "var(--fs-1)" }}>
          Retry scan
        </a>
      </div>
    );
  }
  const response = run.response!;
  if (response.terminalState === "NOT_CONTRACT") {
    return (
      <div className="panel">
        <div className="label" style={{ marginBottom: "var(--s2)" }}>not a contract</div>
        <p className="muted" style={{ fontSize: "var(--fs-1)", margin: "0 0 var(--s2)" }}>
          {run.addr} has no code — a wallet address, not a token. Nothing to scan.
        </p>
        <a href="#/" style={{ fontSize: "var(--fs-1)" }}>Scan a different address</a>
      </div>
    );
  }
  if (response.terminalState === "RPC_RETRYABLE") {
    return (
      <div className="panel">
        <div className="label" style={{ marginBottom: "var(--s2)" }}>rpc unreachable</div>
        <p className="muted" style={{ fontSize: "var(--fs-1)", margin: "0 0 var(--s2)" }}>
          Chain {response.chainId} RPC timed out mid-probe. No partial verdicts are shown.
        </p>
        <a href="#/" onClick={(e) => { e.preventDefault(); onRetry(); }} style={{ fontSize: "var(--fs-1)" }}>
          Retry scan
        </a>
      </div>
    );
  }
  return <VerdictCard response={response} />;
}
