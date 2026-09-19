//! Issuer canonical list — `GET https://api.robinhood.com/rhj/assets`, no auth
//! (knowledge robinhood-stock-tokens.md). ABSORBED from the proven spike
//! worker per plan Revised note 2 (source: spike/fetch/): production /scan
//! fetches the issuer directly, no dependency on the spike-owned URL. The
//! spike deployment stays up as a fallback mirror.
//!
//! Known footgun carried over from the spike source: api.robinhood.com
//! returns an empty body to a default worker fetch without a User-Agent.

use serde::Deserialize;

/// The User-Agent the issuer's edge requires (observed live 2026-09-12).
pub const USER_AGENT: &str = "vetted-scan-backend/0.1";

#[derive(Deserialize, Debug, Clone)]
pub struct Deployment {
    #[serde(rename = "contractAddress")]
    pub contract_address: String,
    #[serde(rename = "chainId")]
    pub chain_id: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Asset {
    /// On-chain `uid()` of the token — "0x" + 66 chars, stable across chains.
    pub id: String,
    #[serde(rename = "tokenSymbol")]
    pub token_symbol: String,
    #[serde(rename = "tokenName")]
    pub token_name: String,
    pub deployments: Vec<Deployment>,
}

#[derive(Debug, Clone)]
pub struct CanonicalList {
    pub assets: Vec<Asset>,
}

/// Pure: parse the registry body. Host-independent on purpose (native tests).
pub fn parse(body: &str) -> Result<CanonicalList, String> {
    #[derive(Deserialize)]
    struct Envelope {
        assets: Vec<Asset>,
    }
    let env: Envelope =
        serde_json::from_str(body).map_err(|e| format!("registry parse failed: {e}"))?;
    Ok(CanonicalList { assets: env.assets })
}

impl CanonicalList {
    /// Is `addr` a canonical deployment on `chain_id`?
    pub fn find_by_address(&self, chain_id: u64, addr: &str) -> Option<&Asset> {
        let addr = addr.to_ascii_lowercase();
        self.assets.iter().find(|a| {
            a.deployments.iter().any(|d| {
                d.chain_id == chain_id && d.contract_address.to_ascii_lowercase() == addr
            })
        })
    }

    /// Rule-1 mimicry evidence: a canonical asset whose name AND symbol equal
    /// the scanned token's metadata, at a different address.
    pub fn find_mimic_source(&self, name: &str, symbol: &str, scanned: &str) -> Option<&Asset> {
        let scanned = scanned.to_ascii_lowercase();
        self.assets.iter().find(|a| {
            a.token_name == name
                && a.token_symbol == symbol
                && !a
                    .deployments
                    .iter()
                    .any(|d| d.contract_address.to_ascii_lowercase() == scanned)
        })
    }

    /// 4663 deployment addresses — the drift-watch's candidate walk.
    pub fn mainnet_addresses(&self) -> Vec<String> {
        self.assets
            .iter()
            .filter_map(|a| {
                a.deployments
                    .iter()
                    .find(|d| d.chain_id == crate::rules::CHAIN_MAINNET)
                    .map(|d| d.contract_address.to_ascii_lowercase())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{"assets": [
      {"id":"0x0000000000000000000000000000000002c4d1ce31ec4310b2c507c921a52b70",
       "tokenSymbol":"P","tokenName":"Everpure • Robinhood Token",
       "deployments":[{"contractAddress":"0x1Cdad396DB64BDa184d5182A97Dd9B3C62100b7D","chainId":4663}]},
      {"id":"0x01","tokenSymbol":"CRM","tokenName":"Salesforce • Robinhood Token",
       "deployments":[{"contractAddress":"0xd95B44124e475743a7589e68F3D74008A5536D44","chainId":4663}]}
    ]}"#;

    #[test]
    fn parses_and_finds_by_address() {
        let list = parse(SAMPLE).unwrap();
        assert_eq!(list.assets.len(), 2);
        let p = list
            .find_by_address(4663, "0x1cdad396db64bda184d5182a97dd9b3c62100b7d")
            .expect("P listed (case-insensitive)");
        assert_eq!(p.token_symbol, "P");
        assert_eq!(p.id, "0x0000000000000000000000000000000002c4d1ce31ec4310b2c507c921a52b70");
        assert!(list.find_by_address(4663, "0x9999999999999999999999999999999999999999").is_none());
    }

    #[test]
    fn finds_mimic_sources_at_a_different_address() {
        let list = parse(SAMPLE).unwrap();
        let twin = list
            .find_mimic_source(
                "Everpure • Robinhood Token",
                "P",
                "0x9999999999999999999999999999999999999999",
            )
            .expect("twin with canonical name+symbol at another address has a mimic source");
        assert_eq!(twin.token_symbol, "P");
        // The canonical address itself is not a mimic of anything.
        assert!(list
            .find_mimic_source(
                "Everpure • Robinhood Token",
                "P",
                "0x1cdad396db64bda184d5182a97dd9b3c62100b7d"
            )
            .is_none());
    }

    #[test]
    fn mainnet_addresses_for_the_drift_walk() {
        let list = parse(SAMPLE).unwrap();
        let addrs = list.mainnet_addresses();
        assert_eq!(addrs.len(), 2);
        assert!(addrs.contains(&"0x1cdad396db64bda184d5182a97dd9b3c62100b7d".to_string()));
    }

    #[test]
    fn parse_failure_is_a_reported_error() {
        assert!(parse("<html>login page</html>").is_err());
    }
}
