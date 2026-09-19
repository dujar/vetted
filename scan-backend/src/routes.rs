//! Pure route parsing — the wire contract's method+path+query surface
//! (`packages/shared/wire.md`). Route names and param spellings are
//! load-bearing: the frontend's `FetchScanClient` calls
//! `GET /scan?chainId=&addr=` and `FetchWatchdogSource` calls
//! `GET /watchdog?chainId=` (frontend/src/lib/api.ts).

pub enum Route {
    Health,
    Scan { chain_id: u64, addr: String },
    Watchdog { chain_id: u64 },
    DriftCheck,
    NotFound,
}

/// Parse (method, path, query). Query is the raw string after `?`.
pub fn parse(method: &str, path: &str, query: &str) -> Route {
    match (method, path) {
        ("GET", "/health") => Route::Health,
        ("POST", "/registry/drift-check") => Route::DriftCheck,
        ("GET", "/scan") => {
            let chain_id = query_param(query, "chainId")
                .and_then(|v| v.parse::<u64>().ok());
            let addr = query_param(query, "addr");
            match (chain_id, addr) {
                (Some(chain_id), Some(addr)) => Route::Scan { chain_id, addr },
                _ => Route::NotFound,
            }
        }
        ("GET", "/watchdog") => {
            match query_param(query, "chainId").and_then(|v| v.parse::<u64>().ok()) {
                Some(chain_id) => Route::Watchdog { chain_id },
                None => Route::NotFound,
            }
        }
        _ => Route::NotFound,
    }
}

fn query_param(query: &str, key: &str) -> Option<String> {
    query.split('&').find_map(|pair| {
        let (k, v) = pair.split_once('=')?;
        (k == key).then(|| v.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_route_matches_the_frontend_seam_exactly() {
        let r = parse(
            "GET",
            "/scan",
            "chainId=4663&addr=0x1Cdad396DB64BDa184d5182A97Dd9B3C62100b7D",
        );
        match r {
            Route::Scan { chain_id, addr } => {
                assert_eq!(chain_id, 4663);
                assert_eq!(addr, "0x1Cdad396DB64BDa184d5182A97Dd9B3C62100b7D");
            }
            _ => panic!("expected scan route"),
        }
    }

    #[test]
    fn missing_or_malformed_params_are_not_found() {
        assert!(matches!(parse("GET", "/scan", "chainId=abc&addr=0x1"), Route::NotFound));
        assert!(matches!(parse("GET", "/scan", "addr=0x1"), Route::NotFound));
        assert!(matches!(parse("GET", "/watchdog", ""), Route::NotFound));
    }

    #[test]
    fn watchdog_and_drift_routes() {
        assert!(matches!(parse("GET", "/watchdog", "chainId=421614"), Route::Watchdog { chain_id: 421614 }));
        assert!(matches!(parse("POST", "/registry/drift-check", ""), Route::DriftCheck));
        assert!(matches!(parse("GET", "/registry/drift-check", ""), Route::NotFound));
    }

    #[test]
    fn health_and_404() {
        assert!(matches!(parse("GET", "/health", ""), Route::Health));
        assert!(matches!(parse("GET", "/nope", ""), Route::NotFound));
    }
}
