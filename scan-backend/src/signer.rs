//! Registrar identity + EIP-1559 signing — the compile-verified recipe from
//! .agent-workbench/knowledge/rust-wasm-signing.md, verbatim:
//!   PrivateKeySigner::from_bytes → sign_transaction_sync(&mut TxEip1559) →
//!   into_signed → TxEnvelope → encoded_2718() → eth_sendRawTransaction
//! Imports: `alloy_network::TxSignerSync`, `alloy_eips::eip2718::Encodable2718`.
//! The key comes from the REGISTRAR_KEY worker secret — never committed.
//! Signing needs no RNG (fixed key); getrandom/js is compile-only.

use alloy_consensus::{SignableTransaction, TxEip1559, TxEnvelope};
use alloy_eips::eip2718::Encodable2718;
use alloy_network::TxSignerSync;
use alloy_primitives::{Address, B256, U256};
use alloy_signer_local::PrivateKeySigner;

use crate::hexutil::{self, selectors};

/// ABI-encode `revoke(address,string)` — selector pinned in abi.ts SELECTORS.
pub fn abi_encode_revoke(token: &str, reason: &str) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&hexutil::decode_hex(selectors::REVOKE).unwrap());
    out.extend_from_slice(&hexutil::decode_hex(&hexutil::addr_word(token)).unwrap());
    // string: offset 0x60, length, word-padded data
    out.extend_from_slice(&U256::from(0x60u64).to_be_bytes::<32>());
    let reason_bytes = reason.as_bytes();
    out.extend_from_slice(&U256::from(reason_bytes.len()).to_be_bytes::<32>());
    out.extend_from_slice(reason_bytes);
    // pad the data to a full 32-byte word (ceil), not to a multiple of 32 overall
    let rem = reason_bytes.len() % 32;
    if rem != 0 {
        out.extend(std::iter::repeat_n(0u8, 32 - rem));
    }
    out
}

/// Build the drift-revoke EIP-1559 transaction (to = registry, revoke(token, reason)).
#[allow(clippy::too_many_arguments)] // a plain data builder — each field is load-bearing
pub fn build_revoke_tx(
    chain_id: u64,
    nonce: u64,
    gas_limit: u64,
    max_fee_per_gas: u128,
    max_priority_fee_per_gas: u128,
    registry: Address,
    token: &str,
    reason: &str,
) -> TxEip1559 {
    TxEip1559 {
        chain_id,
        nonce,
        gas_limit,
        max_fee_per_gas,
        max_priority_fee_per_gas,
        to: alloy_primitives::TxKind::Call(registry),
        value: U256::ZERO,
        access_list: Default::default(),
        input: abi_encode_revoke(token, reason).into(),
    }
}

/// Sign per the recipe; returns the raw 2718 bytes for eth_sendRawTransaction.
pub fn sign_tx(signer: &PrivateKeySigner, tx: &mut TxEip1559) -> Result<Vec<u8>, String> {
    let signature = signer
        .sign_transaction_sync(tx)
        .map_err(|e| format!("signing failed: {e}"))?;
    let signed = tx.clone().into_signed(signature);
    let envelope = TxEnvelope::Eip1559(signed);
    Ok(envelope.encoded_2718())
}

/// Parse the REGISTRAR_KEY secret (32-byte hex, 0x optional).
pub fn signer_from_key(key: &str) -> Result<PrivateKeySigner, String> {
    let bytes = hexutil::decode_hex(key).ok_or("registrar key is not hex")?;
    let arr: [u8; 32] = bytes.try_into().map_err(|_| "registrar key must be 32 bytes")?;
    PrivateKeySigner::from_bytes(&B256::from(arr)).map_err(|e| format!("bad key: {e}"))
}

/// The revoke reason carried on-chain — the REVOKED fixture's decoded reason.
pub const DRIFT_REASON: &str = "beacon impl changed";

#[cfg(test)]
mod tests {
    use super::*;

    /// throwaway test key — NOT the registrar key, never a secret.
    const TEST_KEY: &str =
        "0x4646464646464646464646464646464646464646464646464646464646464646";

    #[test]
    fn revoke_encoding_carries_the_pinned_selector() {
        let data = abi_encode_revoke(
            "0x1cdad396db64bda184d5182a97dd9b3c62100b7d",
            DRIFT_REASON,
        );
        assert_eq!(hexutil::encode_hex(&data[..4]), "afd0224b");
        assert_eq!(data.len(), 4 + 32 * 4, "selector + token + offset + len + 1 word");
        // reason is 19 bytes, padded in the last word
        assert_eq!(&data[4 + 96 + 19..], &vec![0u8; 13][..]);
    }

    #[test]
    fn signs_eip1559_and_recovers_the_signer() {
        let signer = signer_from_key(TEST_KEY).unwrap();
        let mut tx = build_revoke_tx(
            4663,
            0,
            400_000,
            2_000_000_000,
            100_000_000,
            "0x5e1500000000000000000000000000000000aa11"
                .parse::<Address>()
                .unwrap(),
            "0x1cdad396db64bda184d5182a97dd9b3c62100b7d",
            DRIFT_REASON,
        );
        let raw = sign_tx(&signer, &mut tx).unwrap();
        assert!(!raw.is_empty());
        // Recover the sender from the signing hash — proves the recipe
        // end-to-end without network (native; wasm compile is CI's wasm32 check).
        let signature = signer.sign_transaction_sync(&mut tx).unwrap();
        let signed = tx.clone().into_signed(signature);
        let recovered = signature
            .recover_address_from_prehash(&tx.signature_hash())
            .unwrap();
        assert_eq!(recovered, signer.address());
        assert_eq!(signed.tx().chain_id, 4663);
    }

    #[test]
    fn bad_keys_are_rejected() {
        assert!(signer_from_key("0x1234").is_err());
        assert!(signer_from_key("zz").is_err());
    }
}
