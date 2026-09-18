//! Week-1 spike — canonical-list fetch (step 2 task 7). Proves the machine
//! ground truth the scanner's impostor rule (spec.md:27) runs on:
//! `GET https://api.robinhood.com/rhj/assets` — no auth, 60 req/s, ~194
//! assets (knowledge robinhood-stock-tokens.md:12–19). Supersedes spec spike
//! bullet 5's docs-page `.md`/JSON spelunking (spec.md:55) — the docs page
//! renders from this same registry.
//!
//! `GET /assets`  → live fetch + summary: total count, plus the calibration
//!                  rows (P, CRM) with contractAddress + chainId 4663.
//! `GET /health`  → `{"ok":true}`.
//! Fetch failure returns `{"ok":false,...}` — the degraded branch (all-
//! UNVERIFIED banner, spec.md:55) is step 4's consumer of exactly this shape.

use serde::Deserialize;
use worker::*;

const REGISTRY_URL: &str = "https://api.robinhood.com/rhj/assets";

/// The subset of the registry row shape this step verifies
/// (knowledge robinhood-stock-tokens.md:18–22).
#[derive(Deserialize, Debug)]
pub struct Deployment {
    #[serde(rename = "contractAddress")]
    pub contract_address: String,
    #[serde(rename = "chainId")]
    pub chain_id: u64,
}

#[derive(Deserialize, Debug)]
pub struct Asset {
    #[serde(rename = "tokenSymbol")]
    pub token_symbol: String,
    #[serde(rename = "tokenName")]
    pub token_name: String,
    pub deployments: Vec<Deployment>,
}

/// Pure: summarize the registry body — total rows + the calibration symbols'
/// deployments. Host-independent on purpose (native tests). Live envelope is
/// `{"assets": [...]}` (verified 2026-09-12, 194 rows — rhj_assets_sample.json).
pub fn summarize(body: &str) -> Result<String, String> {
    #[derive(Deserialize)]
    struct Envelope {
        assets: Vec<Asset>,
    }
    let env: Envelope =
        serde_json::from_str(body).map_err(|e| format!("registry parse failed: {e}"))?;
    let assets = env.assets;
    let mut out = serde_json::Map::new();
    out.insert("ok".into(), serde_json::Value::Bool(true));
    out.insert("total".into(), serde_json::Value::from(assets.len()));
    let mut calib = serde_json::Map::new();
    for sym in ["P", "CRM"] {
        let rows: Vec<serde_json::Value> = assets
            .iter()
            .filter(|a| a.token_symbol == sym)
            .map(|a| {
                serde_json::json!({
                    "tokenName": a.token_name,
                    "deployments": a.deployments.iter().map(|d| serde_json::json!({
                        "contractAddress": d.contract_address,
                        "chainId": d.chain_id,
                    })).collect::<Vec<_>>(),
                })
            })
            .collect();
        calib.insert(sym.into(), serde_json::Value::Array(rows));
    }
    out.insert("calibration".into(), serde_json::Value::Object(calib));
    serde_json::to_string(&serde_json::Value::Object(out)).map_err(|e| e.to_string())
}

fn json_headers() -> Result<Headers> {
    let headers = Headers::new();
    headers.set("Content-Type", "application/json")?;
    headers.set("Access-Control-Allow-Origin", "*")?;
    Ok(headers)
}

#[event(fetch)]
async fn main(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    match (req.method().to_string().as_str(), req.path().as_str()) {
        ("GET", "/health") => Ok(Response::ok(r#"{"ok":true}"#)?),
        ("GET", "/assets") => {
            // api.robinhood.com returns an empty body to a default worker
            // fetch without a User-Agent (observed live 2026-09-12) — send one
            // and surface the upstream status in the degraded branch.
            let mut headers = Headers::new();
            headers.set("User-Agent", "vetted-spike-canonical-fetch/0.1")?;
            let init = RequestInit {
                method: Method::Get,
                headers,
                ..Default::default()
            };
            let upstream = Request::new_with_init(REGISTRY_URL, &init)?;
            match Fetch::Request(upstream).send().await {
                Ok(mut res) => match res.status_code() {
                    200 => match res.text().await {
                        Ok(body) => match summarize(&body) {
                            Ok(summary) => {
                                Ok(Response::ok(summary)?.with_headers(json_headers()?))
                            }
                            Err(e) => Response::error(format!("degraded: {e}"), 502),
                        },
                        Err(e) => Response::error(format!("degraded: {e}"), 502),
                    },
                    code => Response::error(
                        format!("degraded: upstream status {code}"),
                        502,
                    ),
                },
                Err(e) => Response::error(format!("degraded: {e}"), 502),
            }
        }
        _ => Response::error("not found", 404),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{"assets": [
      {"id":"0x00","tokenSymbol":"P","tokenName":"Everpure • Robinhood Token",
       "deployments":[{"contractAddress":"0x1Cdad396DB64BDa184d5182A97Dd9B3C62100b7D","chainId":4663}]},
      {"id":"0x01","tokenSymbol":"CRM","tokenName":"Salesforce • Robinhood Token",
       "deployments":[{"contractAddress":"0xd95B44124e475743a7589e68F3D74008A5536D44","chainId":4663}]}
    ]}"#;

    #[test]
    fn summarize_counts_and_finds_calibration_rows() {
        let s = summarize(SAMPLE).unwrap();
        assert!(s.contains(r#""total":2"#));
        assert!(s.contains("0x1Cdad396DB64BDa184d5182A97Dd9B3C62100b7D"));
        assert!(s.contains("0xd95B44124e475743a7589e68F3D74008A5536D44"));
        assert!(s.contains(r#""chainId":4663"#));
    }

    #[test]
    fn summarize_reports_parse_failure() {
        assert!(summarize(r#"{"assets": []}"#).is_ok());
        assert!(summarize("<html>login page</html>").is_err());
    }
}
