//! The rule engine — PURE: `ScanInputs` -> `ScanResponse`. Discipline rules
//! 1–4 verbatim (spec.md:25–30); the golden fixtures in
//! packages/shared/fixtures/verdict are the contract (tests/golden_fixtures.rs
//! reproduces each one from labeled inputs, byte-exact).
//!
//! Depth boundary (rule 3 + plan Revised note 3): full verdicts require the
//! structural signature match assembled in fingerprint.rs; everything else is
//! UNVERIFIED + advisory heuristics, labeled as such. Missing evidence degrades
//! to UNVERIFIED, never guesses (rule 1).

use vetted_shared::types::{
    PowerReportRow, RegistryRecord, ScanResponse, Severity, TerminalState, Verdict,
};

use crate::hexutil::short_addr;

pub const CHAIN_MAINNET: u64 = 4663;
pub const CHAIN_TESTNET: u64 = 46630;
pub const CHAIN_ARB_SEPOLIA: u64 = 421614;

/// The issuer canonical list — fetched live at scan time (rule 1); absorbed
/// from the proven spike worker into this one (plan Revised note 2).
pub const CANONICAL_ASSETS_URL: &str = "https://api.robinhood.com/rhj/assets";

pub const NOTICE_NON_4663: &str =
    "Stock-token verdicts exist only on Robinhood Chain (4663) — this scan ran read-only.";

/// What the EIP-1967/beacon resolution found (fingerprint.rs output).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Structure {
    /// Beacon-proxy layout observed (beacon slot set + impl slot empty).
    pub beacon_proxy: bool,
    /// The resolved beacon (forwarder-bytecode extraction, guard-pinned path).
    pub beacon: Option<String>,
    /// The beacon's current implementation().
    pub impl_addr: Option<String>,
    /// The full calibrated signature matched (depth boundary — rule 3).
    pub signature_match: bool,
}

/// Probe outcomes at the calibrated targets. `true` = the selector ANSWERED
/// (the mechanism exists on-chain); `false` = it reverted (mechanism absent).
/// `probes: None` = probes not run (degraded/terminal scenarios).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Probes {
    /// `paused()` on the token proxy.
    pub paused: bool,
    /// `isBlocked(buyer)` on the resolved beacon (blocklist state lives there).
    pub blocklist: bool,
}

/// Canonical ground-truth evidence assembled from the issuer list + metadata
/// reads (rule 1's positive evidence).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalEvidence {
    /// The scanned address IS in the issuer's canonical list for this chain.
    pub listed: bool,
    /// When not listed but name+symbol equal a canonical asset: that asset's
    /// identity (the mimicry evidence — rule 1).
    pub mimics_name: Option<String>,
    pub mimics_symbol: Option<String>,
    /// on-chain uid() vs the issuer row's id: Some(false) = mismatch.
    pub uid_match: Option<bool>,
    /// `(live_short, canonical_short)` when live impl codehash differs from
    /// the canonical implementation's (IMPOSTOR fixture row 2).
    pub impl_differs: Option<(String, String)>,
}

/// Everything the engine needs — the async orchestrator assembles this; the
/// engine itself never awaits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanInputs {
    pub chain_id: u64,
    pub addr: String,
    pub explorer: &'static str,
    pub has_code: bool,
    pub structure: Option<Structure>,
    pub probes: Option<Probes>,
    pub canonical: Option<CanonicalEvidence>,
    pub record: Option<RegistryRecord>,
    /// Issuer canonical list unreachable (journeys.md:20): ALL verdicts drop
    /// to UNVERIFIED. Only settable on 4663 — other chains never fetch it.
    pub degraded: bool,
    /// Revocation tx (journeys.md:14). Live source: the registry's `Revoke`
    /// event logs (topic0 pinned in hexutil::events, byte-equal to step-3's
    /// events.ts); a failed read degrades to null — the frontend's existing
    /// "not indexed yet" rendering.
    pub revocation_tx: Option<String>,
}

