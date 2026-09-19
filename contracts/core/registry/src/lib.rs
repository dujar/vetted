//! The Canonical Registry — Vetted's on-chain ground truth for stock-token
//! verification (spec.md Scope 2). The registrar writes a verification record
//! per token after checking the issuer's live list; the contract stores and
//! exposes, it does not judge (rule-engine use of records is step 4's
//! business). Revocation on beacon upgrade is the feature, not a wart: the
//! backend's cron revokes on implementation drift, and the guard (step 3,
//! contracts/core/guard) refuses swapped tokens on exactly that record.
//!
//! Wording discipline (spec.md:35): this is the Canonical Registry, its rows
//! are verification records, and the single writer is the registrar. The word
//! "attestation" is banned everywhere.
//!
//! Surface (pinned byte-for-byte by `packages/shared/abi.ts` — SELECTORS):
//! - `getRecord(address)` → 0x617fba04 — the J3 primitive: callable from
//!   other contracts (journeys.md:43). A token with no record returns the
//!   zero tuple (registrar = address(0)); consumers treat zero-registrar or
//!   unknown status as no-record, never guessed (wire.md).
//! - `verify(address,uint256,address)` → 0x73c7cf61 — registrar-only.
//! - `revoke(address,string)` → 0xafd0224b — registrar-only.
//! Events (pinned by `packages/shared/events.ts` — topic0 hashes):
//! - `Verify(address,uint256,address)`, `Revoke(address,string)`.
//!
//! `record.reason` stores keccak256(utf8(reason)); the full human-readable
//! text rides the Revoke event, which is what J3 renders (verify.md loose
//! end 3). Param/field names are cosmetic in the ABI — the selectors and
//! event topics carry no names, and the Rust keyword `impl` is stored as
//! `impl_addr`/`implAddr`.

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

// The SDK's storage macros expand to `alloc` paths.
#[macro_use]
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use stylus_sdk::{
    alloy_primitives::{aliases::{U8, U64}, Address, B256, U256},
    alloy_sol_types::sol,
    prelude::*,
};

/// Record status u8 → wire value 0 (wire.md): the canonical verification.
pub const STATUS_VERIFIED: u8 = 0;
/// Record status u8 → wire value 1 (wire.md): revoked since verification.
pub const STATUS_REVOKED: u8 = 1;

sol! {
    /// Emitted when the registrar writes (or re-affirms) a verification
    /// record. topic0 = keccak256("Verify(address,uint256,address)").
    #[derive(Debug)]
    event Verify(address indexed token, uint256 riskFlags, address indexed implAddr);

    /// Emitted on revocation; `reason` carries the FULL human-readable text
    /// (the record's bytes32 field keeps only keccak256(utf8(reason))).
    /// topic0 = keccak256("Revoke(address,string)").
    #[derive(Debug)]
    event Revoke(address indexed token, string reason);

    /// Caller is not the registrar (registrar-only writes).
    #[derive(Debug)]
    error NotRegistrar(address caller, address registrar);
    /// `revoke` on a token with no verification record — revocation cannot
    /// create rows, or the guard's GUARD_NO_RECORD flag could never fire.
    #[derive(Debug)]
    error NoRecord(address token);
    /// Registrar transfers to address(0) would strand writes forever and
    /// break the zero-registrar-means-no-record convention.
    #[derive(Debug)]
    error ZeroRegistrar();
}

/// Registry revert set. Encodes as standard Solidity error data.
#[derive(Debug, SolidityError)]
pub enum RegistryError {
    NotRegistrar(NotRegistrar),
    NoRecord(NoRecord),
    ZeroRegistrar(ZeroRegistrar),
}

sol_storage! {
    #[entrypoint]
    pub struct Registry {
        /// The single registrar — every write must come from this address.
        /// Set once at construction; transfer via `transferRegistrar`.
        address registrar;
        /// token → verification record. An entry that was never written
        /// reads as the all-zero tuple; `registrar == 0` marks "no record".
        mapping(address => Record) records;
    }

    /// Seven-field verification record (wire.md) — field order matches
    /// `packages/shared/abi.ts` REGISTRY_ABI and `packages/shared/types.ts`
    /// RegistryRecord exactly.
    pub struct Record {
        uint8 status;
        uint256 risk_flags;
        uint64 verified_at;
        address impl_addr;
        address registrar;
        uint64 revoked_at;
        bytes32 reason;
    }
}

