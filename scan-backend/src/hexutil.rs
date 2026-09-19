//! Hex + ABI word helpers — pure, host-independent (native tests run on CI).

/// Calibrated probe/function selectors — the Rust mirror of the shared
/// `PROBE_SELECTORS` + `SELECTORS` (packages/shared/abi.ts) and the dispatcher
/// selectors recorded in spike/evidence/calibration_4663.json. The Rust side
/// cannot import TS (verify loose end 8): the tripwire test below asserts the
/// bytes against keccak so a transcription error cannot ship silently.
pub mod selectors {
    /// `paused()` — probe the token PROXY (blocklist state lives elsewhere).
    pub const PAUSED: &str = "0x5c975abb";
    /// `isBlocked(address)` — probe the resolved BEACON; the same selector
    /// reverts on the proxy/impl (live-verified, spike findings).
    pub const BLOCKLIST: &str = "0xfbac3951";
    /// `implementation()` — resolved against the BEACON.
    pub const IMPL: &str = "0x5c60da1b";
    /// `uid()` — on-chain id; equals the issuer registry row id for genuine
    /// tokens (strongest single check, calibration_4663.json).
    pub const UID: &str = "0xf514ce36";
    /// `name()`
    pub const NAME: &str = "0x06fdde03";
    /// `symbol()`
    pub const SYMBOL: &str = "0x95d89b41";
    /// `getRecord(address)` — must equal abi.ts SELECTORS.getRecord.
    pub const GET_RECORD: &str = "0x617fba04";
    /// `revoke(address,string)` — must equal abi.ts SELECTORS.revoke.
    pub const REVOKE: &str = "0xafd0224b";
}

/// Registry event topic0 pins — the Rust mirror of packages/shared/events.ts
/// (step 3's pin; the Rust worker cannot import TS, same tripwire pattern as
/// `selectors`). Consumers: REVOKED's revocation-tx extraction (Revoke) and
/// the drift-watch's record enumeration (Verify).
pub mod events {
    /// `Verify(address indexed token, uint256 riskFlags, address indexed impl)`
    pub const VERIFY_TOPIC0: &str =
        "0x3e797825af25f16433592602e044447c6902a26f7c31d1918940c56b824c7db3";
    /// `Revoke(address indexed token, string reason)`
    pub const REVOKE_TOPIC0: &str =
        "0x2fa80445a7995a05a1a47457227da064b86a578212322a7cd41a235d469749a1";
}

/// EIP-1967 implementation slot (0x360894…82bbc).
pub const EIP1967_IMPL_SLOT: &str =
    "0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc";
/// EIP-1967 beacon slot (0xa3f0a…3d50).
pub const EIP1967_BEACON_SLOT: &str =
    "0xa3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50";

pub fn strip0x(s: &str) -> &str {
    s.strip_prefix("0x").unwrap_or(s)
}

