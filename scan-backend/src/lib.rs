//! Vetted scan backend — the stateless verdict engine (step 4). Route shape
//! stays step 1's contract: pure route parser (unit-testable on any target) +
//! thin `#[event(fetch)]`/`#[event(scheduled)]` wrappers + permissive CORS
//! (knowledge workers-rust.md — scans are anonymous, no session state).
//!
//! Wire contract: packages/shared/wire.md — `GET /scan?chainId=&addr=`,
//! `GET /watchdog?chainId=`, `POST /registry/drift-check` (admin-secret gated,
//! same code path as the cron). The engine emits the shared types directly
//! (`vetted-shared` path dep) so it cannot drift from the golden fixtures.

pub mod canonical;
pub mod drift;
pub mod fingerprint;
pub mod hexutil;
pub mod rpc;
pub mod rules;
pub mod routes;
pub mod signer;
pub mod watchdog;

use vetted_shared::types::ScanResponse;
use vetted_shared::watchdog::WatchdogStats;

use worker::*;

use rules::{ScanInputs, Structure};

/// The calibration probe buyer (a well-known funded EOA the spike's live
/// reads used — probes judge answered-vs-reverted, never the answer).
const FRESH_BUYER: &str = "0xd8da6bf26964af9d7eed9e03e53415d37aa96045";

/// The calibrated shared implementation (calibration_4663.json) — the
/// impostor impl-diff evidence's canonical side.
/// ponytail: a constant, not a per-scan re-resolution of a canonical token's
/// beacon (that would double the impostor path's RPC cost); if the issuer
/// upgrades, refresh from calibration evidence.
const CANONICAL_IMPL: &str = "0xb35490d6f9163de4f80d88dc75c3516eb64c5ae2";
const CANONICAL_IMPL_CODEHASH: &str =
    "0xdc07e86ee482f99641bdafb9a0d772846b167401e094d90a666b94dbdcd1eec7";

fn chain_rpc(chain_id: u64) -> Option<&'static str> {
    match chain_id {
        rules::CHAIN_MAINNET => Some("https://rpc.mainnet.chain.robinhood.com"),
        rules::CHAIN_TESTNET => Some("https://rpc.testnet.chain.robinhood.com"),
        rules::CHAIN_ARB_SEPOLIA => Some("https://sepolia-rollup.arbitrum.io/rpc"),
        _ => None,
    }
}

/// The worker's outbound transport — the runtime half of rpc::Transport.
struct FetchTransport;

impl rpc::Transport for FetchTransport {
    async fn post(&self, url: &str, body: String) -> Result<String, String> {
        let headers = Headers::new();
        headers.set("Content-Type", "application/json").map_err(|e| e.to_string())?;
        let init = RequestInit {
            method: Method::Post,
            headers,
            body: Some(body.into()),
            ..Default::default()
        };
        let req = Request::new_with_init(url, &init).map_err(|e| e.to_string())?;
        let mut res = Fetch::Request(req).send().await.map_err(|e| e.to_string())?;
        res.text().await.map_err(|e| e.to_string())
    }
}

/// The issuer fetch — worker-side half of canonical.rs (needs a User-Agent:
/// api.robinhood.com serves an empty body without one, spike/fetch findings).
async fn fetch_canonical() -> Result<canonical::CanonicalList, String> {
    let headers = Headers::new();
    headers.set("User-Agent", canonical::USER_AGENT).map_err(|e| e.to_string())?;
    let init = RequestInit {
        method: Method::Get,
        headers,
        ..Default::default()
    };
    let req =
        Request::new_with_init(rules::CANONICAL_ASSETS_URL, &init).map_err(|e| e.to_string())?;
    let mut res = Fetch::Request(req).send().await.map_err(|e| e.to_string())?;
    if res.status_code() != 200 {
        return Err(format!("canonical fetch status {}", res.status_code()));
    }
    let body = res.text().await.map_err(|e| e.to_string())?;
    canonical::parse(&body)
}

fn env_var(env: &Env, name: &str) -> Option<String> {
    env.var(name)
        .ok()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
}

