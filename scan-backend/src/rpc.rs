//! JSON-RPC client over an injected transport — host-independent so native
//! tests exercise the same call path the worker uses. Short in-isolate cache
//! absorbs the public 4663 RPC's rate limit (knowledge robinhood-chain.md:18).
//!
//! ponytail: single-flight elided — the 30s TTL cache absorbs demo-scale
//! concurrent scans of the same token; add a per-key flight merge if the
//! public RPC starts 429ing under load.

use std::collections::HashMap;
use std::sync::Mutex;

use serde_json::Value;

use crate::hexutil;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RpcError {
    /// Transport/parse failure — surfaces as the RPC_RETRYABLE terminal state.
    Transport(String),
    /// The node answered with an error object.
    Rpc(String),
}

/// The result of an `eth_call` — probes must distinguish revert from answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallResult {
    Ok(String),
    Reverted(String),
}

pub trait Transport {
    /// POST `body` to `url`, return the response body text.
    // No Send bound: the Workers runtime is single-threaded and the wasm
    // fetch future is not Send — nothing here spawns.
    fn post(&self, url: &str, body: String) -> impl std::future::Future<Output = Result<String, String>>;
}

/// Monotonic milliseconds — `std::time::Instant` is not implemented on
/// wasm32-unknown-unknown; the Workers runtime exposes Date through the crate.
pub fn now_ms() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        worker::Date::now().as_millis()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

const DEFAULT_TTL_MS: u64 = 30_000;

pub struct RpcClient<T: Transport> {
    pub url: String,
    transport: T,
    cache: Mutex<HashMap<String, (u64, Value)>>,
    ttl_ms: u64,
}

impl<T: Transport> RpcClient<T> {
    pub fn new(url: &str, transport: T) -> Self {
        Self {
            url: url.to_string(),
            transport,
            cache: Mutex::new(HashMap::new()),
            ttl_ms: DEFAULT_TTL_MS,
        }
    }

    /// Test seam — a zero TTL disables caching.
    pub fn with_ttl(url: &str, transport: T, ttl_ms: u64) -> Self {
        Self {
            url: url.to_string(),
            transport,
            cache: Mutex::new(HashMap::new()),
            ttl_ms,
        }
    }