#[public]
impl Registry {
    /// Deploys the Canonical Registry with its one registrar.
    /// Registrar-only writes start immediately; `transferRegistrar` is the
    /// recovery path (plan task 1).
    #[constructor]
    pub fn constructor(&mut self, registrar: Address) -> Result<(), RegistryError> {
        if registrar.is_zero() {
            return Err(ZeroRegistrar {}.into());
        }
        self.registrar.set(registrar);
        Ok(())
    }

    /// The registrar's address — J3's criteria panel shows it
    /// (journeys.md:42) so integrators know whose writes to trust.
    pub fn registrar(&self) -> Address {
        self.registrar.get()
    }

    /// The J3 primitive: read a token's verification record, callable from
    /// other contracts (journeys.md:43). Never reverts — a token with no
    /// record returns the zero tuple; consumers treat `registrar == 0` or a
    /// status outside {0, 1} as no-record (wire.md), never guessed.
    ///
    /// Returned tuple order (matches abi.ts REGISTRY_ABI):
    /// `(status, riskFlags, verifiedAt, impl, registrar, revokedAt, reason)`.
    pub fn get_record(&self, token: Address) -> (u8, U256, u64, Address, Address, u64, B256) {
        let r = self.records.get(token);
        (
            r.status.get().to::<u8>(),
            r.risk_flags.get(),
            r.verified_at.get().to::<u64>(),
            r.impl_addr.get(),
            r.registrar.get(),
            r.revoked_at.get().to::<u64>(),
            r.reason.get(),
        )
    }

    /// Write (or re-affirm) a token's verification record: status VERIFIED
    /// with the registrar's current flags and implementation pointer.
    /// Registrar-only. Re-verifying a REVOKED record reinstates it — the
    /// registrar re-checked the issuer's live list, and the registry stores,
    /// it does not judge (plan task 2).
    pub fn verify(
        &mut self,
        token: Address,
        risk_flags: U256,
        impl_: Address,
    ) -> Result<(), RegistryError> {
        self.only_registrar()?;
        let sender = self.vm().msg_sender();
        let now = self.vm().block_timestamp();

        let mut r = self.records.setter(token);
        r.status.set(U8::from(STATUS_VERIFIED));
        r.risk_flags.set(risk_flags);
        r.verified_at.set(U64::from(now));
        r.impl_addr.set(impl_);
        r.registrar.set(sender);
        r.revoked_at.set(U64::from(0));
        r.reason.set(B256::ZERO);

        self.vm().log(Verify {
            token,
            riskFlags: risk_flags,
            implAddr: impl_,
        });
        Ok(())
    }

    /// Revoke a token's verification record with a human-readable reason
    /// (e.g. "beacon impl changed 2026-09-06 — record stale"). Registrar-only.
    /// The record must exist — revocation never creates rows, or the guard's
    /// `GUARD_NO_RECORD` flag could never fire. The bytes32 record field gets
    /// keccak256(utf8(reason)); the full text rides the Revoke event.
    pub fn revoke(&mut self, token: Address, reason: String) -> Result<(), RegistryError> {
        self.only_registrar()?;
        let r = self.records.get(token);
        if r.registrar.get().is_zero() {
            return Err(NoRecord { token }.into());
        }

        let reason_hash = self.vm().native_keccak256(reason.as_bytes());
        let now = self.vm().block_timestamp();
        let mut r = self.records.setter(token);
        r.status.set(U8::from(STATUS_REVOKED));
        r.revoked_at.set(U64::from(now));
        r.reason.set(reason_hash);

        self.vm().log(Revoke { token, reason });
        Ok(())
    }

    /// Recovery path for the registrar key (plan task 1): hand the single
    /// writer role to a new address. Registrar-only; refuses address(0).
    pub fn transfer_registrar(&mut self, new_registrar: Address) -> Result<(), RegistryError> {
        self.only_registrar()?;
        if new_registrar.is_zero() {
            return Err(ZeroRegistrar {}.into());
        }
        self.registrar.set(new_registrar);
        Ok(())
    }
}