impl ScanInputs {
    pub fn new(chain_id: u64, addr: &str) -> Self {
        Self {
            chain_id,
            addr: addr.to_string(),
            explorer: explorer_for(chain_id),
            has_code: true,
            structure: None,
            probes: None,
            canonical: None,
            record: None,
            degraded: false,
            revocation_tx: None,
        }
    }
}

pub fn explorer_for(chain_id: u64) -> &'static str {
    match chain_id {
        CHAIN_MAINNET => "https://robinhoodchain.blockscout.com",
        CHAIN_TESTNET => "https://explorer.testnet.chain.robinhood.com",
        CHAIN_ARB_SEPOLIA => "https://sepolia.arbiscan.io",
        _ => "https://robinhoodchain.blockscout.com",
    }
}

fn code_url(explorer: &str, addr: &str) -> String {
    format!("{explorer}/address/{addr}?tab=code")
}

fn read_url(explorer: &str, addr: &str) -> String {
    format!("{explorer}/address/{addr}/read")
}

fn row(check: String, result: &str, severity: Severity, evidence_url: String) -> PowerReportRow {
    PowerReportRow {
        check,
        result: result.to_string(),
        severity,
        evidence_url,
    }
}

/// RPC failure — retryable terminal state, nothing guessed (journeys.md).
pub fn rpc_retryable(chain_id: u64, addr: &str) -> ScanResponse {
    ScanResponse {
        chain_id,
        addr: addr.to_string(),
        verdict: None,
        terminal_state: Some(TerminalState::RpcRetryable),
        degraded: false,
        notice: notice_for(chain_id),
        power_report: vec![],
        record: None,
        revocation_tx: None,
    }
}

fn notice_for(chain_id: u64) -> Option<String> {
    (chain_id != CHAIN_MAINNET).then(|| NOTICE_NON_4663.to_string())
}

/// The advisory heuristics row — its wording IS the depth-boundary disclosure
/// (rule 3). Result reflects what structure was visible.
fn heuristics_row(rows: &mut Vec<PowerReportRow>, inputs: &ScanInputs) {
    let signature_matched = inputs
        .structure
        .as_ref()
        .map(|s| s.signature_match)
        .unwrap_or(false);
    let (check, result) = if signature_matched {
        (
            "Pattern match without ground truth".to_string(),
            "ROBINHOOD_PATTERN",
        )
    } else if inputs.structure.as_ref().map(|s| s.beacon_proxy).unwrap_or(false) {
        // The fixture-pinned wording (UNVERIFIED.json).
        (
            "Structural heuristics (advisory — no signature match)".to_string(),
            "BEACON_LAYOUT",
        )
    } else {
        (
            "Structural heuristics (advisory — no signature match)".to_string(),
            "GENERIC_CONTRACT",
        )
    };
    rows.push(row(check, result, Severity::Advisory, code_url(inputs.explorer, &inputs.addr)));
}