    async fn raw(&self, method: &str, params: Value) -> Result<Value, RpcError> {
        let key = format!("{method}:{params}");
        if self.ttl_ms > 0 {
            if let Some(entry) = self.cache.lock().ok().and_then(|c| c.get(&key).cloned()) {
                if now_ms().saturating_sub(entry.0) < self.ttl_ms {
                    return Ok(entry.1);
                }
            }
        }
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        })
        .to_string();
        let text = self
            .transport
            .post(&self.url, body)
            .await
            .map_err(RpcError::Transport)?;
        // Node error objects are Rpc errors (a revert is an Rpc error here;
        // call() re-classifies the revert case), transport failures stay Transport.
        let v: Value =
            serde_json::from_str(&text).map_err(|e| RpcError::Rpc(format!("malformed rpc body: {e}")))?;
        if let Some(err) = v.get("error") {
            return Err(RpcError::Rpc(err.to_string()));
        }
        let parsed: Value = v
            .get("result")
            .cloned()
            .ok_or_else(|| RpcError::Rpc("no result field".into()))?;
        // Only successful reads are cached — a cached error would resurface
        // with the wrong classification (e.g. a cached revert).
        if self.ttl_ms > 0 {
            if let Ok(mut cache) = self.cache.lock() {
                cache.insert(key, (now_ms(), parsed.clone()));
            }
        }
        Ok(parsed)
    }

    pub async fn get_code(&self, addr: &str) -> Result<Vec<u8>, RpcError> {
        let v = self
            .raw("eth_getCode", serde_json::json!([addr, "latest"]))
            .await?;
        let hex = v.as_str().unwrap_or("0x");
        hexutil::decode_hex(hex).ok_or(RpcError::Rpc("malformed code hex".into()))
    }

    /// Raw 32-byte storage word (0x + 64 hex).
    pub async fn get_storage_at(&self, addr: &str, slot: &str) -> Result<String, RpcError> {
        let v = self
            .raw("eth_getStorageAt", serde_json::json!([addr, slot, "latest"]))
            .await?;
        v.as_str()
            .map(str::to_string)
            .ok_or(RpcError::Rpc("malformed storage word".into()))
    }

    pub async fn call(&self, to: &str, data: &str) -> Result<CallResult, RpcError> {
        match self
            .raw(
                "eth_call",
                serde_json::json!([{"to": to, "data": data}, "latest"]),
            )
            .await
        {
            Ok(v) => match v.as_str() {
                Some(hex) => Ok(CallResult::Ok(hex.to_string())),
                None => Ok(CallResult::Reverted(String::new())),
            },
            // Nodes report reverts as error objects, usually with empty data
            // (the calibrated blocklist probe reverts with empty data —
            // calibration_4663.json).
            Err(RpcError::Rpc(msg)) if msg.contains("revert") => Ok(CallResult::Reverted(msg)),
            Err(e) => Err(e),
        }
    }

    /// Log objects for one address + ordered topic filter (None = wildcard).
    pub async fn get_logs(
        &self,
        address: &str,
        topics: Vec<Option<String>>,
    ) -> Result<Vec<Value>, RpcError> {
        let topics_json: Vec<Value> = topics
            .into_iter()
            .map(|t| t.map(Value::String).unwrap_or(Value::Null))
            .collect();
        let v = self
            .raw(
                "eth_getLogs",
                serde_json::json!([{
                    "address": address,
                    "topics": topics_json,
                    "fromBlock": "0x1",
                    "toBlock": "latest",
                }]),
            )
            .await?;
        v.as_array().cloned().ok_or(RpcError::Rpc("malformed logs".into()))
    }

    /// Gas price hex string, for the drift-revoke fee fields.
    pub async fn gas_price(&self) -> Result<u128, RpcError> {
        let v = self.raw("eth_gasPrice", serde_json::json!([])).await?;
        let hex = v.as_str().ok_or(RpcError::Rpc("malformed gas price".into()))?;
        u128::from_str_radix(hexutil::strip0x(hex), 16)
            .map_err(|e| RpcError::Rpc(format!("bad gas price: {e}")))
    }

    pub async fn transaction_count(&self, addr: &str) -> Result<u64, RpcError> {
        let v = self
            .raw("eth_getTransactionCount", serde_json::json!([addr, "pending"]))
            .await?;
        let hex = v.as_str().ok_or(RpcError::Rpc("malformed nonce".into()))?;
        u64::from_str_radix(hexutil::strip0x(hex), 16)
            .map_err(|e| RpcError::Rpc(format!("bad nonce: {e}")))
    }

    pub async fn estimate_gas(&self, to: &str, data: &str, from: &str) -> Result<u64, RpcError> {
        let v = self
            .raw(
                "eth_estimateGas",
                serde_json::json!([{"to": to, "data": data, "from": from}]),
            )
            .await?;
        let hex = v.as_str().ok_or(RpcError::Rpc("malformed gas estimate".into()))?;
        u64::from_str_radix(hexutil::strip0x(hex), 16)
            .map_err(|e| RpcError::Rpc(format!("bad gas estimate: {e}")))
    }

    pub async fn send_raw_transaction(&self, raw: &[u8]) -> Result<String, RpcError> {
        let hex = format!("0x{}", hexutil::encode_hex(raw));
        let v = self
            .raw("eth_sendRawTransaction", serde_json::json!([hex]))
            .await?;
        v.as_str()
            .map(str::to_string)
            .ok_or(RpcError::Rpc("no tx hash".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Replays canned bodies keyed by the method in the request.
    struct MockTransport {
        respond: Box<dyn Fn(String) -> Result<String, String> + Send + Sync>,
        calls: Mutex<u32>,
    }

    impl MockTransport {
        fn jsonrpc(result: serde_json::Value) -> Self {
            Self {
                respond: Box::new(move |body| {
                    let req: Value = serde_json::from_str(&body).unwrap();
                    Ok(serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": req["id"],
                        "result": result.clone(),
                    })
                    .to_string())
                }),
                calls: Mutex::new(0),
            }
        }
    }

    impl Transport for MockTransport {
        async fn post(&self, _url: &str, body: String) -> Result<String, String> {
            *self.calls.lock().unwrap() += 1;
            (self.respond)(body)
        }
    }

    #[tokio::test]
    async fn caches_identical_reads_within_ttl() {
        let transport = MockTransport::jsonrpc(serde_json::json!("0xdeadbeef"));
        let client = RpcClient::new("http://test", transport);
        assert_eq!(
            client.call("0xabc", "0x5c975abb").await.unwrap(),
            CallResult::Ok("0xdeadbeef".into())
        );
        // second identical call is served from cache
        let _ = client.call("0xabc", "0x5c975abb").await.unwrap();
        assert_eq!(*client.transport.calls.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn zero_ttl_ignores_cache() {
        let transport = MockTransport::jsonrpc(serde_json::json!("0x00"));
        let client = RpcClient::with_ttl("http://test", transport, 0);
        let _ = client.get_code("0xabc").await.unwrap();
        let _ = client.get_code("0xabc").await.unwrap();
        assert_eq!(*client.transport.calls.lock().unwrap(), 2);
    }

    #[tokio::test]
    async fn rpc_error_object_is_an_error() {
        struct ErrTransport;
        impl Transport for ErrTransport {
            async fn post(&self, _url: &str, _body: String) -> Result<String, String> {
                Ok(r#"{"jsonrpc":"2.0","id":1,"error":{"code":3,"message":"execution reverted"}}"#.into())
            }
        }
        let client: RpcClient<ErrTransport> = RpcClient::new("http://test", ErrTransport);
        match client.call("0xabc", "0xfbac3951").await {
            Ok(CallResult::Reverted(msg)) => assert!(msg.contains("execution reverted")),
            other => panic!("expected revert classification, got {other:?}"),
        }
    }
}