// Internal auth helper — kept out of `#[public]`, which ABI-exports every
// fn in its block (a leaked `onlyRegistrar()` selector is surface pollution).
impl Registry {
    /// Revert unless the caller is the registrar.
    fn only_registrar(&self) -> Result<(), RegistryError> {
        let registrar = self.registrar.get();
        let sender = self.vm().msg_sender();
        if sender != registrar {
            return Err(NotRegistrar { caller: sender, registrar }.into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use stylus_sdk::alloy_primitives::b256;
    use stylus_sdk::testing::TestVM;

    const REGISTRAR: Address = const { Address::new([0x11; 20]) };
    const OTHER: Address = const { Address::new([0x22; 20]) };
    const TOKEN: Address = const { Address::new([0x33; 20]) };
    const IMPL: Address = const { Address::new([0x44; 20]) };
    const NOW: u64 = 1_700_000_000;

    /// Fresh registry with REGISTRAR installed and the clock at NOW.
    fn vm_with_registry() -> (TestVM, Registry) {
        let vm = TestVM::default();
        vm.set_sender(REGISTRAR);
        vm.set_block_timestamp(NOW);
        let mut registry = Registry::from(&vm);
        registry.constructor(REGISTRAR).expect("constructor");
        (vm, registry)
    }

    #[test]
    fn constructor_rejects_zero_registrar() {
        let vm = TestVM::default();
        vm.set_sender(REGISTRAR);
        let mut registry = Registry::from(&vm);
        assert!(matches!(
            registry.constructor(Address::ZERO),
            Err(RegistryError::ZeroRegistrar(_))
        ));
    }

    #[test]
    fn constructor_sets_registrar_readable() {
        let (_, registry) = vm_with_registry();
        assert_eq!(registry.registrar(), REGISTRAR);
    }

    #[test]
    fn get_record_on_unknown_token_is_zero_tuple() {
        // The no-record convention: zero registrar, never a revert — the
        // shipped frontend decodes registrar == 0 as null (registry.ts).
        let (_, registry) = vm_with_registry();
        let (status, risk_flags, verified_at, impl_, registrar, revoked_at, reason) =
            registry.get_record(TOKEN);
        assert_eq!(status, STATUS_VERIFIED); // default u8 — zero-registrar disambiguates
        assert_eq!(risk_flags, U256::ZERO);
        assert_eq!(verified_at, 0);
        assert_eq!(impl_, Address::ZERO);
        assert_eq!(registrar, Address::ZERO);
        assert_eq!(revoked_at, 0);
        assert_eq!(reason, B256::ZERO);
    }

    #[test]
    fn verify_writes_all_seven_fields_and_emits() {
        let (vm, mut registry) = vm_with_registry();
        registry.verify(TOKEN, U256::from(0xdead_u64), IMPL).expect("verify");

        let (status, risk_flags, verified_at, impl_, registrar, revoked_at, reason) =
            registry.get_record(TOKEN);
        assert_eq!(status, STATUS_VERIFIED);
        assert_eq!(risk_flags, U256::from(0xdead_u64));
        assert_eq!(verified_at, NOW);
        assert_eq!(impl_, IMPL);
        assert_eq!(registrar, REGISTRAR);
        assert_eq!(revoked_at, 0);
        assert_eq!(reason, B256::ZERO);

        let logs = vm.get_emitted_logs();
        assert_eq!(logs.len(), 1);
        let (topics, _data) = &logs[0];
        assert_eq!(topics.len(), 3); // topic0 + two indexed
        assert_eq!(
            topics[0],
            stylus_sdk::crypto::keccak("Verify(address,uint256,address)")
        );
    }

    #[test]
    fn non_registrar_writes_revert_without_state_change() {
        let (vm, mut registry) = vm_with_registry();
        registry.verify(TOKEN, U256::ZERO, IMPL).expect("verify");

        vm.set_sender(OTHER);
        assert!(matches!(
            registry.verify(TOKEN, U256::from(1u8), OTHER),
            Err(RegistryError::NotRegistrar(_))
        ));
        assert!(matches!(
            registry.revoke(TOKEN, "x".into()),
            Err(RegistryError::NotRegistrar(_))
        ));
        assert!(matches!(
            registry.transfer_registrar(OTHER),
            Err(RegistryError::NotRegistrar(_))
        ));

        vm.set_sender(REGISTRAR);
        let (status, _, _, impl_, _, _, _) = registry.get_record(TOKEN);
        assert_eq!(status, STATUS_VERIFIED);
        assert_eq!(impl_, IMPL); // untouched by the failed writes
        assert_eq!(registry.registrar(), REGISTRAR);
    }

    #[test]
    fn revoke_marks_record_and_hashes_reason() {
        let (vm, mut registry) = vm_with_registry();
        registry.verify(TOKEN, U256::ZERO, IMPL).expect("verify");

        const REASON: &str = "beacon impl changed 2026-09-06 — record stale";
        vm.set_block_timestamp(NOW + 100);
        registry.revoke(TOKEN, REASON.into()).expect("revoke");

        let (status, _, _, _, _, revoked_at, reason_hash) = registry.get_record(TOKEN);
        assert_eq!(status, STATUS_REVOKED);
        assert_eq!(revoked_at, NOW + 100);
        assert_eq!(reason_hash, stylus_sdk::crypto::keccak(REASON.as_bytes()));

        // The Revoke event carries the full text (J3 renders it) under the
        // pinned topic0 = keccak256("Revoke(address,string)").
        let logs = vm.get_emitted_logs();
        assert_eq!(logs.len(), 2);
        let (topics, data) = &logs[1];
        assert_eq!(topics.len(), 2); // topic0 + indexed token
        assert_eq!(topics[0], stylus_sdk::crypto::keccak("Revoke(address,string)"));
        let text = core::str::from_utf8(data).unwrap_or("");
        assert!(text.contains("record stale"), "full reason must ride the event");
    }

    #[test]
    fn revoke_on_missing_record_reverts() {
        // Revocation cannot create rows — or GUARD_NO_RECORD could never fire.
        let (_, mut registry) = vm_with_registry();
        assert!(matches!(
            registry.revoke(TOKEN, "no such token".into()),
            Err(RegistryError::NoRecord(_))
        ));
        let (_, _, _, _, registrar, _, _) = registry.get_record(TOKEN);
        assert_eq!(registrar, Address::ZERO); // still no record
    }

    #[test]
    fn re_verify_after_revoke_reinstates() {
        let (_, mut registry) = vm_with_registry();
        registry.verify(TOKEN, U256::ZERO, IMPL).expect("verify");
        registry.revoke(TOKEN, "stale".into()).expect("revoke");
        registry
            .verify(TOKEN, U256::from(7u8), IMPL)
            .expect("re-verify");

        let (status, risk_flags, _, _, _, revoked_at, reason) = registry.get_record(TOKEN);
        assert_eq!(status, STATUS_VERIFIED);
        assert_eq!(risk_flags, U256::from(7u8));
        assert_eq!(revoked_at, 0);
        assert_eq!(reason, B256::ZERO);
    }

    #[test]
    fn transfer_registrar_hands_over_writes() {
        let (vm, mut registry) = vm_with_registry();
        registry.transfer_registrar(OTHER).expect("transfer");
        assert_eq!(registry.registrar(), OTHER);

        // Old key lost the pen; new key holds it.
        vm.set_sender(REGISTRAR);
        assert!(matches!(
            registry.verify(TOKEN, U256::ZERO, IMPL),
            Err(RegistryError::NotRegistrar(_))
        ));
        vm.set_sender(OTHER);
        registry.verify(TOKEN, U256::ZERO, IMPL).expect("verify");
        let (status, _, _, _, registrar, _, _) = registry.get_record(TOKEN);
        assert_eq!(status, STATUS_VERIFIED);
        assert_eq!(registrar, OTHER);
    }

    #[test]
    fn transfer_registrar_refuses_zero() {
        let (_, mut registry) = vm_with_registry();
        assert!(matches!(
            registry.transfer_registrar(Address::ZERO),
            Err(RegistryError::ZeroRegistrar(_))
        ));
        assert_eq!(registry.registrar(), REGISTRAR); // unchanged
    }

    #[test]
    fn pinned_selectors_match_abi_ts() {
        // Byte-pin against packages/shared/abi.ts SELECTORS — drift breaks
        // here, not on-chain (mirrors the vitest recompute tripwire).
        let cases: [(&str, u32); 3] = [
            ("getRecord(address)", 0x617fba04),
            ("verify(address,uint256,address)", 0x73c7cf61),
            ("revoke(address,string)", 0xafd0224b),
        ];
        for (sig, pinned) in cases {
            let word = stylus_sdk::crypto::keccak(sig.as_bytes());
            let selector = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
            assert_eq!(selector, pinned, "selector drift on {sig}");
        }
    }

    #[test]
    fn pinned_event_topics_match_events_ts() {
        // Byte-pin against packages/shared/events.ts REGISTRY_EVENT_TOPICS.
        let verify = stylus_sdk::crypto::keccak("Verify(address,uint256,address)");
        let revoke = stylus_sdk::crypto::keccak("Revoke(address,string)");
        assert_eq!(
            B256::from(verify),
            b256!("3e797825af25f16433592602e044447c6902a26f7c31d1918940c56b824c7db3")
        );
        assert_eq!(
            B256::from(revoke),
            b256!("2fa80445a7995a05a1a47457227da064b86a578212322a7cd41a235d469749a1")
        );
    }
}
