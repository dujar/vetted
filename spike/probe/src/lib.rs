//! Week-1 spike probe (step 2 tasks 2–4): does Stylus on Robinhood Chain 4663
//! support the three host capabilities the spec's gate needs (spec.md:50–52)?
//!   (a) staticcall an arbitrary address          → probe_paused
//!   (b) remote code read (extcodecopy-equiv)     → probe_code / code_bytes
//!   (c) EIP-1967 implementation-slot resolution  → probe_resolve
//! plus the seven-field registry-record stub read (verify.md loose end 3) and
//! `run_suite` — the whole probe suite in ONE tx so a single receipt's
//! gasUsed is the spec'd ≤200,000 number.
//!
//! All calls are infallible views: every remote outcome is returned as an
//! ok-flag, never a revert, so an empty-code target (the unfunded-chain
//! mirror, verify.md loose end 2) reads "ok, nothing there" instead of
//! failing the tx.
//!
//! Selector/slot constants are CALIBRATED against the live genuine tokens
//! (spike/evidence/calibration_4663.json, task 5) — they are the bytes the
//! merge writes into `PROBE_SELECTORS` in packages/shared/abi.ts.

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

// The SDK's storage macros expand to `alloc` paths.
#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use stylus_sdk::{
    alloy_primitives::{
        aliases::{U8, U64},
        b256, Address, B256, U256,
    },
    call,
    prelude::*,
    stylus_core::calls::{CallContext, StaticCallContext},
    stylus_core::host::Host,
};

/// paused() on the shared implementation — live-verified on 4663.
pub const SEL_PAUSED: u32 = 0x5c975abb;
/// isBlocked(address) — CALIBRATION SURPRISE: the per-address blocklist state
/// lives on the BEACON, not the impl/proxy. The same selector exists on the
/// impl but REVERTS there; on the beacon it answers (live-verified, task 5).
pub const SEL_IS_BLOCKED: u32 = 0xfbac3951;
/// implementation() — beacon → impl (the beacon staticcall of gate task 2c).
pub const SEL_IMPLEMENTATION: u32 = 0x5c60da1b;
/// extsload(address,bytes32) — slot-read helper interface. Stylus has no
/// remote-storage hostio, and no EVM this spike targets is known to ship
/// EIP-2330's EXTSLOAD opcode (0x5c) — the 42-byte canary decides at deploy
/// time. Until a chain ships it, criterion (c)'s evidence is node-side
/// (eth_getStorageAt on the live tokens — see calibration_4663.json); this
/// interface is what a helper uses if a target chain ever provides one.
pub const SEL_EXTSLOAD: u32 = 0x25a130c3;

/// EIP-1967 implementation slot.
pub const IMPL_SLOT: B256 =
    b256!("360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc");
/// EIP-1967 beacon slot.
pub const BEACON_SLOT: B256 =
    b256!("a3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50");

/// Static-call context: u64::MAX is clipped to remaining gas by the hostio,
/// i.e. "forward everything, keep the refund" — the standard probe posture.
struct AllGas;
impl CallContext for AllGas {
    fn gas(&self) -> u64 {
        u64::MAX
    }
}
impl StaticCallContext for AllGas {}

