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
//!
//! # Security considerations
//!
//! The registrar is this contract's single trust root; the full trust model
//! (registrar-compromise blast radius, why one registrar is acceptable for
//! v1, revocation-as-safety) lives in `docs/threats.md` §3. In-contract
//! facts that model rests on:
//!
//! - **Authority.** Every write (`verify`, `revoke`, `transfer_registrar`)
//!   funnels through one `only_registrar` check; there is no other caller-
//!   -dependent branch anywhere. `transfer_registrar` is the recovery path
//!   and refuses `address(0)` (a stranded pen would also break the
//!   zero-registrar-means-no-record convention the guard and every consumer
//!   key on).
//! - **No external calls.** The contract makes none — not even token
//!   callbacks — so there is no reentrancy surface and no return-data
//!   decoding to attack; the only host interactions are storage, events,
//!   `msg_sender`, `block_timestamp`, and `native_keccak256`.
//! - **Fail-total reads.** `get_record` never reverts and encodes
//!   "unknown" as the all-zero tuple (`registrar == 0`); consumers are
//!   spec-bound (wire.md) to treat that — or `status > 1` — as no-record,
//!   never as data. Revocation cannot create rows (`NoRecord`), so the
//!   guard's `GUARD_NO_RECORD` flag stays meaningful forever.
//! - **Arithmetic posture.** No arithmetic on user input at all: risk flags
//!   are an opaque U256 copied verbatim, timestamps are u64 stored/never
//!   compared, and the only "computation" is the reason hash.
//! - **Audit trail.** Every write emits its pinned event with the caller's
//!   claims in full — a false record is public the moment it is written,
//!   attributed by `registrar` and `verified_at`/`revoked_at`.

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

#[cfg(test)]
mod fuzz {
    //! Property (fuzz) suite — proptest over the native TestVM host
    //! (plan task 1). Deterministic: every property runs a FIXED seed with
    //! 10_000 cases (the plan's "seeded ≥10k" check), so CI failures
    //! reproduce bit-for-bit anywhere; proptest also shrinks + persists
    //! counterexamples under `proptest-regressions/`.
    //!
    //! One model-based property carries the plan's two registry invariants —
    //! "registrar authority is the only write path" and "record transitions
    //! are total and monotone per the rules": an independent mirror model
    //! predicts, for EVERY op in an arbitrary sequence, exactly which of
    //! {success, NotRegistrar, NoRecord, ZeroRegistrar} must happen and the
    //! exact seven-field state afterwards. Exact-state comparison after
    //! every op subsumes "reverts leave zero state change" (a revert that
    //! mutated anything would diverge from the model), so no separate
    //! delta-scan property exists. The J3 read primitive gets its own
    //! totality property (never reverts, all-zero for unknown tokens).

    use super::*;
    use alloc::collections::BTreeMap;
    use proptest::prelude::*;
    use proptest::test_runner::{Config, RngAlgorithm, TestRng, TestRunner};
    use stylus_sdk::testing::TestVM;

    const REGISTRAR: Address = const { Address::new([0x11; 20]) };
    const OTHER: Address = const { Address::new([0x22; 20]) };
    const TOKEN_A: Address = const { Address::new([0x33; 20]) };
    const TOKEN_B: Address = const { Address::new([0x34; 20]) };
    const UNKNOWN: Address = const { Address::new([0x99; 20]) };
    const NOW: u64 = 1_700_000_000;

    /// Deterministic 10k-case runner: fixed ChaCha seed (proptest's
    /// documented cross-platform-reproducible mode — same algorithm+seed,
    /// same cases, every machine), no entropy. The label folds into the
    /// 32-byte key ChaCha requires, so seed strings stay readable.
    fn runner(seed: &[u8]) -> TestRunner {
        let config = Config {
            cases: 10_000,
            ..Config::default()
        };
        let mut key = [0u8; 32];
        for (i, b) in seed.iter().enumerate() {
            key[i % 32] ^= b;
        }
        TestRunner::new_with_rng(config, TestRng::from_seed(RngAlgorithm::ChaCha, &key))
    }