pub fn decode_hex(s: &str) -> Option<Vec<u8>> {
    let s = strip0x(s);
    if !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

pub fn encode_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

/// Normalize an address to `0x` + 40 lowercase hex; None when malformed.
pub fn norm_addr(s: &str) -> Option<String> {
    let s = strip0x(s);
    if s.len() != 40 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    Some(format!("0x{}", s.to_ascii_lowercase()))
}

/// `0xc30d…e812` — the wire's short-address form (IMPOSTOR fixture).
pub fn short_addr(s: &str) -> String {
    let s = strip0x(s).to_ascii_lowercase();
    if s.len() < 8 {
        return format!("0x{s}");
    }
    format!("0x{}…{}", &s[..4], &s[s.len() - 4..])
}

/// Last 20 bytes of a 32-byte storage word -> normalized address.
pub fn word_to_addr(word: &str) -> Option<String> {
    let s = strip0x(word);
    if s.len() != 64 {
        return None;
    }
    norm_addr(&s[24..])
}

/// Address as a right-padded 32-byte ABI word (with 0x), lowercased.
pub fn addr_word(addr: &str) -> String {
    format!("0x{}{}", "0".repeat(24), strip0x(addr).to_ascii_lowercase())
}

/// eth_call data prefix: selector + one address argument.
pub fn call_addr(selector: &str, addr: &str) -> String {
    format!("{selector}{}", &addr_word(addr)[2..])
}

/// Decode an ABI-encoded `string` return (offset, len, data) — enough for
/// name()/symbol() on well-formed ERC-20s; None on anything unexpected.
pub fn decode_abi_string(hex: &str) -> Option<String> {
    let bytes = decode_hex(hex)?;
    if bytes.len() < 64 {
        return None;
    }
    let len_word = &bytes[32..64];
    let len = u64::from_be_bytes(len_word[24..32].try_into().ok()?) as usize;
    if bytes.len() < 64 + len {
        return None;
    }
    String::from_utf8(bytes[64..64 + len].to_vec()).ok()
}

/// Decode a bool return (last byte of the word).
pub fn decode_abi_bool(hex: &str) -> Option<bool> {
    let bytes = decode_hex(hex)?;
    if bytes.len() < 32 {
        return None;
    }
    match bytes[31] {
        0 => Some(false),
        _ => Some(true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selectors_are_keccak_derived() {
        let k = |sig: &str| {
            let h = alloy_primitives::keccak256(sig.as_bytes());
            format!("0x{}", encode_hex(&h[..4]))
        };
        assert_eq!(selectors::PAUSED, k("paused()"));
        assert_eq!(selectors::BLOCKLIST, k("isBlocked(address)"));
        assert_eq!(selectors::IMPL, k("implementation()"));
        assert_eq!(selectors::UID, k("uid()"));
        assert_eq!(selectors::NAME, k("name()"));
        assert_eq!(selectors::SYMBOL, k("symbol()"));
        assert_eq!(selectors::GET_RECORD, k("getRecord(address)"));
        assert_eq!(selectors::REVOKE, k("revoke(address,string)"));
    }

    #[test]
    fn selectors_match_shared_abi_ts() {
        // The TS tripwire (packages/shared round-trip test) recomputes
        // SELECTORS with viem; these literals must never drift from it.
        assert_eq!(selectors::GET_RECORD, "0x617fba04");
        assert_eq!(selectors::REVOKE, "0xafd0224b");
        // And the calibrated PROBE_SELECTORS pair (abi.ts:61-62).
        assert_eq!(selectors::PAUSED, "0x5c975abb");
        assert_eq!(selectors::BLOCKLIST, "0xfbac3951");
    }

    #[test]
    fn event_topics_recompute_from_the_canonical_signatures() {
        let k = |sig: &str| {
            let h = alloy_primitives::keccak256(sig.as_bytes());
            format!("0x{}", encode_hex(h.as_slice()))
        };
        assert_eq!(
            events::VERIFY_TOPIC0,
            k("Verify(address,uint256,address)")
        );
        assert_eq!(events::REVOKE_TOPIC0, k("Revoke(address,string)"));
    }

    #[test]
    fn addr_helpers_round_trip() {
        let a = "0x1Cdad396DB64BDa184d5182A97Dd9B3C62100b7D";
        assert_eq!(norm_addr(a).unwrap(), "0x1cdad396db64bda184d5182a97dd9b3c62100b7d");
        assert!(norm_addr("0x1234").is_none());
        assert_eq!(short_addr(a), "0x1cda…0b7d");
        let word = addr_word(a);
        assert_eq!(word_to_addr(&word).unwrap(), norm_addr(a).unwrap());
        assert_eq!(
            call_addr(selectors::BLOCKLIST, a),
            format!("{}0000000000000000000000001cdad396db64bda184d5182a97dd9b3c62100b7d", selectors::BLOCKLIST)
        );
    }

    #[test]
    fn abi_string_decode() {
        // name() of P, captured live 2026-09-19 ("Everpure • Robinhood Token").
        let hex = "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000001c457665727075726520e280a220526f62696e686f6f6420546f6b656e00000000";
        assert_eq!(decode_abi_string(hex).unwrap(), "Everpure \u{2022} Robinhood Token");
        assert!(decode_abi_string("0x").is_none());
    }

    #[test]
    fn abi_bool_decode() {
        assert_eq!(decode_abi_bool("0x0000000000000000000000000000000000000000000000000000000000000000"), Some(false));
        assert_eq!(decode_abi_bool("0x0000000000000000000000000000000000000000000000000000000000000001"), Some(true));
    }
}