sol_storage! {
    #[entrypoint]
    pub struct Probe {
        // Seven-field registry record stub (verify.md loose end 3) — the
        // getRecord tuple shape of packages/shared/abi.ts REGISTRY_ABI, in
        // field order. `impl` is a Rust keyword → impl_addr (spike-only
        // divergence; step 3's real registry names it in ABI terms).
        Record record;
    }

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
impl Probe {
    /// Seed the record stub (setup — NOT part of the measured suite).
    pub fn seed_record(
        &mut self,
        status: u8,
        risk_flags: U256,
        verified_at: u64,
        impl_addr: Address,
        registrar: Address,
        revoked_at: u64,
        reason: B256,
    ) {
        let r = &mut self.record;
        r.status.set(U8::from(status));
        r.risk_flags.set(risk_flags);
        r.verified_at.set(U64::from(verified_at));
        r.impl_addr.set(impl_addr);
        r.registrar.set(registrar);
        r.revoked_at.set(U64::from(revoked_at));
        r.reason.set(reason);
    }

    /// The seven-field record read, measured alone (suite component 1).
    pub fn read_record(&self) -> (u8, U256, u64, Address, Address, u64, B256) {
        let r = &self.record;
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

    /// (a) staticcall paused() on an arbitrary address.
    /// Empty-code target: success + empty returndata → (true, false) — the
    /// defined 421614-mirror behavior (verify.md loose end 2). Any other
    /// shape: (false, false).
    pub fn probe_paused(&self, token: Address) -> (bool, bool) {
        call_bool(self.vm(), token, SEL_PAUSED, &[])
    }

    /// (b) remote code read — size + hash (the extcodecopy-equivalent host
    /// fns; reading 11 KB through the hostio is for forensics only, so the
    /// measured suite carries size+hash, not the bytes).
    pub fn probe_code(&self, a: Address) -> (U256, B256) {
        let h = self.vm();
        (U256::from(h.code_size(a)), h.code_hash(a))
    }

    /// (b) full remote code read, sliceable — the literal extcodecopy
    /// equivalent. Caller picks the window; Host::code() has no offset args.
    pub fn code_bytes(&self, a: Address, offset: U256, len: U256) -> Vec<u8> {
        let code = self.vm().code(a);
        let off = offset.to::<usize>();
        let ln = len.to::<usize>();
        let end = off.saturating_add(ln).min(code.len());
        if off >= end {
            Vec::new()
        } else {
            code[off..end].to_vec()
        }
    }

    /// (c) part 1 — EIP-1967 beacon-slot read via the extsload helper.
    pub fn read_beacon_slot(&self, proxy: Address, extsload: Address) -> (Address, bool) {
        let (w, ok) = slot(self.vm(), extsload, proxy, BEACON_SLOT);
        (Address::from_word(B256::from(w)), ok)
    }

    /// (c) part 2 — beacon.implementation() staticcall.
    pub fn resolve_impl(&self, beacon: Address) -> (Address, bool) {
        call_addr(self.vm(), beacon, SEL_IMPLEMENTATION, &[])
    }

    /// (c) composed: beacon slot + implementation() → (beacon, impl, ok).
    /// Classic no-beacon proxies: read the impl slot instead — caller's
    /// choice of helper call; this spike's targets are beacon proxies.
    pub fn probe_resolve(
        &self,
        proxy: Address,
        extsload: Address,
    ) -> (Address, Address, bool) {
        let (beacon, ok) = self.read_beacon_slot(proxy, extsload);
        if !ok {
            return (Address::ZERO, Address::ZERO, false);
        }
        let (impl_, ok2) = self.resolve_impl(beacon);
        (beacon, impl_, ok2)
    }

    /// Calibrated blocklist probe: isBlocked(buyer) against the BEACON
    /// (state lives there — see SEL_IS_BLOCKED).
    pub fn probe_blocklist(&self, beacon: Address, buyer: Address) -> (bool, bool) {
        call_bool(self.vm(), beacon, SEL_IS_BLOCKED, &enc_addr(buyer))
    }

    /// THE SUITE — one tx, one receipt gasUsed ≤ 200,000 = the spec number
    /// (spec.md:51): record read + paused + blocklist + impl-vs-record.
    /// Packed result (bit = meaning):
    ///   0 paused-call ok        1 paused value
    ///   2 blocklist-call ok     3 blocked value
    ///   4 slot read ok          5 impl resolved (nonzero)
    ///   6 impl == record.impl   7 code read ok (size>0)
    /// bits 16..48 = token code size in bytes.
    pub fn run_suite(
        &self,
        token: Address,
        beacon: Address,
        buyer: Address,
        extsload: Address,
    ) -> U256 {
        // 1: record read (value discarded — the read itself is the cost)
        let (_, _, _, record_impl, _, _, _) = self.read_record();
        // 2: paused()
        let (paused_ok, paused) = self.probe_paused(token);
        // 3: blocklist on the beacon
        let (bl_ok, blocked) = self.probe_blocklist(beacon, buyer);
        // 4: impl resolution + comparison
        let (_, impl_, slot_ok) = self.probe_resolve(token, extsload);
        let resolved = impl_ != Address::ZERO;
        let matches = resolved && impl_ == record_impl;
        // 5: remote code presence
        let (size, hash) = self.probe_code(token);

        let mut bits: u64 = (paused_ok as u64)
            | ((paused as u64) << 1)
            | ((bl_ok as u64) << 2)
            | ((blocked as u64) << 3)
            | ((slot_ok as u64) << 4)
            | ((resolved as u64) << 5)
            | ((matches as u64) << 6)
            | (((size > U256::ZERO && hash != B256::ZERO) as u64) << 7);
        bits |= (size & U256::from(0xffff_ffffu64)).to::<u64>() << 16;
        U256::from(bits)
    }
}

// ---- calldata/returndata helpers (pure, host-independent) ----

/// STATICCALL `sel ++ args`; decode a 32-byte word.
/// (ok=true, None) = success but empty/odd returndata (empty-code target).
fn call_word<H: Host + ?Sized>(
    h: &H,
    to: Address,
    sel: u32,
    args: &[u8],
) -> (bool, Option<[u8; 32]>) {
    let mut data = Vec::with_capacity(4 + args.len());
    data.extend_from_slice(&sel.to_be_bytes());
    data.extend_from_slice(args);
    match call::static_call(h, AllGas, to, &data) {
        Ok(ret) if ret.len() == 32 => {
            let mut w = [0u8; 32];
            w.copy_from_slice(&ret);
            (true, Some(w))
        }
        Ok(_) => (true, None),
        Err(_) => (false, None),
    }
}

fn call_bool<H: Host + ?Sized>(h: &H, to: Address, sel: u32, args: &[u8]) -> (bool, bool) {
    match call_word(h, to, sel, args) {
        (true, Some(w)) => (true, w[31] != 0),
        (true, None) => (true, false),
        (false, _) => (false, false),
    }
}

fn call_addr<H: Host + ?Sized>(h: &H, to: Address, sel: u32, args: &[u8]) -> (Address, bool) {
    match call_word(h, to, sel, args) {
        (true, Some(w)) => (Address::from_word(B256::from(w)), true),
        _ => (Address::ZERO, false),
    }
}

/// STATICCALL extsload(target, key) on the helper → 32-byte slot value.
fn slot<H: Host + ?Sized>(
    h: &H,
    extsload: Address,
    target: Address,
    key: B256,
) -> ([u8; 32], bool) {
    // calldata = sel ++ word(target) ++ word(key) — extsload(address,bytes32)
    let mut cd = Vec::with_capacity(4 + 64);
    cd.extend_from_slice(&SEL_EXTSLOAD.to_be_bytes());
    cd.extend_from_slice(&enc_addr(target));
    cd.extend_from_slice(key.as_slice());
    match call::static_call(h, AllGas, extsload, &cd) {
        Ok(ret) if ret.len() == 32 => {
            let mut w = [0u8; 32];
            w.copy_from_slice(&ret);
            (w, true)
        }
        _ => ([0u8; 32], false),
    }
}

fn enc_addr(a: Address) -> [u8; 32] {
    let mut w = [0u8; 32];
    w[12..].copy_from_slice(a.as_slice());
    w
}

#[cfg(test)]
mod test {
    use super::*;
    use stylus_sdk::testing::TestVM;

    const IMPL: Address = const { Address::new([0x11; 20]) };
    const BEACON: Address = const { Address::new([0x22; 20]) };
    const TOKEN: Address = const { Address::new([0x33; 20]) };
    const BUYER: Address = const { Address::new([0x44; 20]) };
    const HELPER: Address = const { Address::new([0x55; 20]) };

    fn word(a: Address) -> Vec<u8> {
        enc_addr(a).to_vec()
    }

    #[test]
    fn record_roundtrip_seven_fields() {
        let vm = TestVM::default();
        let mut probe = Probe::from(&vm);
        probe.seed_record(
            1,
            U256::from(0xdead_u64),
            1_700_000_000u64,
            IMPL,
            BUYER,
            0,
            B256::repeat_byte(0xab),
        );
        let (s, rf, va, im, reg, rev, reason) = probe.read_record();
        assert_eq!(s, 1);
        assert_eq!(rf, U256::from(0xdead_u64));
        assert_eq!(va, 1_700_000_000);
        assert_eq!(im, IMPL);
        assert_eq!(reg, BUYER);
        assert_eq!(rev, 0);
        assert_eq!(reason, B256::repeat_byte(0xab));
    }

    #[test]
    fn remote_code_read_size_hash_and_slice() {
        let vm = TestVM::default();
        let probe = Probe::from(&vm);
        let code: Vec<u8> = (0u8..=255).cycle().take(600).collect();
        vm.set_code(TOKEN, code.clone());
        let (size, hash) = probe.probe_code(TOKEN);
        assert_eq!(size, U256::from(600u64));
        assert!(hash != B256::ZERO);
        // extcodecopy-equivalent slice
        let mid = probe.code_bytes(TOKEN, U256::from(596u64), U256::from(100u64));
        assert_eq!(mid.len(), 4);
        assert_eq!(mid, code[596..600]);
        // empty account
        let (size0, hash0) = probe.probe_code(BUYER);
        assert_eq!(size0, U256::ZERO);
        assert_eq!(hash0, B256::ZERO);
    }

    #[test]
    fn paused_staticcall_three_shapes() {
        let vm = TestVM::default();
        let probe = Probe::from(&vm);
        let sel = SEL_PAUSED.to_be_bytes().to_vec();
        // true
        let mut t = vec![0u8; 32];
        t[31] = 1;
        vm.mock_static_call(TOKEN, sel.clone(), Ok(t));
        assert_eq!(probe.probe_paused(TOKEN), (true, true));
        // empty returndata = empty-code target (421614 mirror)
        vm.mock_static_call(TOKEN, sel.clone(), Ok(alloc::vec![]));
        assert_eq!(probe.probe_paused(TOKEN), (true, false));
        // revert
        vm.mock_static_call(TOKEN, sel, Err(alloc::vec![]));
        assert_eq!(probe.probe_paused(TOKEN), (false, false));
    }

    #[test]
    fn blocklist_staticcall_encodes_buyer() {
        let vm = TestVM::default();
        let probe = Probe::from(&vm);
        let mut cd = SEL_IS_BLOCKED.to_be_bytes().to_vec();
        cd.extend_from_slice(&word(BUYER));
        let mut t = vec![0u8; 32];
        t[31] = 1;
        vm.mock_static_call(BEACON, cd, Ok(t));
        assert_eq!(probe.probe_blocklist(BEACON, BUYER), (true, true));
    }

    #[test]
    fn read_beacon_slot_decodes_word() {
        let vm = TestVM::default();
        let probe = Probe::from(&vm);
        let mut cd = SEL_EXTSLOAD.to_be_bytes().to_vec();
        cd.extend_from_slice(&word(TOKEN));
        cd.extend_from_slice(BEACON_SLOT.as_slice());
        vm.mock_static_call(HELPER, cd, Ok(word(BEACON)));
        let (beacon, ok) = probe.read_beacon_slot(TOKEN, HELPER);
        assert!(ok);
        assert_eq!(beacon, BEACON);
    }

    #[test]
    fn resolve_impl_decodes_address_and_revert() {
        let vm = TestVM::default();
        let probe = Probe::from(&vm);
        vm.mock_static_call(BEACON, SEL_IMPLEMENTATION.to_be_bytes().to_vec(), Ok(word(IMPL)));
        assert_eq!(probe.resolve_impl(BEACON), (IMPL, true));
        // revert → unresolved, not a panic
        let vm2 = TestVM::default();
        let probe2 = Probe::from(&vm2);
        vm2.mock_static_call(BEACON, SEL_IMPLEMENTATION.to_be_bytes().to_vec(), Err(alloc::vec![]));
        assert_eq!(probe2.resolve_impl(BEACON), (Address::ZERO, false));
    }

    #[test]
    fn probe_resolve_composition_plumbing() {
        // stylus-test 0.10.9 quirk: static_call_contract writes only the
        // returndata LENGTH; the BYTES every staticcall returns come from
        // state.return_data = the LAST-registered mock. A two-staticcall
        // composition therefore cannot see two distinct words in native
        // tests — this asserts both calls issue and succeed; the distinct
        // values are covered by read_beacon_slot/resolve_impl above, and
        // end-to-end on-chain by the deployed probe against live 4663.
        let vm = TestVM::default();
        let probe = Probe::from(&vm);
        let mut cd = SEL_EXTSLOAD.to_be_bytes().to_vec();
        cd.extend_from_slice(&word(TOKEN));
        cd.extend_from_slice(BEACON_SLOT.as_slice());
        vm.mock_static_call(HELPER, cd, Ok(word(BEACON)));
        vm.mock_static_call(BEACON, SEL_IMPLEMENTATION.to_be_bytes().to_vec(), Ok(word(IMPL)));
        let (b, i, ok) = probe.probe_resolve(TOKEN, HELPER);
        assert!(ok);
        assert_eq!(b, IMPL); // both staticcalls read the shared buffer
        assert_eq!(i, IMPL);
    }

    #[test]
    fn suite_packs_expected_bits() {
        // NB: under the stylus-test quirk (see probe_resolve_composition_
        // _plumbing) every staticcall here reads bytes = word(IMPL) — the
        // last-registered mock. That makes paused/blocklist true and
        // resolves beacon=impl=IMPL, which still lands every ok/value bit;
        // per-call byte fidelity is covered by the split tests above.
        let vm = TestVM::default();
        let mut probe = Probe::from(&vm);
        // seed record with impl = IMPL → suite's impl-vs-record must match
        probe.seed_record(1, U256::ZERO, 1, IMPL, BUYER, 0, B256::ZERO);
        // paused → true
        let mut t = vec![0u8; 32];
        t[31] = 1;
        vm.mock_static_call(TOKEN, SEL_PAUSED.to_be_bytes().to_vec(), Ok(t.clone()));
        // blocklist on beacon → false but ok
        let mut cd = SEL_IS_BLOCKED.to_be_bytes().to_vec();
        cd.extend_from_slice(&word(BUYER));
        vm.mock_static_call(BEACON, cd, Ok(vec![0u8; 32]));
        // slot read → beacon; implementation() → IMPL
        let mut cd3 = SEL_EXTSLOAD.to_be_bytes().to_vec();
        cd3.extend_from_slice(&word(TOKEN));
        cd3.extend_from_slice(BEACON_SLOT.as_slice());
        vm.mock_static_call(HELPER, cd3, Ok(word(BEACON)));
        vm.mock_static_call(BEACON, SEL_IMPLEMENTATION.to_be_bytes().to_vec(), Ok(word(IMPL)));
        // code present
        vm.set_code(TOKEN, alloc::vec![0x60, 0x00, 0xf3]);

        let r = probe.run_suite(TOKEN, BEACON, BUYER, HELPER);
        assert_eq!(r & U256::from(0xffu64), U256::from(0b1111_1111u64));
        assert_eq!(r >> 16, U256::from(3u64)); // code size 3
    }

    #[test]
    fn suite_on_empty_mirror_is_not_a_revert() {
        // 421614-with-nothing-deployed world: every remote probe degrades,
        // the suite still returns (verify.md loose end 2).
        let vm = TestVM::default();
        let mut probe = Probe::from(&vm);
        probe.seed_record(1, U256::ZERO, 1, IMPL, BUYER, 0, B256::ZERO);
        vm.mock_static_call(TOKEN, SEL_PAUSED.to_be_bytes().to_vec(), Ok(alloc::vec![]));
        let mut cd = SEL_IS_BLOCKED.to_be_bytes().to_vec();
        cd.extend_from_slice(&word(BUYER));
        vm.mock_static_call(BEACON, cd, Err(alloc::vec![]));
        let mut cd3 = SEL_EXTSLOAD.to_be_bytes().to_vec();
        cd3.extend_from_slice(&word(TOKEN));
        cd3.extend_from_slice(BEACON_SLOT.as_slice());
        vm.mock_static_call(HELPER, cd3, Ok(alloc::vec![0u8; 32]));
        vm.mock_static_call(BEACON, SEL_IMPLEMENTATION.to_be_bytes().to_vec(), Err(alloc::vec![]));

        let r = probe.run_suite(TOKEN, BEACON, BUYER, HELPER);
        let bits = (r & U256::from(0xffu64)).to::<u64>();
        assert_eq!(bits & 0b1, 1); // paused ok (empty returndata)
        assert_eq!(bits & 0b10, 0); // paused false
        assert_eq!(bits & 0b100, 0); // blocklist reverted
        // slot read: ok status comes from the mock key, but the quirk routes
        // the BYTES from the last-registered (Err, empty) mock → len ≠ 32 →
        // the slot decode degrades exactly like an empty-code target would.
        assert_eq!(bits & 0b1011_0000, 0b0000_0000);
        assert_eq!(bits & 0b1000_0000, 0); // no code
    }
}
