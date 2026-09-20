/**
 * Guarded-swap screen (plan task 3, J2) — states per screens/swap.html:
 * connect → wrong-network (switch to 4663 BEFORE quoting) → pre-flight with
 * the four guard checks → confirming → settled / refused (verbatim revert
 * reason) / nothing-sent (wallet rejection or gas failure). Funds-never-move
 * wording is load-bearing (journeys.md:34).
 */
import { useEffect, useState } from "react";
import { ALL_CHAINS, VETTED_CHAIN } from "../lib/chains";
import {
  classifyExecuteError,
  getGuardProbeSource,
  guardReasonDescription,
  previewGuard,
  type GuardExecutionResult,
  type GuardProbeSource,
  type GuardRevertReason,
} from "../lib/guard";
import type { WalletAdapter } from "../lib/wallet";
import { explorerTx } from "../components/VerdictCard";
import { shortAddr } from "../lib/format";
import { KNOWN_TOKENS, MOCK_USDG_ADDR } from "../lib/mockData";

const ADDR_RE = /^0x[0-9a-fA-F]{40}$/;

type Phase =
  | { s: "form" }
  | { s: "confirming" }
  | { s: "settled"; result: Extract<GuardExecutionResult, { kind: "settled" }> }
  | { s: "refused"; reason: GuardRevertReason | null; txHash?: string }
  | { s: "nothing-sent"; kind: "rejected" | "gas" };

// Lazy module singleton — a per-render default would re-fire the preview
// effect every cycle (infinite loop in the browser; found by e2e).
let defaultProbeSource: GuardProbeSource | null = null;
const probeSourceDefault = (): GuardProbeSource => (defaultProbeSource ??= getGuardProbeSource());

