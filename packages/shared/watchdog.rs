//! Watchdog widget wire type, Rust mirror of watchdog.ts (step 4). Both sides
//! round-trip fixtures/watchdog byte-identically (CI).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchdogStats {
    pub chain_id: u64,
    /// Cumulative runs — `0` with a provenance_url means the live count is
    /// UNAVAILABLE (degrade path), never a measured zero (wire.md).
    pub runs: u64,
    /// Published 6-week baseline, daily rate (~150/day, measured 2026-08).
    pub baseline_per_day: u64,
    /// Live counter's public entry, or the published-baseline source when
    /// degraded. null = no provenance at all.
    pub provenance_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_field_names_are_camel_case() {
        let s = WatchdogStats {
            chain_id: 4663,
            runs: 0,
            baseline_per_day: 150,
            provenance_url: None,
        };
        let v: serde_json::Value = serde_json::to_value(&s).unwrap();
        for key in ["chainId", "runs", "baselinePerDay", "provenanceUrl"] {
            assert!(v.get(key).is_some(), "missing wire key {key}");
        }
    }
}
