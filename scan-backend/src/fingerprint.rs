//! EIP-1967/beacon resolution + the depth-boundary signature matcher
//! (discipline rule 3, spec.md:29).
//!
//! Resolution mirrors the guard's pinned path (coordinator decision,
//! 2026-09-19): the beacon is extracted from the forwarder bytecode by
//! shape/offset, and the implementation comes from a STATICCALL to the beacon's
//! `implementation()` getter. Blocklist + impl checks probe the BEACON, not the
//! proxy — the blocklist state lives there (spike findings, Revised note 1).
//!
//! The shipped signature-match list (composed jointly with step 6 per Revised
//! note 3 — the live P/CRM tokens hit it, a faithful replica hits it, twins
//! miss it; proxy-codehash byte-equality deliberately NOT required):
//!   S1 forwarder shape — proxy bytecode carries `implementation()` (0x5c60da1b)
//!   S2 beacon slot set + impl slot empty (beacon-proxy layout)
//!   S3 beacon extracts from the bytecode and answers implementation() != 0
//!   S4 paused() answers on the token proxy (calibrated target)
//!   S5 isBlocked(address) answers on the resolved beacon (calibrated target)

use crate::hexutil::{
    decode_abi_bool, decode_hex, encode_hex, norm_addr, selectors, word_to_addr,
};
use crate::rpc::{CallResult, RpcClient, Transport};

/// What the resolution pass established about one contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    pub beacon_slot_set: bool,
    pub impl_slot_empty: bool,
    /// The beacon extracted from the forwarder bytecode (guard-pinned path),
    /// cross-checked against the beacon slot when the slot is set.
    pub beacon: Option<String>,
    /// The beacon's current `implementation()`.
    pub impl_addr: Option<String>,
    /// keccak256 of the implementation's code (empty when not fetched).
    pub impl_codehash: Option<String>,
    pub signature_match: bool,
}

/// PUSH-literal candidates from forwarder-shaped bytecode, extracted by
/// shape/offset (the guard-pinned path). The genuine 283-byte forwarder loads
/// its beacon as a solc immutable — a PUSH32 (0x7f) with the address in the
/// low 20 bytes, emitted immediately before the `implementation()` STATICCALL
/// push — so the extraction scans backwards from each PUSH4
/// `<implementation()>` occurrence and reports the embedded literal. Other
/// forwarder layouts that keep a plain PUSH20 there resolve the same way.
pub fn beacon_candidates(code: &[u8]) -> Vec<String> {
    let sel = decode_hex(selectors::IMPL).unwrap();
    let zero = [0u8; 20];
    let mut out = Vec::new();
    // Every occurrence of PUSH4 <implementation()> — the STATICCALL site.
    if code.len() >= 5 {
        for i in 0..=code.len() - 5 {
            if code[i] != 0x63 || &code[i + 1..i + 5] != sel.as_slice() {
                continue;
            }
            // Scan the small window before the push: the immutable/address load.
            let start = i.saturating_sub(64);
            for j in (start..i).rev() {
                let cand: Option<&[u8]> = match code[j] {
                    // PUSH32: address is the low 20 bytes of the word
                    0x7f if j + 33 <= code.len() && j + 33 <= i => Some(&code[j + 13..j + 33]),
                    // PUSH20: address is the immediate
                    0x73 if j + 21 <= code.len() && j + 21 <= i => Some(&code[j + 1..j + 21]),
                    _ => None,
                };
                if let Some(c) = cand {
                    if c != zero.as_slice() {
                        let addr = format!("0x{}", encode_hex(c));
                        if let Some(norm) = norm_addr(&addr) {
                            if !out.contains(&norm) {
                                out.push(norm);
                            }
                        }
                    }
                }
            }
        }
    }
    out
}

/// The five checks, over one resolved layout + probe outcomes (answered flags).
pub fn signature_match(layout: &Layout, paused_answered: bool, blocklist_answered: bool) -> bool {
    layout.beacon_slot_set
        && layout.impl_slot_empty
        && layout.beacon.is_some()
        && layout
            .impl_addr
            .as_ref()
            .map(|a| a.chars().any(|c| c != '0'))
            .unwrap_or(false)
        && paused_answered
        && blocklist_answered
}

