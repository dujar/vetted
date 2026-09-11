//! Wire-contract types, Rust side. TS mirror: types.ts — both must round-trip
//! the golden fixtures byte-identically (CI). Shape decisions live in wire.md.

use serde::{Deserialize, Serialize};

/// The four verdicts — the engine never guesses outside these (journeys.md:14).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Verdict {
    Verified,
    Impostor,
    Unverified,
    Revoked,
}

/// Guard revert reasons — verbatim ABI revert strings (journeys.md:32).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GuardRevertReason {
    #[serde(rename = "GUARD_NO_RECORD")]
    NoRecord,
    #[serde(rename = "GUARD_RECORD_REVOKED")]
    RecordRevoked,
    #[serde(rename = "GUARD_PAUSED")]
    Paused,
    #[serde(rename = "GUARD_BLOCKLISTED")]
    Blocklisted,
    #[serde(rename = "GUARD_IMPL_MISMATCH")]
    ImplMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardRevert {
    pub reason: GuardRevertReason,
    pub description: String,
}

/// Registry record status — on-chain u8: 0 = VERIFIED, 1 = REVOKED (wire.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecordStatus {
    Verified,
    Revoked,
}

/// Seven-field registry record (step-3 plan). `riskFlags` is a u256 carried as
/// a decimal string; timestamps are unix seconds; `revoked_at`/`reason` are
/// zero while status is VERIFIED.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryRecord {
    pub status: RecordStatus,
    /// u256 as decimal string (JSON has no u256).
    pub risk_flags: String,
    pub verified_at: u64,
    #[serde(rename = "impl")]
    pub impl_: String,
    pub registrar: String,
    pub revoked_at: u64,
    pub reason: String,
}

/// Severity of a power-report row — drives theme color (green/red duality; amber advisory).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Risk,
    Advisory,
    Verified,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PowerReportRow {
    pub check: String,
    pub result: String,
    pub severity: Severity,
    /// Every row carries evidence: tx / slot / bytecode diff / probe result (spec.md:30).
    pub evidence_url: String,
}

/// Terminal states — no verdict attempted, nothing guessed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TerminalState {
    NotContract,
    RpcRetryable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResponse {
    pub chain_id: u64,
    pub addr: String,
    /// None only when terminal_state is set.
    pub verdict: Option<Verdict>,
    pub terminal_state: Option<TerminalState>,
    /// Issuer canonical list unreachable — all verdicts UNVERIFIED (journeys.md:20).
    pub degraded: bool,
    /// Non-4663 chains: stock-token verdicts exist only on 4663 (journeys.md:19).
    pub notice: Option<String>,
    pub power_report: Vec<PowerReportRow>,
    pub record: Option<RegistryRecord>,
    /// Revocation tx for REVOKED verdicts (journeys.md:14).
    pub revocation_tx: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verdict_names_are_verbatim() {
        assert_eq!(serde_json::to_string(&Verdict::Verified).unwrap(), "\"VERIFIED\"");
        assert_eq!(serde_json::to_string(&Verdict::Impostor).unwrap(), "\"IMPOSTOR\"");
        assert_eq!(serde_json::to_string(&Verdict::Unverified).unwrap(), "\"UNVERIFIED\"");
        assert_eq!(serde_json::to_string(&Verdict::Revoked).unwrap(), "\"REVOKED\"");
    }

    #[test]
    fn guard_reasons_are_verbatim() {
        assert_eq!(
            serde_json::to_string(&GuardRevertReason::NoRecord).unwrap(),
            "\"GUARD_NO_RECORD\""
        );
        assert_eq!(
            serde_json::to_string(&GuardRevertReason::RecordRevoked).unwrap(),
            "\"GUARD_RECORD_REVOKED\""
        );
        assert_eq!(
            serde_json::to_string(&GuardRevertReason::Paused).unwrap(),
            "\"GUARD_PAUSED\""
        );
        assert_eq!(
            serde_json::to_string(&GuardRevertReason::Blocklisted).unwrap(),
            "\"GUARD_BLOCKLISTED\""
        );
        assert_eq!(
            serde_json::to_string(&GuardRevertReason::ImplMismatch).unwrap(),
            "\"GUARD_IMPL_MISMATCH\""
        );
    }

    #[test]
    fn registry_record_uses_wire_field_names() {
        let rec = RegistryRecord {
            status: RecordStatus::Revoked,
            risk_flags: "0".into(),
            verified_at: 1788048000,
            impl_: "0x77be0000000000000000000000000000000041af".into(),
            registrar: "0x5e1500000000000000000000000000000000aa11".into(),
            revoked_at: 1788652800,
            reason: "0xab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12".into(),
        };
        let v: serde_json::Value = serde_json::to_value(&rec).unwrap();
        for key in [
            "status",
            "riskFlags",
            "verifiedAt",
            "impl",
            "registrar",
            "revokedAt",
            "reason",
        ] {
            assert!(v.get(key).is_some(), "missing wire key {key}");
        }
    }
}