pub fn evaluate(inputs: &ScanInputs) -> ScanResponse {
    // journeys.md:18 — no code at the address: NOT_CONTRACT, no verdict attempted.
    if !inputs.has_code {
        return ScanResponse {
            chain_id: inputs.chain_id,
            addr: inputs.addr.clone(),
            verdict: None,
            terminal_state: Some(TerminalState::NotContract),
            degraded: false,
            notice: notice_for(inputs.chain_id),
            power_report: vec![],
            record: inputs.record.clone(),
            revocation_tx: None,
        };
    }

    let mut rows: Vec<PowerReportRow> = vec![];

    // Power-report probe rows (rule 4 — every row carries evidence). The
    // blocklist row's evidence is the BEACON probe result (plan Revised note 1);
    // pause is probed on the proxy. PRESENT = the hidden power exists on-chain:
    // a VERIFIED token can still carry red power flags (legitimacy ≠ safety).
    if let Some(probes) = &inputs.probes {
        let beacon = inputs
            .structure
            .as_ref()
            .and_then(|s| s.beacon.as_ref())
            .map(|b| read_url(inputs.explorer, b));
        rows.push(row(
            "Buyer blocklist in modifier".to_string(),
            if probes.blocklist { "PRESENT" } else { "ABSENT" },
            if probes.blocklist { Severity::Risk } else { Severity::Verified },
            beacon.unwrap_or_else(|| code_url(inputs.explorer, &inputs.addr)),
        ));
        rows.push(row(
            "Global pause control".to_string(),
            if probes.paused { "PRESENT" } else { "ABSENT" },
            if probes.paused { Severity::Risk } else { Severity::Verified },
            code_url(inputs.explorer, &inputs.addr),
        ));
        // Upgradeability is disclosed only when the scan fully probed the token
        // (the degraded fixture scenario reports heuristics only).
        if inputs.structure.as_ref().map(|s| s.beacon_proxy).unwrap_or(false) {
            rows.push(row(
                "Upgradeability".to_string(),
                "BEACON_PROXY",
                Severity::Advisory,
                read_url(inputs.explorer, &inputs.addr),
            ));
        }
    }

    // The drift check: live implementation vs the record's pointer.
    let drifted = |record: &RegistryRecord| -> bool {
        inputs
            .structure
            .as_ref()
            .and_then(|s| s.impl_addr.as_ref())
            .map(|live| !live.eq_ignore_ascii_case(&record.impl_))
            .unwrap_or(false)
    };
    let drift_row = |rows: &mut Vec<PowerReportRow>, inputs: &ScanInputs| {
        rows.push(row(
            "Beacon implementation changed since verification".to_string(),
            "IMPL_DRIFT",
            Severity::Risk,
            read_url(inputs.explorer, &inputs.addr),
        ));
    };

    let verdict;
    let mut revocation_tx = None;

    if inputs.degraded {
        // journeys.md:20 — ground truth down: every verdict drops to UNVERIFIED.
        verdict = Verdict::Unverified;
        heuristics_row(&mut rows, inputs);
    } else if let Some(rec) = &inputs.record {
        match rec.status {
            vetted_shared::types::RecordStatus::Revoked => {
                verdict = Verdict::Revoked;
                revocation_tx = inputs.revocation_tx.clone();
                if drifted(rec) {
                    drift_row(&mut rows, inputs);
                }
            }
            vetted_shared::types::RecordStatus::Verified => {
                if drifted(rec) {
                    // The window between beacon upgrade and the registrar's
                    // auto-revoke (spec.md:35): the record is stale, the token
                    // is no longer verifiable — never report VERIFIED.
                    verdict = Verdict::Unverified;
                    drift_row(&mut rows, inputs);
                } else {
                    verdict = Verdict::Verified;
                }
            }
        }
    } else if inputs.chain_id != CHAIN_MAINNET {
        // journeys.md:19 — read-only scan; stock-token verdicts exist only on 4663.
        verdict = Verdict::Unverified;
        heuristics_row(&mut rows, inputs);
    } else {
        // No registry record (registry unconfigured — verify loose end 1 — or
        // none for this token): canonical-fetch-only rules.
        let signature_matched = inputs
            .structure
            .as_ref()
            .map(|s| s.signature_match)
            .unwrap_or(false);
        match inputs.canonical.as_ref() {
            Some(c) if c.listed && signature_matched && c.uid_match != Some(false) => {
                verdict = Verdict::Verified;
                rows.push(row(
                    "Issuer canonical list".to_string(),
                    "LISTED",
                    Severity::Verified,
                    CANONICAL_ASSETS_URL.to_string(),
                ));
            }
            Some(c) if !c.listed && signature_matched => {
                // Rule 1 positive evidence: canonical name+symbol at a
                // different address, with the signature match backing the read.
                verdict = Verdict::Impostor;
                rows.push(row(
                    format!(
                        "Metadata mimics canonical {} ({})",
                        c.mimics_name.as_deref().unwrap_or("?"),
                        c.mimics_symbol.as_deref().unwrap_or("?")
                    ),
                    "MIMICRY",
                    Severity::Risk,
                    CANONICAL_ASSETS_URL.to_string(),
                ));
                if let Some((live, canonical)) = &c.impl_differs {
                    rows.push(row(
                        "Implementation differs from canonical impl".to_string(),
                        &format!("{live} ≠ {canonical}"),
                        Severity::Risk,
                        read_url(inputs.explorer, &inputs.addr),
                    ));
                }
            }
            other => {
                verdict = Verdict::Unverified;
                if let Some(c) = other {
                    if !c.listed && !signature_matched {
                        // Mimicry seen on a contract outside the depth boundary:
                        // advisory only — never an IMPOSTOR verdict (rule 3).
                        rows.push(row(
                            format!(
                                "Metadata mimics canonical {} ({})",
                                c.mimics_name.as_deref().unwrap_or("?"),
                                c.mimics_symbol.as_deref().unwrap_or("?")
                            ),
                            "MIMICRY",
                            Severity::Advisory,
                            CANONICAL_ASSETS_URL.to_string(),
                        ));
                    }
                }
                heuristics_row(&mut rows, inputs);
            }
        }
    }

    ScanResponse {
        chain_id: inputs.chain_id,
        addr: inputs.addr.clone(),
        verdict: Some(verdict),
        terminal_state: None,
        degraded: inputs.degraded,
        notice: notice_for(inputs.chain_id),
        power_report: rows,
        record: inputs.record.clone(),
        revocation_tx,
    }
}

