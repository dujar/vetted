/**
 * The verdict card (plan task 2) — four verdict states plus the power report,
 * per screens/scan.html. Every power row carries its evidence link (spec.md:30);
 * UNVERIFIED shows the depth-boundary notice (spec.md:29); REVOKED links the
 * revocation tx when the wire carries one (verify.md loose end 6).
 */
import type { ScanResponse } from "vetted-shared";

import { VERDICT_VARIANT } from "./verdicts";
import { decodeBytes32, formatDate, shortAddr } from "../lib/format";
import { KNOWN_TOKENS } from "../lib/mockData";

const SEVERITY_COLOR = {
  risk: "var(--risk)",
  advisory: "var(--advisory)",
  verified: "var(--verified)",
} as const;

const BADGE_CLASS = { verified: "verified", risk: "impostor", advisory: "unverified" } as const;

const DOCS_CANONICAL_URL = "https://docs.robinhood.com/chain/contracts";

function knownName(addr: string): string | null {
  const hit = KNOWN_TOKENS.find((t) => t.address.toLowerCase() === addr.toLowerCase());
  return hit ? hit.name : null;
}

const riskBorder = { borderColor: "var(--risk-border)" };

export function VerdictCard({ response, swapHref = "#/swap" }: { response: ScanResponse; swapHref?: string }) {
  const { verdict } = response;
  const name = knownName(response.addr);

  const header = (
    <div style={{ display: "flex", alignItems: "center", gap: "var(--s3)", marginBottom: "var(--s3)" }}>
      {verdict !== null ? (
        <span
          className={`badge ${BADGE_CLASS[VERDICT_VARIANT[verdict]]}`}
          style={{ fontSize: "var(--fs-3)", padding: "var(--s1) var(--s3)" }}
        >
          {verdict}
        </span>
      ) : null}
      <div>
        <Title response={response} name={name} />
        <div className="mono faint">{response.addr}</div>
      </div>
    </div>
  );

  if (verdict === "VERIFIED") {
    return (
      <section className="panel">
        {header}
        <div className="label" style={{ marginBottom: "var(--s1)" }}>what this contract can do to your balance</div>
        {response.powerReport.map((row) => (
          <div className="report-row" key={row.check}>
            <span>{row.check}</span>
            <span className="mono" style={{ color: SEVERITY_COLOR[row.severity] }}>{row.result}</span>
            <a href={row.evidenceUrl} target="_blank" rel="noreferrer">evidence</a>
          </div>
        ))}
        <p className="faint" style={{ fontSize: "var(--fs-1)", marginBottom: 0 }}>
          Verified ≠ safe. This is the genuine token, and it still carries the powers above.{" "}
          <a href={swapHref}>Swap guarded →</a>
        </p>
      </section>
    );
  }

  if (verdict === "IMPOSTOR") {
    return (
      <section className="panel" style={riskBorder}>
        {header}
        <table className="table">
          <thead>
            <tr>
              <th></th>
              <th>check</th>
              <th style={{ color: "var(--risk)" }}>result</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {response.powerReport.map((row) => (
              <tr key={row.check}>
                <td className="label">{row.severity}</td>
                <td>{row.check}</td>
                <td className="mono" style={{ color: "var(--risk)" }}>{row.result}</td>
                <td><a href={row.evidenceUrl} target="_blank" rel="noreferrer">evidence</a></td>
              </tr>
            ))}
          </tbody>
        </table>
        <p className="faint" style={{ fontSize: "var(--fs-1)", marginBottom: 0 }}>
          Positive evidence only — mimicry of a canonical token at a different address (spec discipline 1).{" "}
          <a href={swapHref}>Watch the guard refuse it on-chain →</a>
        </p>
      </section>
    );
  }

  if (verdict === "UNVERIFIED") {
    return (
      <section className="panel">
        {header}
        {response.powerReport.map((row) => (
          <div className="report-row" key={row.check}>
            <span>{row.check}</span>
            <span className="mono" style={{ color: SEVERITY_COLOR[row.severity] }}>{row.result}</span>
            <a href={row.evidenceUrl} target="_blank" rel="noreferrer">evidence</a>
          </div>
        ))}
        {response.notice === null ? (
          <div className="banner advisory" style={{ marginTop: "var(--s3)" }}>
            Depth boundary: full verdicts cover the Robinhood stock-token pattern (signature match). This
            contract is off-pattern, so every line above is a structural heuristic — advisory only, never a
            verdict.
          </div>
        ) : null}
      </section>
    );
  }

  if (verdict === "REVOKED") {
    return (
      <section className="panel" style={riskBorder}>
        {header}
        <div className="banner risk mono" style={{ marginBottom: "var(--s3)" }}>
          {response.record ? `beacon implementation changed ${formatDate(response.record.revokedAt)}` : "record revoked"}
          {" · record revoked"}
          {response.record?.reason ? ` · ${decodeBytes32(response.record.reason)}` : ""}
          {" · revocation tx "}
          {response.revocationTx ? (
            <a href={explorerTx(response.revocationTx)} target="_blank" rel="noreferrer">{shortAddr(response.revocationTx)}</a>
          ) : (
            <span className="faint">not indexed yet — linked from the registry when step 3 emits the Revoked event</span>
          )}
        </div>
        <p className="faint" style={{ fontSize: "var(--fs-1)", marginBottom: 0 }}>
          The registry record is stale by definition — the guard refuses swaps against it.{" "}
          <a href={swapHref}>See the refusal →</a>
        </p>
      </section>
    );
  }

  return null;
}

function Title({ response, name }: { response: ScanResponse; name: string | null }) {
  if (response.verdict === "VERIFIED") {
    return (
      <div style={{ fontWeight: 700 }}>
        {name ?? "Token"} <span className="muted">· Robinhood Stock Token</span>
      </div>
    );
  }
  if (response.verdict === "IMPOSTOR") {
    return <div style={{ fontWeight: 700, color: "var(--risk)" }}>This is not a Robinhood Stock Token.</div>;
  }
  if (response.verdict === "REVOKED") {
    return (
      <div style={{ fontWeight: 700, color: "var(--risk)" }}>
        Verification revoked — this token changed since it was verified.
      </div>
    );
  }
  return null;
}

export function explorerTx(txHash: string): string {
  return `https://robinhoodchain.blockscout.com/tx/${txHash}`;
}

export { DOCS_CANONICAL_URL };