    fn arb_addr() -> impl Strategy<Value = Address> {
        (any::<u64>(), any::<u64>(), any::<u32>()).prop_map(|(hi, mid, lo)| {
            let mut bytes = [0u8; 20];
            bytes[0..8].copy_from_slice(&hi.to_be_bytes());
            bytes[8..16].copy_from_slice(&mid.to_be_bytes());
            bytes[16..20].copy_from_slice(&lo.to_be_bytes());
            Address::new(bytes)
        })
    }

    /// The mirror model's idea of one record — exactly the seven fields.
    #[derive(Clone, Debug, PartialEq)]
    struct Rec {
        status: u8,
        flags: U256,
        verified_at: u64,
        impl_: Address,
        registrar: Address,
        revoked_at: u64,
        reason: B256,
    }

    fn vm_record(registry: &Registry, token: Address) -> Rec {
        let (status, flags, at, impl_, registrar, revoked, reason) = registry.get_record(token);
        Rec {
            status,
            flags,
            verified_at: at,
            impl_,
            registrar,
            revoked_at: revoked,
            reason,
        }
    }

    /// The whole assertion surface: registrar, every modeled record, the
    /// unknown-token zero convention, and timestamp monotonicity.
    fn assert_vm_matches_model(
        registry: &Registry,
        model_registrar: Address,
        model: &BTreeMap<Address, Rec>,
        now: u64,
    ) {
        assert_eq!(registry.registrar(), model_registrar, "registrar drift");
        for (token, want) in model {
            assert!(&vm_record(registry, *token) == want, "record drift for {token}");
            assert!(want.verified_at <= now, "verified_at regressed the clock");
            assert!(want.revoked_at <= now, "revoked_at regressed the clock");
        }
        let (status, flags, at, impl_, registrar, revoked, reason) = registry.get_record(UNKNOWN);
        assert_eq!(registrar, Address::ZERO, "unknown token must read no-record");
        assert_eq!(status, 0);
        assert_eq!(flags, U256::ZERO);
        assert_eq!((at, revoked), (0, 0));
        assert_eq!(impl_, Address::ZERO);
        assert_eq!(reason, B256::ZERO);
    }

    #[derive(Clone, Debug)]
    enum Op {
        Verify,
        Revoke,
        Transfer,
    }

    #[derive(Clone, Debug)]
    struct Action {
        sender: Address,
        op: Op,
        token: Address,
        flags: U256,
        impl_: Address,
        reason: String,
        new_registrar: Address,
        dt: u64,
    }

    fn action() -> impl Strategy<Value = Action> {
        (
            0u8..3,                    // sender pool index
            0u8..3,                    // op
            prop::bool::ANY,           // token A or B
            any::<u64>(),              // risk flags
            arb_addr(),                // impl pointer
            "[a-z]{1,10}",             // revoke reason
            arb_addr(),                // transfer target
            0u64..1_000,               // clock advance
        )
            .prop_map(
                |(sender_i, op_i, token_b, flags, impl_, reason, new_registrar, dt)| Action {
                    sender: match sender_i {
                        0 => REGISTRAR,
                        1 => OTHER,
                        _ => const { Address::new([0x77; 20]) },
                    },
                    op: match op_i {
                        0 => Op::Verify,
                        1 => Op::Revoke,
                        _ => Op::Transfer,
                    },
                    token: if token_b { TOKEN_B } else { TOKEN_A },
                    flags: U256::from(flags),
                    impl_,
                    reason,
                    new_registrar,
                    dt,
                },
            )
    }