/// Fixture-formatting helper for the IMPOSTOR impl-differs row (`(live, canonical)` shorts).
pub fn impl_diff_pair(live: &str, canonical: &str) -> (String, String) {
    (short_addr(live), short_addr(canonical))
}

#[cfg(test)]
mod tests {
    use super::*;
    use vetted_shared::types::RecordStatus;

    fn verified_record(impl_addr: &str) -> RegistryRecord {
        RegistryRecord {
            status: RecordStatus::Verified,
            risk_flags: "0".into(),
            verified_at: 1788048000,
            impl_: impl_addr.into(),
            registrar: "0x5e1500000000000000000000000000000000aa11".into(),
            revoked_at: 0,
            reason: "0x0000000000000000000000000000000000000000000000000000000000000000".into(),
        }
    }

    #[test]
    fn not_contract_is_terminal() {
        let mut inputs = ScanInputs::new(CHAIN_MAINNET, "0xabc0000000000000000000000000000000000001");
        inputs.has_code = false;
        let out = evaluate(&inputs);
        assert_eq!(out.verdict, None);
        assert_eq!(out.terminal_state, Some(TerminalState::NotContract));
        assert!(out.power_report.is_empty());
    }

    #[test]
    fn degraded_forces_unverified() {
        let mut inputs = ScanInputs::new(CHAIN_MAINNET, "0xabc0000000000000000000000000000000000001");
        inputs.degraded = true;
        inputs.structure = Some(Structure {
            beacon_proxy: true,
            beacon: None,
            impl_addr: None,
            signature_match: true,
        });
        let out = evaluate(&inputs);
        assert_eq!(out.verdict, Some(Verdict::Unverified));
        assert!(out.degraded);
        // A signature-matching token under degradation discloses exactly that.
        assert_eq!(out.power_report.len(), 1);
        assert_eq!(out.power_report[0].check, "Pattern match without ground truth");
        assert_eq!(out.power_report[0].result, "ROBINHOOD_PATTERN");

        // Without the signature, the depth-boundary wording applies.
        inputs.structure = Some(Structure {
            beacon_proxy: true,
            beacon: None,
            impl_addr: None,
            signature_match: false,
        });
        let out = evaluate(&inputs);
        assert_eq!(
            out.power_report[0].check,
            "Structural heuristics (advisory — no signature match)"
        );
        assert_eq!(out.power_report[0].result, "BEACON_LAYOUT");
    }