struct Config {
    registry_4663: Option<String>,
    registry_46630: Option<String>,
    registry_421614: Option<String>,
    registrar_key: Option<String>,
    drift_admin: Option<String>,
    drift_extra: Vec<String>,
}

fn config(env: &Env) -> Config {
    Config {
        registry_4663: env_var(env, "REGISTRY_ADDRESS_4663"),
        registry_46630: env_var(env, "REGISTRY_ADDRESS_46630"),
        registry_421614: env_var(env, "REGISTRY_ADDRESS_421614"),
        registrar_key: env_var(env, "REGISTRAR_KEY"),
        drift_admin: env_var(env, "DRIFT_ADMIN_SECRET"),
        drift_extra: env_var(env, "DRIFT_EXTRA_TOKENS")
            .map(|s| s.split(',').map(|t| t.trim().to_string()).collect())
            .unwrap_or_default(),
    }
}

fn registry_for(cfg: &Config, chain_id: u64) -> Option<String> {
    match chain_id {
        rules::CHAIN_MAINNET => cfg.registry_4663.clone(),
        rules::CHAIN_TESTNET => cfg.registry_46630.clone(),
        rules::CHAIN_ARB_SEPOLIA => cfg.registry_421614.clone(),
        _ => None,
    }
}

/// Canonical ground-truth evidence for one scanned address (rule 1). Needs the
/// resolved layout for the impl-diff evidence pair.
async fn canonical_evidence(
    client: &rpc::RpcClient<FetchTransport>,
    chain_id: u64,
    addr: &str,
    layout: &fingerprint::Layout,
) -> Result<Option<rules::CanonicalEvidence>, String> {
    let list = fetch_canonical().await?;
    if let Some(asset) = list.find_by_address(chain_id, addr) {
        // The strongest single check: on-chain uid() == issuer row id
        // (calibration_4663.json). A mismatch drops the listing to UNVERIFIED.
        let uid_match = match client.call(addr, hexutil::selectors::UID).await {
            Ok(rpc::CallResult::Ok(hex)) => {
                Some(hex.eq_ignore_ascii_case(&asset.id))
            }
            _ => None,
        };
        return Ok(Some(rules::CanonicalEvidence {
            listed: true,
            mimics_name: None,
            mimics_symbol: None,
            uid_match,
            impl_differs: None,
        }));
    }
    // Not listed — metadata reads for the rule-1 mimicry evidence.
    let name = match client.call(addr, hexutil::selectors::NAME).await {
        Ok(rpc::CallResult::Ok(hex)) => hexutil::decode_abi_string(&hex),
        _ => None,
    };
    let symbol = match client.call(addr, hexutil::selectors::SYMBOL).await {
        Ok(rpc::CallResult::Ok(hex)) => hexutil::decode_abi_string(&hex),
        _ => None,
    };
    let (Some(name), Some(symbol)) = (name, symbol) else {
        return Ok(None);
    };
    let mimic = list.find_mimic_source(&name, &symbol, addr);
    let impl_differs = mimic.and_then(|_| {
        // Positive evidence needs the live implementation to differ from the
        // canonical implementation's — compared by codehash (upgrade-proof),
        // displayed as the calibrated pair (IMPOSTOR fixture format).
        let live_hash = layout.impl_codehash.as_deref()?;
        (live_hash.to_ascii_lowercase() != CANONICAL_IMPL_CODEHASH).then(|| {
            (
                hexutil::short_addr(layout.impl_addr.as_deref().unwrap_or("0x0000")),
                hexutil::short_addr(CANONICAL_IMPL),
            )
        })
    });
    Ok(mimic.map(|a| rules::CanonicalEvidence {
        listed: false,
        mimics_name: Some(a.token_name.clone()),
        mimics_symbol: Some(a.token_symbol.clone()),
        uid_match: None,
        impl_differs,
    }))
}

