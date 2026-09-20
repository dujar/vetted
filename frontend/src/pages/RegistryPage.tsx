/**
 * Canonical Registry screen (plan task 4, J3) — per screens/registry.html:
 * known-token table (REVOKED rows red with reason + drill-in evidence),
 * criteria panel (registrar, criteria.md link, on-chain read interface),
 * loading skeleton, empty + degraded states, roadmap strip.
 */
import { Fragment, useEffect, useState } from "react";
import { REGISTRY_ABI_SIGNATURES, SELECTORS } from "vetted-shared";

import { Skeleton } from "../components/Skeleton";
import { explorerTx } from "../components/VerdictCard";
import { VERDICT_VARIANT, type Verdict } from "../components/verdicts";
import { getRegistrySource, type RegistryRow, type RegistrySource } from "../lib/registry";
import { VETTED_CHAIN } from "../lib/chains";
import { decodeBytes32, formatDate, shortAddr } from "../lib/format";

/** Path reserved by step 7 (its plan links the registry page to exactly this). */
const CRITERIA_URL = "https://github.com/dujar/vetted/blob/main/docs/criteria.md";

type Phase =
  | { s: "loading" }
  | { s: "ready"; rows: RegistryRow[] }
  | { s: "degraded"; rows: RegistryRow[] };

// Lazy module singleton — a per-render default would re-fire the load effect
// every cycle (infinite loop in the browser; found by e2e).
let defaultSource: RegistrySource | null = null;
const sourceDefault = (): RegistrySource => (defaultSource ??= getRegistrySource());