    /// Model-based sequence property — the plan's two registry invariants.
    #[test]
    fn registrar_authority_and_transition_rules_hold_for_arbitrary_sequences() {
        let strat = prop::collection::vec(action(), 1..=8);
        runner(b"vetted-step7-registry-sequences")
            .run(&strat, |actions| {
                let vm = TestVM::default();
                vm.set_sender(REGISTRAR);
                vm.set_block_timestamp(NOW);
                let mut registry = Registry::from(&vm);
                registry.constructor(REGISTRAR).expect("constructor");

                let mut model_registrar = REGISTRAR;
                let mut model: BTreeMap<Address, Rec> = BTreeMap::new();
                let mut now = NOW;

                for a in actions {
                    now += a.dt;
                    vm.set_block_timestamp(now);
                    vm.set_sender(a.sender);
                    let known = model.get(&a.token).cloned();

                    let outcome = match a.op {
                        Op::Verify => registry.verify(a.token, a.flags, a.impl_),
                        Op::Revoke => registry.revoke(a.token, a.reason.clone()),
                        Op::Transfer => registry.transfer_registrar(a.new_registrar),
                    };
                    let is_registrar = a.sender == model_registrar;

                    match (&a.op, is_registrar) {
                        (Op::Verify, true) => {
                            outcome.expect("registrar verify must succeed");
                            model.insert(
                                a.token,
                                Rec {
                                    status: STATUS_VERIFIED,
                                    flags: a.flags,
                                    verified_at: now,
                                    impl_: a.impl_,
                                    registrar: a.sender,
                                    revoked_at: 0,
                                    reason: B256::ZERO,
                                },
                            );
                        }
                        (Op::Verify, false) => assert!(
                            matches!(outcome, Err(RegistryError::NotRegistrar(_))),
                            "non-registrar verify must revert NotRegistrar"
                        ),
                        (Op::Revoke, true) if known.is_some() => {
                            outcome.expect("registrar revoke on a record must succeed");
                            let mut r = known.unwrap();
                            // The revoke rule: status/revoked_at/reason move;
                            // the verified fields are PRESERVED, not cleared.
                            r.status = STATUS_REVOKED;
                            r.revoked_at = now;
                            r.reason = stylus_sdk::crypto::keccak(a.reason.as_bytes());
                            model.insert(a.token, r);
                        }
                        (Op::Revoke, true) => assert!(
                            matches!(outcome, Err(RegistryError::NoRecord(_))),
                            "revoke must not create rows"
                        ),
                        (Op::Revoke, false) => assert!(
                            matches!(outcome, Err(RegistryError::NotRegistrar(_))),
                            "non-registrar revoke must revert NotRegistrar"
                        ),
                        (Op::Transfer, true) if !a.new_registrar.is_zero() => {
                            outcome.expect("registrar transfer must succeed");
                            model_registrar = a.new_registrar;
                        }
                        (Op::Transfer, true) => assert!(
                            matches!(outcome, Err(RegistryError::ZeroRegistrar(_))),
                            "transfer to zero must revert"
                        ),
                        (Op::Transfer, false) => assert!(
                            matches!(outcome, Err(RegistryError::NotRegistrar(_))),
                            "non-registrar transfer must revert NotRegistrar"
                        ),
                    }

                    assert_vm_matches_model(&registry, model_registrar, &model, now);
                }
                Ok(())
            })
            .unwrap();
    }

    /// The J3 read primitive: total, never reverts, all-zero tuple for any
    /// arbitrary unknown token (consumers key on registrar == 0 — wire.md).
    #[test]
    fn get_record_is_total_and_zero_for_arbitrary_unknown_tokens() {
        runner(b"vetted-step7-registry-get-record")
            .run(&arb_addr(), |token| {
                let vm = TestVM::default();
                vm.set_sender(REGISTRAR);
                let registry = Registry::from(&vm);
                let (status, flags, at, impl_, registrar, revoked, reason) =
                    registry.get_record(token);
                assert_eq!(registrar, Address::ZERO);
                assert_eq!(status, 0);
                assert_eq!(flags, U256::ZERO);
                assert_eq!((at, revoked), (0, 0));
                assert_eq!(impl_, Address::ZERO);
                assert_eq!(reason, B256::ZERO);
                Ok(())
            })
            .unwrap();
    }
}