/// The scan pipeline (plan task 1–4). Returns the wire response or an error
/// string for 400s.
async fn scan(
    cfg: &Config,
    chain_id: u64,
    addr: &str,
) -> Result<ScanResponse, String> {
    let addr = hexutil::norm_addr(addr).ok_or("bad_address")?;
    let rpc_url = chain_rpc(chain_id).ok_or("unknown_chain")?;
    let client = rpc::RpcClient::new(rpc_url, FetchTransport);

    let code = match client.get_code(&addr).await {
        Ok(c) => c,
        Err(_) => return Ok(rules::rpc_retryable(chain_id, &addr)),
    };
    if code.is_empty() {
        let mut inputs = ScanInputs::new(chain_id, &addr);
        inputs.has_code = false;
        // A registry record can outlive the code (selfdestruct) — still shown.
        if let Some(reg) = registry_for(cfg, chain_id) {
            inputs.record = drift::get_record(&client, &reg, &addr).await.ok().flatten();
        }
        return Ok(rules::evaluate(&inputs));
    }

    // Resolve the beacon (guard-pinned path) + probes at calibrated targets.
    // probe() yields None when a probe READ failed — the scan then completes
    // with no probe rows and no signature match (honest UNVERIFIED), never
    // with fabricated ABSENT rows.
    let mut layout = fingerprint::resolve(&client, &addr, &code).await;
    let probes = fingerprint::probe(&client, &addr, layout.beacon.as_deref(), FRESH_BUYER).await;
    let (paused, blocklist) = probes
        .as_ref()
        .map(|p| (p.paused, p.blocklist))
        .unwrap_or((false, false));
    fingerprint::finalize_signature_match(&mut layout, paused, blocklist);

    let mut inputs = ScanInputs::new(chain_id, &addr);
    inputs.probes = probes;
    inputs.structure = Some(Structure {
        beacon_proxy: layout.beacon_slot_set && layout.impl_slot_empty,
        beacon: layout.beacon.clone(),
        impl_addr: layout.impl_addr.clone(),
        signature_match: layout.signature_match,
    });

    // Canonical ground truth, fetched live at scan time (rule 1) — 4663 only;
    // failure degrades every verdict to UNVERIFIED (journeys.md:20).
    if chain_id == rules::CHAIN_MAINNET {
        inputs.canonical = match canonical_evidence(&client, chain_id, &addr, &layout).await {
            Ok(e) => e,
            Err(_) => {
                inputs.degraded = true;
                None
            }
        };
    }

    // Registry record — unset registry => record:null, canonical-only rules
    // (verify loose end 1). A failed read is not fatal: record:null.
    if let Some(reg) = registry_for(cfg, chain_id) {
        inputs.record = drift::get_record(&client, &reg, &addr).await.ok().flatten();
        // REVOKED links its revocation tx — the last Revoke(token,…) log
        // (plan Revised note 5, step 3's event pin). Failure degrades to
        // null: the frontend's existing "not indexed yet" rendering.
        if inputs
            .record
            .as_ref()
            .map(|r| r.status == vetted_shared::types::RecordStatus::Revoked)
            .unwrap_or(false)
        {
            if let Ok(logs) = client
                .get_logs(
                    &reg,
                    vec![
                        Some(hexutil::events::REVOKE_TOPIC0.to_string()),
                        Some(hexutil::addr_word(&addr)),
                    ],
                )
                .await
            {
                inputs.revocation_tx = drift::last_log_tx_hash(&logs);
            }
        }
    }

    Ok(rules::evaluate(&inputs))
}

fn json_response(status: u16, body: String) -> Result<Response> {
    Ok(Response::ok(body)?
        .with_status(status)
        .with_headers(json_headers()?))
}

fn json_headers() -> Result<Headers> {
    let headers = Headers::new();
    headers.set("Content-Type", "application/json")?;
    headers.set("Access-Control-Allow-Origin", "*")?;
    headers.set("Access-Control-Allow-Methods", "GET, POST, OPTIONS")?;
    headers.set("Access-Control-Allow-Headers", "Content-Type, X-Admin-Secret")?;
    Ok(headers)
}