/// The async resolution pass: slots, bytecode-shape beacon extraction,
/// beacon.implementation(), implementation codehash. Every read is fallible —
/// failures degrade the corresponding check rather than failing the scan.
pub async fn resolve<T: Transport>(rpc: &RpcClient<T>, addr: &str, code: &[u8]) -> Layout {
    // S2: slot layout (node-side storage reads — proven live on 4663).
    // ponytail: sequential slot reads — a join! needs the futures crate; two
    // extra RTTs are within the scan budget (the TTL cache absorbs repeats).
    let impl_slot = rpc.get_storage_at(addr, crate::hexutil::EIP1967_IMPL_SLOT).await;
    let beacon_slot = rpc.get_storage_at(addr, crate::hexutil::EIP1967_BEACON_SLOT).await;
    let impl_slot_empty = impl_slot
        .ok()
        .and_then(|w| word_to_addr(&w))
        .map(|a| a == "0x0000000000000000000000000000000000000000")
        .unwrap_or(false);
    let beacon_slot_addr = beacon_slot.ok().and_then(|w| word_to_addr(&w));
    let beacon_slot_set = beacon_slot_addr
        .as_ref()
        .map(|a| a.as_str() != "0x0000000000000000000000000000000000000000")
        .unwrap_or(false);

    // S1/S3: bytecode-shape extraction (guard-pinned), validated by a live
    // staticcall; the slot read is the cross-check/fallback.
    let mut beacon: Option<String> = None;
    for cand in beacon_candidates(code) {
        if beacon_slot_set && beacon_slot_addr.as_deref() != Some(cand.as_str()) {
            continue; // mismatched literal — not the beacon
        }
        if answers_implementation(rpc, &cand).await {
            beacon = Some(cand);
            break;
        }
    }
    if beacon.is_none() && beacon_slot_set {
        // Fallback: a beacon the forwarder shape missed (e.g. a different
        // forwarder layout) — the slot still names it.
        if let Some(slot_addr) = beacon_slot_addr.as_deref() {
            if answers_implementation(rpc, slot_addr).await {
                beacon = beacon_slot_addr.clone();
            }
        }
    }

    // S3: implementation() + codehash.
    let mut impl_addr = None;
    if let Some(b) = beacon.as_ref() {
        if let Ok(CallResult::Ok(hex)) = rpc.call(b, selectors::IMPL).await {
            impl_addr = decode_hex(&hex)
                .filter(|b| b.len() == 32)
                .map(|b| format!("0x{}", encode_hex(&b[12..])));
        }
    }

    let mut impl_codehash = None;
    if let Some(imp) = &impl_addr {
        if let Ok(imp_code) = rpc.get_code(imp).await {
            impl_codehash = Some(format!(
                "0x{}",
                encode_hex(alloy_primitives::keccak256(&imp_code).as_slice())
            ));
        }
    }

    let mut layout = Layout {
        beacon_slot_set,
        impl_slot_empty,
        beacon,
        impl_addr,
        impl_codehash,
        signature_match: false,
    };
    // S4/S5 land in lib.rs (probe outcomes); signature_match is finalized there
    // via `finalize_signature_match`.
    layout.signature_match = false;
    layout
}

/// Does `addr` answer `implementation()` with a non-zero address? (S3)
pub async fn answers_implementation<T: Transport>(rpc: &RpcClient<T>, addr: &str) -> bool {
    matches!(
        rpc.call(addr, selectors::IMPL).await,
        Ok(CallResult::Ok(hex)) if decode_hex(&hex).map(|b| b.iter().any(|&x| x != 0)).unwrap_or(false)
    )
}