export function SwapPage({
  wallet,
  probeSource = probeSourceDefault(),
}: {
  wallet: WalletAdapter;
  probeSource?: GuardProbeSource;
}) {
  const [phase, setPhase] = useState<Phase>({ s: "form" });
  const [amountIn, setAmountIn] = useState("1,250");
  const [tokenIn, setTokenIn] = useState<string>(MOCK_USDG_ADDR);
  const [tokenOut, setTokenOut] = useState("");
  const [preview, setPreview] = useState<Awaited<ReturnType<typeof previewGuard>> | null>(null);

  const tokenOutValid = ADDR_RE.test(tokenOut.trim());

  useEffect(() => {
    if (!tokenOutValid || wallet.status !== "connected") {
      setPreview(null);
      return;
    }
    let alive = true;
    void previewGuard(probeSource, { token: tokenOut.trim(), buyer: wallet.address ?? "" }).then((p) => {
      if (alive) setPreview(p);
    });
    return () => {
      alive = false;
    };
  }, [tokenOut, tokenOutValid, wallet.status, wallet.address, probeSource]);

  async function execute() {
    setPhase({ s: "confirming" });
    try {
      const result = await wallet.execute({
        tokenIn: tokenIn.trim(),
        tokenOut: tokenOut.trim(),
        amountIn,
        buyer: wallet.address ?? "",
      });
      if (result.kind === "settled") setPhase({ s: "settled", result });
      else setPhase({ s: "refused", reason: result.reason, txHash: result.txHash });
    } catch (e) {
      const failure = classifyExecuteError(e);
      setPhase(
        failure.kind === "reverted"
          ? { s: "refused", reason: failure.reason }
          : { s: "nothing-sent", kind: failure.kind },
      );
    }
  }

  // --- connect states -------------------------------------------------------
  if (wallet.status !== "connected") {
    const zeroConnectors = wallet.connectorNames.length === 0;
    return (
      <div className="panel">
        <h1 style={{ fontSize: "var(--fs-4)", margin: "0 0 var(--s3)" }}>Guarded swap</h1>
        <p className="muted" style={{ fontSize: "var(--fs-1)" }}>
          This swap settles only tokens with a live verification record. At execution the guard re-runs
          every check on-chain and reverts on any deterministic red flag — your funds never move on a
          revert.
        </p>
        <button
          className="btn ghost"
          disabled={zeroConnectors || wallet.status === "connecting"}
          onClick={() => void wallet.connect()}
        >
          {wallet.status === "connecting" ? "Connecting…" : "Connect wallet"}
        </button>
        {zeroConnectors ? (
          <p className="faint" style={{ fontSize: "var(--fs-1)", marginBottom: 0 }} role="alert">
            No wallet connector is configured. Set <span className="mono">VITE_WALLETCONNECT_PROJECT_ID</span>{" "}
            (a WalletConnect Cloud projectId — the project's one outstanding operator action) to enable
            wallet connect. Scanning and the registry work without a wallet.
          </p>
        ) : null}
      </div>
    );
  }

  // --- wrong network: before any quoting (journeys.md:34) -------------------
  // VETTED_CHAIN is 4663 in the product; VITE_CHAIN_ID re-points the bundle at
  // a scratch chain for the e2e live pass (step 8) — default copy unchanged.
  if (wallet.chainId !== VETTED_CHAIN.id) {
    const name = ALL_CHAINS.find((c) => c.id === wallet.chainId)?.name ?? `chain ${wallet.chainId ?? "?"}`;
    const vettedLabel = `${VETTED_CHAIN.name} · ${VETTED_CHAIN.id}`;
    return (
      <div className="panel">
        <h1 style={{ fontSize: "var(--fs-4)", margin: "0 0 var(--s3)" }}>Guarded swap</h1>
        <p className="muted" style={{ fontSize: "var(--fs-1)", margin: "0 0 var(--s2)" }}>
          Wallet is connected to <strong>{name}</strong> — switch to {vettedLabel} to quote.
          Stock-token verdicts and the Canonical Registry live on {VETTED_CHAIN.id}; funds never move on a revert.
        </p>
        <button className="btn ghost" onClick={() => void wallet.switchToVetted()}>
          {`Switch to ${vettedLabel}`}
        </button>
      </div>
    );
  }

  const symbolOf = (addr: string) =>
    KNOWN_TOKENS.find((t) => t.address.toLowerCase() === addr.toLowerCase())?.symbol ?? shortAddr(addr);

  // --- post-execution states ------------------------------------------------
  if (phase.s === "confirming") {
    return (
      <div className="panel">
        <h1 style={{ fontSize: "var(--fs-4)", margin: "0 0 var(--s3)" }}>Confirming…</h1>
        <div
          className="mono"
          style={{
            background: "var(--bg-inset)",
            border: "1px solid var(--border)",
            borderRadius: "var(--radius)",
            padding: "var(--s3)",
            margin: "0 0 var(--s3)",
            display: "grid",
            gap: "var(--s2)",
            fontSize: "var(--fs-1)",
          }}
        >
          <div>
            <span style={{ color: "var(--accent)" }}>●</span> tx broadcast — waiting for receipt
          </div>
          <div className="faint">
            the guard re-runs every check inside this transaction — a red flag reverts it atomically
          </div>
          <div className="faint">outcome lands here the moment the receipt does</div>
        </div>
        <p className="faint mono" style={{ fontSize: "var(--fs-0)", margin: 0 }}>
          {`${VETTED_CHAIN.name} · ${VETTED_CHAIN.id}`}
        </p>
      </div>
    );
  }

  if (phase.s === "settled") {
    const { result } = phase;
    return (
      <div className="panel" style={{ borderColor: "var(--verified-border)" }}>
        <h1 style={{ fontSize: "var(--fs-4)", margin: "0 0 var(--s3)", color: "var(--verified)" }}>Settled</h1>
        <div className="banner verified mono" style={{ marginBottom: "var(--s3)" }}>
          All guard checks passed at execution
        </div>
        <p className="muted" style={{ fontSize: "var(--fs-1)", margin: "0 0 var(--s2)" }}>
          {amountIn} {symbolOf(tokenIn)} → <strong>{result.amountOut} {symbolOf(tokenOut)}</strong>{" "}
          delivered to your wallet. Guard receipt: record live, not paused, buyer clear, implementation
          matched — all re-verified in the same transaction that settled.
        </p>
        <p className="faint mono" style={{ fontSize: "var(--fs-0)", margin: 0 }}>
          tx {shortAddr(result.txHash)} · gas {result.gasUsed} incl. probes ·{" "}
          <a href={explorerTx(result.txHash)} target="_blank" rel="noreferrer">guard receipt</a>
        </p>
      </div>
    );
  }

  if (phase.s === "refused") {
    const revokedBeat = phase.reason === "GUARD_RECORD_REVOKED";
    return (
      <div className="panel" style={{ borderColor: "var(--risk-border)" }}>
        <h1 style={{ fontSize: "var(--fs-4)", margin: "0 0 var(--s3)", color: "var(--risk)" }}>
          {revokedBeat ? "Swap refused — verification revoked" : "Swap refused at execution"}
        </h1>
        <div className="banner risk mono" style={{ marginBottom: "var(--s3)" }}>
          {phase.reason ?? "GUARD_REVERT"}: {phase.reason ? guardReasonDescription(phase.reason) : "the guard reverted on-chain"}
        </div>
        <p className="muted" style={{ fontSize: "var(--fs-1)", margin: "0 0 var(--s2)" }}>
          {revokedBeat
            ? "This token was verified before. Its implementation changed, the registry revoked the stale record, and the guard refused the trade at execution time — the moment that matters. A static docs page cannot do this."
            : "The guard reverted on-chain."}{" "}
          <strong>Your {amountIn} {symbolOf(tokenIn)} never left your wallet.</strong>
        </p>
        {phase.txHash ? (
          <p className="faint mono" style={{ fontSize: "var(--fs-0)", margin: 0 }}>
            tx {shortAddr(phase.txHash)} ·{" "}
            <a href={explorerTx(phase.txHash)} target="_blank" rel="noreferrer">view on explorer</a>
          </p>
        ) : null}
      </div>
    );
  }

  if (phase.s === "nothing-sent") {
    return (
      <div className="panel">
        <h1 style={{ fontSize: "var(--fs-4)", margin: "0 0 var(--s3)" }}>Nothing was sent</h1>
        <p className="muted" style={{ fontSize: "var(--fs-1)", margin: "0 0 var(--s2)" }}>
          <strong>{phase.kind === "rejected" ? "Rejected in wallet" : "Gas too low"}</strong> —{" "}
          {phase.kind === "rejected"
            ? "the signature was declined, so no transaction was broadcast."
            : "the transaction never lands."}{" "}
          Either way, your {amountIn} {symbolOf(tokenIn)} never left your wallet.
        </p>
        <button className="btn danger" onClick={() => setPhase({ s: "form" })}>Swap guarded</button>
        <span className="faint" style={{ fontSize: "var(--fs-1)", marginLeft: "var(--s3)" }}>
          Retry when ready — the guard re-runs every check at execution.
        </span>
      </div>
    );
  }

  // --- pre-flight form (screens/swap.html) ----------------------------------
  return (
    <div className="panel">
      <h1 style={{ fontSize: "var(--fs-4)", margin: "0 0 var(--s3)" }}>Guarded swap</h1>
      <div className="field" style={{ marginBottom: "var(--s3)" }}>
        <span className="label" style={{ display: "block", marginBottom: "var(--s1)" }}>you pay</span>
        <div className="swaprow" style={{ display: "flex", gap: "var(--s3)", alignItems: "center" }}>
          <input className="input" type="text" value={amountIn} onChange={(e) => setAmountIn(e.target.value)} aria-label="amount in" />
          <span className="mono muted">{symbolOf(tokenIn)}</span>
        </div>
      </div>
      <div className="field" style={{ marginBottom: "var(--s3)" }}>
        <span className="label" style={{ display: "block", marginBottom: "var(--s1)" }}>pay token address</span>
        <input className="input" type="text" value={tokenIn} onChange={(e) => setTokenIn(e.target.value)} spellCheck={false} aria-label="pay token address" />
      </div>
      <div className="field" style={{ marginBottom: "var(--s3)" }}>
        <span className="label" style={{ display: "block", marginBottom: "var(--s1)" }}>
          you receive — any token address accepted
        </span>
        <input
          className="input"
          type="text"
          value={tokenOut}
          onChange={(e) => setTokenOut(e.target.value)}
          placeholder="0x… token address"
          spellCheck={false}
          aria-label="receive token address"
        />
      </div>

      <div className="panel" style={{ background: "var(--bg-inset)", margin: "var(--s4) 0" }}>
        <div className="label" style={{ marginBottom: "var(--s2)" }}>
          guard checks — these run again on-chain at execution
        </div>
        {preview === null ? (
          <p className="faint" style={{ fontSize: "var(--fs-1)", margin: 0 }}>
            Enter a receive token address to preview the guard checks.
          </p>
        ) : (
          preview.rows.map((row) => (
            <div className="check-row" key={row.label}>
              <span className="mono" style={{ color: row.ok ? "var(--verified)" : "var(--risk)" }}>
                {row.ok ? "✓" : "✗"}
              </span>
              <span>{row.label}</span>
              <span className="mono" style={row.ok ? undefined : { color: "var(--risk)" }}>{row.detail}</span>
            </div>
          ))
        )}
        {preview?.firstRevert ? (
          <p className="mono" style={{ color: "var(--risk)", fontSize: "var(--fs-0)", margin: "var(--s2) 0 0" }} role="alert">
            the guard will refuse this swap on-chain: {preview.firstRevert} — preview is advisory; the guard decides
          </p>
        ) : null}
      </div>

      <button className="btn danger" disabled={!tokenOutValid} onClick={() => void execute()}>
        Swap guarded
      </button>
      <span className="faint" style={{ fontSize: "var(--fs-1)", marginLeft: "var(--s3)" }}>
        Probe suite ≤ 200,000 gas — included in your trade.
      </span>
    </div>
  );
}