export function RegistryPage({ source = sourceDefault() }: { source?: RegistrySource }) {
  const [phase, setPhase] = useState<Phase>({ s: "loading" });
  const [openRow, setOpenRow] = useState<string | null>(null);

  useEffect(() => {
    let alive = true;
    source
      .rows(VETTED_CHAIN.id)
      .then((rows) => {
        if (alive) setPhase({ s: "ready", rows });
      })
      .catch(() => {
        if (alive) setPhase({ s: "degraded", rows: [] });
      });
    return () => {
      alive = false;
    };
  }, [source]);

  const badgeClass = { verified: "verified", risk: "impostor", advisory: "unverified" } as const;
  const badgeFor = (status: "VERIFIED" | "REVOKED") =>
    badgeClass[VERDICT_VARIANT[status as Verdict]];

  const registrar =
    phase.s !== "loading" ? phase.rows.find((r) => r.record)?.record?.registrar : undefined;

  return (
    <>
      <h1 style={{ fontSize: "var(--fs-4)", margin: "0 0 var(--s2)" }}>Canonical Registry</h1>
      <p className="muted" style={{ fontSize: "var(--fs-1)", marginTop: 0 }}>
        The machine-readable, revocable, upgrade-aware replacement for the issuer's docs page. Ground
        truth is Robinhood's own canonical list, fetched live — this registry never contradicts the
        issuer on a genuine token.
      </p>

      {phase.s === "loading" ? (
        <div className="panel">
          <Skeleton rows={4} label="loading registry records" />
          <p className="faint" style={{ fontSize: "var(--fs-1)", marginBottom: 0 }}>
            Reading verification records from the Canonical Registry…
          </p>
        </div>
      ) : phase.s === "degraded" ? (
        <div className="panel">
          <div className="banner advisory" style={{ marginBottom: "var(--s3)" }} role="alert">
            Issuer canonical list unreachable — new verifications are paused. Existing records stay
            readable on-chain; nothing is modified without ground truth.
          </div>
          <p className="faint" style={{ fontSize: "var(--fs-1)", margin: 0 }}>
            No records could be read.{" "}
            <a
              href="#/registry"
              onClick={(e) => {
                e.preventDefault();
                setPhase({ s: "loading" });
                void source.rows(VETTED_CHAIN.id).then((rows) => setPhase({ s: "ready", rows }));
              }}
            >
              Retry
            </a>
          </p>
        </div>
      ) : phase.rows.length === 0 ? (
        <div className="panel" style={{ textAlign: "center", padding: "var(--s7)" }}>
          <p className="muted" style={{ margin: "0 0 var(--s3)" }}>No verification records yet on this chain.</p>
          <p className="faint" style={{ fontSize: "var(--fs-1)", margin: 0 }}>
            Records appear once the registrar verifies the first canonical token.{" "}
            <a href="#/">Scan an address in the meantime →</a>
          </p>
        </div>
      ) : (
        <div style={{ display: "grid", gridTemplateColumns: "1.7fr 1fr", gap: "var(--s4)", alignItems: "start" }}>
          <div className="panel">
            <table className="table">
              <thead>
                <tr>
                  <th>Token</th>
                  <th>Symbol</th>
                  <th>Status</th>
                  <th>Implementation</th>
                  <th>Verified</th>
                  <th>Registrar</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {phase.rows.map((row) => {
                  const record = row.record;
                  const revoked = record !== null && record.status === "REVOKED";
                  return (
                    <Fragment key={row.token}>
                      <tr>
                        <td>{row.name}</td>
                        <td className="mono">{row.symbol}</td>
                        <td>
                          {record ? (
                            <>
                              <span className={`badge ${badgeFor(record.status)}`}>{record.status}</span>
                              {revoked ? (
                                <div className="faint mono" style={{ fontSize: "var(--fs-0)" }}>
                                  {decodeBytes32(record.reason)} {formatDate(record.revokedAt)} — record stale ·{" "}
                                  <a
                                    href={`#${row.token}`}
                                    onClick={(e) => {
                                      e.preventDefault();
                                      setOpenRow(openRow === row.token ? null : row.token);
                                    }}
                                  >
                                    evidence
                                  </a>
                                </div>
                              ) : null}
                            </>
                          ) : (
                            <span className="faint mono">no record</span>
                          )}
                        </td>
                        <td className="mono">{record ? shortAddr(record.impl) : "—"}</td>
                        <td className="mono">{record ? formatDate(record.verifiedAt) : "—"}</td>
                        <td className="mono">{record ? shortAddr(record.registrar) : "—"}</td>
                        <td>
                          <a href={`#/?addr=${row.token}`}>scan</a>
                        </td>
                      </tr>
                      {revoked && record && openRow === row.token ? (
                        <tr>
                          <td colSpan={7} className="mono faint" style={{ fontSize: "var(--fs-0)" }}>
                            revoked {formatDate(record.revokedAt)} · reason: {decodeBytes32(record.reason)} ·{" "}
                            revocation tx{" "}
                            {row.revocationTx ? (
                              <a href={explorerTx(row.revocationTx)} target="_blank" rel="noreferrer">
                                {shortAddr(row.revocationTx)}
                              </a>
                            ) : (
                              "not indexed yet — linked when step 3 emits the Revoked event"
                            )}
                          </td>
                        </tr>
                      ) : null}
                    </Fragment>
                  );
                })}
              </tbody>
            </table>
          </div>

          <div>
            <div className="panel" style={{ marginBottom: "var(--s3)" }}>
              <div className="label" style={{ marginBottom: "var(--s2)" }}>who writes</div>
              <p className="muted" style={{ fontSize: "var(--fs-1)", margin: "0 0 var(--s2)" }}>
                A single registrar key held by this project's backend —{" "}
                <span className="mono">{registrar ? `registrar: ${shortAddr(registrar)}` : "registrar: —"}</span>.{" "}
                <a href={CRITERIA_URL} target="_blank" rel="noreferrer">Verification criteria, published in the repo.</a>
              </p>
              <p className="faint" style={{ fontSize: "var(--fs-1)", margin: 0 }}>
                Records are <strong>revocable</strong>: a beacon upgrade stales a verification, and
                revocation is the feature — the guard refuses a stale record at execution time.
              </p>
            </div>
            <div className="panel">
              <div className="label" style={{ marginBottom: "var(--s2)" }}>consume it from your contract</div>
              <div
                className="mono"
                style={{
                  background: "var(--bg-inset)",
                  border: "1px solid var(--border)",
                  borderRadius: "var(--radius)",
                  padding: "var(--s3)",
                  fontSize: "var(--fs-1)",
                  overflowX: "auto",
                }}
              >
                <div>{REGISTRY_ABI_SIGNATURES[0]}</div>
                <div>{`selector ${SELECTORS.getRecord}`}</div>
              </div>
              <p className="faint" style={{ fontSize: "var(--fs-1)" }}>
                Wallets, aggregators and frontends can gate on this record on-chain — the registry is a
                primitive, not just a page.
              </p>
              <div style={{ borderTop: "1px solid var(--border)", marginTop: "var(--s3)", paddingTop: "var(--s2)" }}>
                <span className="label">roadmap</span>{" "}
                <span className="faint mono" style={{ fontSize: "var(--fs-0)", marginLeft: "var(--s2)" }}>
                  watchlists · alerts · partner integrations
                </span>
              </div>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