    #[test]
    fn verified_record_with_drift_window_is_unverified() {
        let mut inputs = ScanInputs::new(CHAIN_MAINNET, "0xabc0000000000000000000000000000000000001");
        inputs.structure = Some(Structure {
            beacon_proxy: true,
            beacon: None,
            impl_addr: Some("0xc30d11111111111111111111111111111111e812".into()),
            signature_match: true,
        });
        inputs.record = Some(verified_record("0x77be0000000000000000000000000000000041af"));
        let out = evaluate(&inputs);
        assert_eq!(out.verdict, Some(Verdict::Unverified));
        assert_eq!(out.power_report[0].result, "IMPL_DRIFT");
    }

    #[test]
    fn revoked_record_links_tx_and_reports_drift() {
        let mut rec = verified_record("0x77be0000000000000000000000000000000041af");
        rec.status = RecordStatus::Revoked;
        rec.revoked_at = 1788652800;
        rec.reason = "0x626561636f6e20696d706c206368616e67656400000000000000000000000000".into();
        let mut inputs = ScanInputs::new(CHAIN_MAINNET, "0xabc0000000000000000000000000000000000001");
        inputs.structure = Some(Structure {
            beacon_proxy: true,
            beacon: None,
            impl_addr: Some("0xc30d11111111111111111111111111111111e812".into()),
            signature_match: true,
        });
        inputs.record = Some(rec);
        inputs.revocation_tx =
            Some("0xab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12".into());
        let out = evaluate(&inputs);
        assert_eq!(out.verdict, Some(Verdict::Revoked));
        assert_eq!(out.revocation_tx.as_deref(), inputs.revocation_tx.as_deref());
        assert_eq!(out.power_report.len(), 1);
        assert_eq!(out.power_report[0].result, "IMPL_DRIFT");
    }

    #[test]
    fn non_4663_never_gets_stock_token_verdicts() {
        let mut inputs = ScanInputs::new(CHAIN_ARB_SEPOLIA, "0xabc0000000000000000000000000000000000001");
        inputs.structure = Some(Structure {
            beacon_proxy: true,
            beacon: None,
            impl_addr: None,
            signature_match: true,
        });
        inputs.canonical = Some(CanonicalEvidence {
            listed: true,
            mimics_name: None,
            mimics_symbol: None,
            uid_match: Some(true),
            impl_differs: None,
        });
        let out = evaluate(&inputs);
        assert_eq!(out.verdict, Some(Verdict::Unverified));
        assert_eq!(out.notice.as_deref(), Some(NOTICE_NON_4663));
    }

    #[test]
    fn mimicry_outside_depth_boundary_is_advisory_only() {
        let mut inputs = ScanInputs::new(CHAIN_MAINNET, "0xabc0000000000000000000000000000000000001");
        inputs.structure = Some(Structure {
            beacon_proxy: false,
            beacon: None,
            impl_addr: None,
            signature_match: false,
        });
        inputs.canonical = Some(CanonicalEvidence {
            listed: false,
            mimics_name: Some("NVIDIA".into()),
            mimics_symbol: Some("NVDA".into()),
            uid_match: None,
            impl_differs: None,
        });
        let out = evaluate(&inputs);
        assert_eq!(out.verdict, Some(Verdict::Unverified));
        let mimicry = out
            .power_report
            .iter()
            .find(|r| r.result == "MIMICRY")
            .expect("mimicry disclosed");
        assert_eq!(mimicry.severity, Severity::Advisory);
    }

    #[test]
    fn impl_diff_pair_formats_like_the_fixture() {
        let (live, canonical) = impl_diff_pair(
            "0xc30d11111111111111111111111111111111e812",
            "0x77be0000000000000000000000000000000041af",
        );
        assert_eq!(live, "0xc30d…e812");
        assert_eq!(canonical, "0x77be…41af");
    }
}