fn run_drift_check(cfg: &Config) -> impl std::future::Future<Output = String> {
    let registry = cfg.registry_4663.clone();
    let key = cfg.registrar_key.clone();
    let drift_extra = cfg.drift_extra.clone();
    async move {
        let Some(registry) = registry else {
            return serde_json::to_string(&drift::DriftReport::skipped("registry_not_configured"))
                .unwrap();
        };
        let Some(key) = key else {
            return serde_json::to_string(&drift::DriftReport::skipped("registrar_key_not_configured"))
                .unwrap();
        };
        let signer = match signer::signer_from_key(&key) {
            Ok(s) => s,
            Err(e) => {
                return serde_json::to_string(&drift::DriftReport::skipped(&format!(
                    "registrar_key_invalid: {e}"
                )))
                .unwrap();
            }
        };
        // Candidates: Verify-event enumeration (complete once the registry is
        // live) + the issuer list + operator-pinned extras; each source
        // degrades independently.
        let client = rpc::RpcClient::new(
            chain_rpc(rules::CHAIN_MAINNET).unwrap(),
            FetchTransport,
        );
        let mut candidates = Vec::new();
        if let Ok(logs) = client
            .get_logs(&registry, vec![Some(hexutil::events::VERIFY_TOPIC0.to_string())])
            .await
        {
            candidates.extend(drift::tokens_from_verify_logs(&logs));
        }
        match fetch_canonical().await {
            Ok(list) => candidates.extend(list.mainnet_addresses()),
            Err(_) => {
                if candidates.is_empty() && drift_extra.is_empty() {
                    return serde_json::to_string(&drift::DriftReport::skipped(
                        "no_candidates: registry events unavailable and canonical fetch failed",
                    ))
                    .unwrap();
                }
            }
        }
        candidates.extend(
            drift_extra
                .iter()
                .filter_map(|t| hexutil::norm_addr(t)),
        );
        candidates.sort();
        candidates.dedup();
        let report = drift::run_drift_check(
            &client,
            rules::CHAIN_MAINNET,
            &registry,
            &signer,
            &candidates,
        )
        .await;
        serde_json::to_string(&report).unwrap()
    }
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // CORS preflight — scans are anonymous, permissive (knowledge
    // workers-rust.md:32).
    if req.method() == Method::Options {
        return Ok(Response::empty()
            .unwrap()
            .with_status(204)
            .with_headers(json_headers()?));
    }
    let method = req.method().to_string();
    let path = req.path();
    let query = req.url().ok().map(|u| u.query().unwrap_or_default().to_string()).unwrap_or_default();
    let cfg = config(&env);

    match routes::parse(&method, &path, &query) {
        routes::Route::Health => json_response(200, r#"{"ok":true}"#.to_string()),
        routes::Route::Scan { chain_id, addr } => match scan(&cfg, chain_id, &addr).await {
            Ok(resp) => json_response(200, serde_json::to_string(&resp).unwrap_or_default()),
            Err(e) => json_response(
                400,
                serde_json::json!({ "error": e }).to_string(),
            ),
        },
        routes::Route::Watchdog { chain_id } => {
            if chain_rpc(chain_id).is_none() {
                return json_response(400, r#"{"error":"unknown_chain"}"#.to_string());
            }
            let stats: WatchdogStats = watchdog::stats(chain_id);
            json_response(200, serde_json::to_string(&stats).unwrap_or_default())
        }
        routes::Route::DriftCheck => {
            // Admin-secret gate (the cron path bypasses via scheduled()).
            let provided = req.headers().get("X-Admin-Secret").ok().flatten();
            match (&cfg.drift_admin, provided) {
                (None, _) => json_response(
                    503,
                    r#"{"error":"drift_admin_secret_not_configured"}"#.to_string(),
                ),
                (Some(expected), Some(actual)) if constant_time_eq(expected, &actual) => {
                    let body = run_drift_check(&cfg).await;
                    json_response(200, body)
                }
                (Some(_), _) => {
                    json_response(401, r#"{"error":"bad_admin_secret"}"#.to_string())
                }
            }
        }
        routes::Route::NotFound => json_response(404, r#"{"error":"not_found"}"#.to_string()),
    }
}

#[event(scheduled)]
async fn scheduled(_event: ScheduledEvent, env: Env, ctx: ScheduleContext) {
    // The production drift-revoke story (spec.md:35) — same code path as the
    // manual POST /registry/drift-check.
    let cfg = config(&env);
    ctx.wait_until(async move {
        let _ = run_drift_check(&cfg).await;
    });
}

fn constant_time_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}
