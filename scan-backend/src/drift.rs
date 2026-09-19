//! Registrar drift-watch (spec.md:35): for each ACTIVE (VERIFIED) record,
//! resolve the live implementation and revoke on drift. The registry stays the
//! state — this only sends the revoke transaction the registry already
//! understands; no database anywhere.
//!
//! Candidates: Verify-event enumeration (topic0 pinned in hexutil::events,
//! byte-equal to step-3's events.ts) + the issuer list (+ DRIFT_EXTRA_TOKENS);
//! each source degrades independently.

use alloy_signer_local::PrivateKeySigner;
use serde::Serialize;
use vetted_shared::types::RecordStatus;

use crate::fingerprint;
use crate::hexutil::norm_addr;
use crate::rpc::{CallResult, RpcClient, Transport};
use crate::signer;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Revocation {
    pub token: String,
    pub tx_hash: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DriftReport {
    pub checked: usize,
    pub revoked: Vec<Revocation>,
    pub errors: Vec<String>,
    /// Set when the run could not start (key/registry unconfigured) — the
    /// degraded shape the demo window can hit before step 7's handoff.
    pub skipped: Option<String>,
}

impl DriftReport {
    pub fn skipped(reason: &str) -> Self {
        Self {
            checked: 0,
            revoked: vec![],
            errors: vec![],
            skipped: Some(reason.to_string()),
        }
    }
}

/// Decode `getRecord(address)`'s 7-word tuple into the wire record — the
/// decode the /scan path uses too. None on malformed returndata.
pub fn decode_record(returndata: &str) -> Option<vetted_shared::types::RegistryRecord> {
    let bytes = crate::hexutil::decode_hex(returndata)?;
    if bytes.len() != 32 * 7 {
        return None;
    }
    let word = |i: usize| &bytes[i * 32..(i + 1) * 32];
    let status = match word(0)[31] {
        0 => RecordStatus::Verified,
        1 => RecordStatus::Revoked,
        // Unknown u8 = no-record, never guessed (wire.md).
        _ => return None,
    };
    let u64_at = |i: usize| -> Option<u64> {
        let w = word(i);
        let mut b = [0u8; 8];
        b.copy_from_slice(&w[24..]);
        Some(u64::from_be_bytes(b))
    };
    let addr_at = |i: usize| -> Option<String> {
        norm_addr(&format!(
            "0x{}",
            crate::hexutil::encode_hex(&word(i)[12..])
        ))
    };
    let risk_flags = alloy_primitives::U256::from_be_slice(word(1)).to_string();
    Some(vetted_shared::types::RegistryRecord {
        status,
        risk_flags,
        verified_at: u64_at(2)?,
        impl_: addr_at(3)?,
        registrar: addr_at(4)?,
        revoked_at: u64_at(5)?,
        reason: format!("0x{}", crate::hexutil::encode_hex(word(6))),
    })
}

/// Read one token's record through the RPC transport. None = no record.
pub async fn get_record<T: Transport>(
    rpc: &RpcClient<T>,
    registry: &str,
    token: &str,
) -> Result<Option<vetted_shared::types::RegistryRecord>, crate::rpc::RpcError> {
    let data = crate::hexutil::call_addr(crate::hexutil::selectors::GET_RECORD, token);
    match rpc.call(registry, &data).await? {
        CallResult::Ok(hex) if hex.len() >= 2 && crate::hexutil::decode_hex(&hex).map(|b| b.len()).unwrap_or(0) == 224 => {
            Ok(decode_record(&hex))
        }
        // Empty/reverted returndata = no record (registry returns zeroed tuple
        // for unknown tokens; step 3's contract decides the exact encoding).
        _ => Ok(None),
    }
}

/// The revocation tx for a REVOKED record: the last `Revoke(token, …)` log's
/// transactionHash (plan Revised note 5). None = not indexed (the frontend's
/// existing rendering) — a failed read never blocks the scan.
pub fn last_log_tx_hash(logs: &[serde_json::Value]) -> Option<String> {
    logs.iter().rev().find_map(|log| {
        log.get("transactionHash")
            .and_then(|v| v.as_str())
            .map(str::to_string)
    })
}

/// Indexed tokens: the `Verify(?, ?, ?)` logs' topic1 values — the drift
/// walk's complete candidate list once the registry is live.
pub fn tokens_from_verify_logs(logs: &[serde_json::Value]) -> Vec<String> {
    let mut out = Vec::new();
    for log in logs {
        if let Some(topic) = log.get("topics").and_then(|t| t.get(1)).and_then(|v| v.as_str()) {
            if let Some(addr) = crate::hexutil::word_to_addr(topic) {
                if addr != "0x0000000000000000000000000000000000000000" && !out.contains(&addr) {
                    out.push(addr);
                }
            }
        }
    }
    out
}

/// One drift-check pass over `candidates`. Sends revoke transactions when a
/// VERIFIED record's implementation pointer has drifted from the live beacon.
pub async fn run_drift_check<T: Transport>(
    rpc: &RpcClient<T>,
    chain_id: u64,
    registry: &str,
    signer: &PrivateKeySigner,
    candidates: &[String],
) -> DriftReport {
    let mut report = DriftReport {
        checked: 0,
        revoked: vec![],
        errors: vec![],
        skipped: None,
    };

    let gas_price = match rpc.gas_price().await {
        Ok(p) => p,
        Err(e) => {
            report.errors.push(format!("gas price unavailable: {e:?}"));
            return report;
        }
    };

    for token in candidates {
        let token = match norm_addr(token) {
            Some(t) => t,
            None => {
                report.errors.push(format!("bad candidate address {token}"));
                continue;
            }
        };
        let record = match get_record(rpc, registry, &token).await {
            Ok(Some(r)) => r,
            Ok(None) => continue, // no record — nothing to watch
            Err(e) => {
                report.errors.push(format!("{token}: record read failed: {e:?}"));
                continue;
            }
        };
        if record.status != RecordStatus::Verified {
            continue; // only ACTIVE records are watched
        }
        report.checked += 1;

        let code = match rpc.get_code(&token).await {
            Ok(c) => c,
            Err(e) => {
                report.errors.push(format!("{token}: code read failed: {e:?}"));
                continue;
            }
        };
        let layout = fingerprint::resolve(rpc, &token, &code).await;
        let drifted = layout
            .impl_addr
            .as_ref()
            .map(|live| !live.eq_ignore_ascii_case(&record.impl_))
            .unwrap_or(false);
        if !drifted {
            continue;
        }

        // Revoke on drift (spec.md:35). Fixed gas ceiling — revoke is a
        // storage write far under it.
        // ponytail: estimate-first with a 400k fallback would be two extra
        // reads per revoke; the fixed ceiling is safe for the single function.
        let nonce = match rpc.transaction_count(&signer.address().to_string()).await {
            Ok(n) => n,
            Err(e) => {
                report.errors.push(format!("{token}: nonce failed: {e:?}"));
                continue;
            }
        };
        let max_fee = gas_price.saturating_mul(2).max(2_000_000_000);
        let registry_addr: alloy_primitives::Address = match registry.parse() {
            Ok(a) => a,
            Err(_) => {
                report.errors.push("registry address malformed".into());
                return report;
            }
        };
        let mut tx = signer::build_revoke_tx(
            chain_id,
            nonce,
            400_000,
            max_fee,
            100_000_000,
            registry_addr,
            &token,
            signer::DRIFT_REASON,
        );
        let raw = match signer::sign_tx(signer, &mut tx) {
            Ok(r) => r,
            Err(e) => {
                report.errors.push(format!("{token}: signing failed: {e}"));
                continue;
            }
        };
        match rpc.send_raw_transaction(&raw).await {
            Ok(tx_hash) => report.revoked.push(Revocation { token, tx_hash }),
            Err(e) => report.errors.push(format!("{token}: send failed: {e:?}")),
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_the_seven_field_record() {
        // status=0 VERIFIED, riskFlags=0, verifiedAt, impl, registrar, revokedAt=0, reason=0
        let mut data = String::from("0x");
        data.push_str(&"0".repeat(64)); // status 0
        data.push_str(&"0".repeat(64)); // riskFlags 0
        data.push_str(&format!("{:064x}", 1788048000u64)); // verifiedAt
        data.push_str(&"0".repeat(24)); // impl padding
        data.push_str("77be0000000000000000000000000000000041af");
        data.push_str(&"0".repeat(24)); // registrar padding
        data.push_str("5e1500000000000000000000000000000000aa11");
        data.push_str(&"0".repeat(64)); // revokedAt 0
        data.push_str(&"0".repeat(64)); // reason 0
        let rec = decode_record(&data).expect("decodes");
        assert_eq!(rec.status, RecordStatus::Verified);
        assert_eq!(rec.risk_flags, "0");
        assert_eq!(rec.verified_at, 1788048000);
        assert_eq!(rec.impl_, "0x77be0000000000000000000000000000000041af");
        assert_eq!(rec.revoked_at, 0);
    }

    #[test]
    fn unknown_status_u8_is_no_record() {
        let mut data = String::from("0x");
        data.push_str(&format!("{:064x}", 7u64)); // status 7 — outside {0,1}
        data.push_str(&"0".repeat(64 * 6));
        assert!(decode_record(&data).is_none());
        assert!(decode_record("0x").is_none());
        assert!(decode_record("0x1234").is_none());
    }

    #[tokio::test]
    async fn skipped_report_when_there_is_nothing_to_do() {
        let report = DriftReport::skipped("registrar_key_not_configured");
        assert_eq!(report.checked, 0);
        assert_eq!(report.skipped.as_deref(), Some("registrar_key_not_configured"));
    }

    #[test]
    fn revocation_tx_extracted_from_the_last_revoke_log() {
        let logs: Vec<serde_json::Value> = serde_json::from_str(
            r#"[
              {"transactionHash": "0xaaa1", "topics": ["0x2fa8…", "0x…token"]},
              {"transactionHash": "0xbbb2", "topics": ["0x2fa8…", "0x…token"]}
            ]"#,
        )
        .unwrap();
        assert_eq!(last_log_tx_hash(&logs).as_deref(), Some("0xbbb2"));
        assert_eq!(last_log_tx_hash(&[]), None);
    }

    #[test]
    fn verify_logs_yield_indexed_tokens() {
        let logs: Vec<serde_json::Value> = serde_json::from_str(
            r#"[
              {"topics": ["0x3e79…", "0x0000000000000000000000001cdad396db64bda184d5182a97dd9b3c62100b7d"]},
              {"topics": ["0x3e79…", "0x00000000000000000000000077be0000000000000000000000000000000041af"]}
            ]"#,
        )
        .unwrap();
        let tokens = tokens_from_verify_logs(&logs);
        assert_eq!(
            tokens,
            vec![
                "0x1cdad396db64bda184d5182a97dd9b3c62100b7d".to_string(),
                "0x77be0000000000000000000000000000000041af".to_string(),
            ]
        );
    }
}