/// Probes at the calibrated targets (S4/S5). Returns the answered flags.
pub async fn probe<T: Transport>(
    rpc: &RpcClient<T>,
    proxy: &str,
    beacon: Option<&str>,
    buyer: &str,
) -> (bool, bool) {
    // S4: paused() → the token PROXY.
    let paused = matches!(
        rpc.call(proxy, selectors::PAUSED).await,
        Ok(CallResult::Ok(hex)) if decode_abi_bool(&hex) == Some(false) || decode_abi_bool(&hex) == Some(true)
    );
    // S5: isBlocked(buyer) → the resolved BEACON (reverts on proxy/impl).
    let blocklist = match beacon {
        Some(b) => matches!(rpc.call(b, &crate::hexutil::call_addr(selectors::BLOCKLIST, buyer)).await,
            Ok(CallResult::Ok(_))),
        None => false,
    };
    (paused, blocklist)
}

/// Finalize S4/S5 into the layout's signature flag.
pub fn finalize_signature_match(layout: &mut Layout, paused_answered: bool, blocklist_answered: bool) {
    layout.signature_match = signature_match(layout, paused_answered, blocklist_answered);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_fixture(name: &str) -> Vec<u8> {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/bytecode")
            .join(format!("{name}.hex"));
        let raw = std::fs::read_to_string(path).expect("bytecode fixture readable");
        decode_hex(raw.trim()).expect("fixture parses as hex")
    }

    #[test]
    fn fixture_codehashes_match_the_calibration_evidence() {
        // Tripwire: the committed fixtures ARE the recorded 4663 evidence
        // (spike/evidence/calibration_4663.json). Any corruption shows here.
        let k = |bytes: &[u8]| format!("0x{}", encode_hex(alloy_primitives::keccak256(bytes).as_slice()));
        assert_eq!(
            k(&load_fixture("p_proxy")),
            "0x6c1fdd40002dcb440c7fff6a84171404d279ccb057803b65826f7546acd65630"
        );
        assert_eq!(
            k(&load_fixture("beacon")),
            "0x8b465c0b53a2ba499566e9b4ca67d8c90ed6131743df806a570d156956a7e90e"
        );
        assert_eq!(
            k(&load_fixture("shared_impl")),
            "0xdc07e86ee482f99641bdafb9a0d772846b167401e094d90a666b94dbdcd1eec7"
        );
    }

    #[test]
    fn extracts_the_shared_beacon_from_the_genuine_forwarder() {
        let code = load_fixture("p_proxy");
        assert_eq!(code.len(), 283, "the recorded 283-byte forwarder");
        // The beacon is the solc immutable pushed right before the
        // implementation() STATICCALL — exactly one candidate, no metadata
        // false positives (the window search never reaches the swarm hash).
        assert_eq!(
            beacon_candidates(&code),
            vec!["0xe10b6f6b275de231345c20d14ab812db62151b00".to_string()]
        );
    }

    #[test]
    fn plain_contracts_yield_no_candidates() {
        // A non-pattern contract (recorded twin fixture) — shape gate fails.
        let code = load_fixture("usdg_twin");
        assert!(beacon_candidates(&code).is_empty());
    }

    #[test]
    fn signature_requires_all_five_checks() {
        let full = Layout {
            beacon_slot_set: true,
            impl_slot_empty: true,
            beacon: Some("0xe10b6f6b275de231345c20d14ab812db62151b00".into()),
            impl_addr: Some("0xb35490d6f9163de4f80d88dc75c3516eb64c5ae2".into()),
            impl_codehash: None,
            signature_match: false,
        };
        assert!(signature_match(&full, true, true));
        // An OZ-style proxy with a NON-empty impl slot fails the layout (S2).
        let oz = Layout { impl_slot_empty: false, ..full.clone() };
        assert!(!signature_match(&oz, true, true));
        // A beacon-less forwarder fails (S3).
        let no_beacon = Layout { beacon: None, ..full.clone() };
        assert!(!signature_match(&no_beacon, true, true));
        // Unanswered probes fail (S4/S5) — a twin that merely embeds a beacon.
        assert!(!signature_match(&full, true, false));
        assert!(!signature_match(&full, false, true));
    }
}
