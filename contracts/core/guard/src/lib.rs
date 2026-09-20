//! The Guarded Swap — an escrow-style swap that refuses red-flagged stock
//! tokens at execution time (spec.md Scope 3). This is the enforcement no
//! incumbent has: a buyer's funds only move when BOTH tokens pass the
//! Canonical Registry's record checks and the live probes, in a fixed order,
//! with deterministic reverts (journeys.md:32).
//!
//! Flow (plan task 3, Revised note 7 — MM-key counterparty, no-arg execute):
//! 1. `commit(tokenIn, tokenOut, amountIn, minOut)` — the buyer escrows
//!    `amountIn` of tokenIn (transferFrom buyer → guard).
//! 2. The counterparty (project MM key, set at construction) pre-approves the
//!    guard on tokenOut — that standing allowance IS the fill.
//! 3. `execute()` — the buyer runs ALL checks on BOTH tokens before any
//!    movement, then pulls the fill (`transferFrom counterparty → buyer` for
//!    exactly `minOut`) and releases the escrow (`transfer guard →
//!    counterparty` for `amountIn`).
//!
//! The five deterministic red flags revert with byte-exact strings
//! (pinned by `packages/shared/abi.ts` GUARD_REVERT_REASONS, rendered
//! verbatim by the frontend):
//!   `GUARD_NO_RECORD` — no Canonical Registry record for a swap token;
//!   `GUARD_RECORD_REVOKED` — record status REVOKED;
//!   `GUARD_PAUSED` — `paused()` (0x5c975abb) true on the token PROXY;
//!   `GUARD_BLOCKLISTED` — `isBlocked(buyer)` (0xfbac3951) true on the
//!     token's RESOLVED BEACON (the genuine blocklist state lives there, not
//!     on the proxy/impl — spike calibration, plan Revised note 2);
//!   `GUARD_IMPL_MISMATCH` — resolved implementation ≠ record.impl.
//!
//! Beacon resolution is from-contract (plan Revised note 2): the genuine
//! forwarder embeds its beacon as a PUSH32 zero-padded word followed by the
//! `implementation()` selector (0x5c60da1b) — that calibrated SHAPE is the
//! extraction anchor (never codehash equality; step-6 replicas embed a
//! different beacon at the same shape). From-contract EIP-1967 slot reads
//! are impossible on stock EVMs (no EXTSLOAD; spike findings). Extraction
//! failure degrades: the blocklist and impl checks are SKIPPED for that
//! token — heuristics never revert (spec.md:28); an impl check only reverts
//! when resolution succeeds and mismatches.
//!
//! Safety: every error propagates — nothing is swallowed — so EVM atomicity
//! keeps every revert path at zero movement. The five flags revert strictly
//! before any movement. Storage follows checks-effects-interactions: the
//! order is deactivated before the settlement calls, so reentrant tokens
//! find no active escrow.
//!
//! Surface (pinned byte-for-byte by `packages/shared/abi.ts` — SELECTORS):
//! `commit(address,address,uint256,uint256)` → 0x498ab631, `execute()` →
//! 0x61461954. Internal escrow invariants revert with dedicated
//! `Escrow*` errors — the five pinned strings stay reserved for red flags.

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

// The SDK's storage macros expand to `alloc` paths.
#[macro_use]
extern crate alloc;

use alloc::string::ToString;
use alloc::vec::Vec;

use stylus_sdk::{
    alloy_primitives::{Address, B256, U256},
    alloy_sol_types::{sol, Revert},
    call,
    prelude::*,
    stylus_core::calls::{errors::Error as CallError, CallContext, MutatingCallContext, StaticCallContext},
};

// ---- calibrated probe surface (packages/shared/abi.ts PROBE_SELECTORS and
// spike/probe — never re-derive these from strings at runtime) ----

/// `paused()` — probed on the token PROXY (answered live on 4663).
pub const SEL_PAUSED: u32 = 0x5c975abb;
/// `isBlocked(address)` — probed on the RESOLVED BEACON (blocklist state
/// lives there; the same selector reverts on the proxy/impl — live-verified).
pub const SEL_IS_BLOCKED: u32 = 0xfbac3951;
/// `implementation()` — beacon → implementation pointer.
pub const SEL_IMPLEMENTATION: u32 = 0x5c60da1b;
/// `getRecord(address)` on the Canonical Registry (abi.ts SELECTORS).
pub const SEL_GET_RECORD: u32 = 0x617fba04;
/// ERC-20 `transferFrom(address,address,uint256)`.
pub const SEL_TRANSFER_FROM: u32 = 0x23b872dd;
/// ERC-20 `transfer(address,uint256)`.
pub const SEL_TRANSFER: u32 = 0xa9059cbb;

/// Record status u8 values (wire.md).
pub const STATUS_VERIFIED: u8 = 0;
pub const STATUS_REVOKED: u8 = 1;

sol! {
    /// commit with amountIn == 0 — nothing to escrow, nothing to guard.
    #[derive(Debug)]
    error EscrowAmountZero();
    /// commit with an order already escrowed — overwriting would strand the
    /// first escrow.
    #[derive(Debug)]
    error EscrowOrderActive(address buyer);
    /// execute with no active order for the caller.
    #[derive(Debug)]
    error EscrowNoOrder(address buyer);
    /// An ERC-20 movement returned `false` (non-reverting token). Tokens
    /// that revert honestly surface as `TokenCallFailed` with their raw
    /// revert data carried losslessly in `revertData`.
    #[derive(Debug)]
    error TokenTransferFailed(address token);
    /// An ERC-20 movement reverted — the token's raw revert data rides in
    /// `revertData` (stylus error enums require `SolError` variants, so raw
    /// bubbling is impossible; the bytes survive untouched inside).
    #[derive(Debug)]
    error TokenCallFailed(address token, bytes revertData);
}

/// Guard revert set. `Revert` carries the five pinned red-flag strings as
/// classic `Error(string)` data; `Call` passes foreign revert data through
/// untouched (EVM atomicity then keeps the whole tx at zero movement).
#[derive(Debug, SolidityError)]
pub enum GuardError {
    Revert(Revert),
    EscrowAmountZero(EscrowAmountZero),
    EscrowOrderActive(EscrowOrderActive),
    EscrowNoOrder(EscrowNoOrder),
    TokenTransferFailed(TokenTransferFailed),
    TokenCallFailed(TokenCallFailed),
}

/// The five deterministic red flags — byte-exact strings (journeys.md:32).
const GUARD_NO_RECORD: &str = "GUARD_NO_RECORD";
const GUARD_RECORD_REVOKED: &str = "GUARD_RECORD_REVOKED";
const GUARD_PAUSED: &str = "GUARD_PAUSED";
const GUARD_BLOCKLISTED: &str = "GUARD_BLOCKLISTED";
const GUARD_IMPL_MISMATCH: &str = "GUARD_IMPL_MISMATCH";

sol_storage! {
    #[entrypoint]
    pub struct Guard {
        /// The Canonical Registry (contracts/core/registry) — record source.
        address registry;
        /// The project MM key: fills by granting the guard a standing
        /// tokenOut allowance, receives the escrowed tokenIn on settlement.
        address counterparty;
        /// buyer → their single active order (key = buyer; execute() is
        /// no-arg and runs for msg.sender).
        mapping(address => Order) orders;
    }

    /// One escrowed swap intent.
    pub struct Order {
        address token_in;
        address token_out;
        uint256 amount_in;
        uint256 min_out;
        bool active;
    }
}

/// Static-call context: forward all gas, no ETH — the standard probe posture
/// (spike/probe's AllGas).
struct Probe;
impl CallContext for Probe {
    fn gas(&self) -> u64 {
        u64::MAX
    }
}
impl StaticCallContext for Probe {}

/// Mutating-call context: forward all gas, move no ETH (token calls only).
struct TokenCall;
impl CallContext for TokenCall {
    fn gas(&self) -> u64 {
        u64::MAX
    }
}
unsafe impl MutatingCallContext for TokenCall {
    fn value(&self) -> U256 {
        U256::ZERO
    }
}

/// The genuine forwarder's implementation()-selector bytes — the calibrated
/// shape anchor after the embedded PUSH32 beacon word (spike/evidence).
const SEL_IMPLEMENTATION_BYTES: [u8; 4] = [0x5c, 0x60, 0xda, 0x1b];

#[public]
impl Guard {
    /// Deploys the guard bound to one Canonical Registry and one MM-key
    /// counterparty. Both must be non-zero: a zero registry would fail
    /// closed on every swap, and a zero counterparty could neither fill nor
    /// receive the escrow.
    #[constructor]
    pub fn constructor(
        &mut self,
        registry: Address,
        counterparty: Address,
    ) -> Result<(), GuardError> {
        if registry.is_zero() || counterparty.is_zero() {
            return Err(Revert {
                reason: "GUARD_ZERO_CONFIG".to_string(),
            }
            .into());
        }
        self.registry.set(registry);
        self.counterparty.set(counterparty);
        Ok(())
    }

    /// The bound Canonical Registry address.
    pub fn registry(&self) -> Address {
        self.registry.get()
    }

    /// The bound counterparty (project MM key) address.
    pub fn counterparty(&self) -> Address {
        self.counterparty.get()
    }

    /// A buyer's order — `(tokenIn, tokenOut, amountIn, minOut, active)`.
    /// All-zero tuple with `active == false` when no order exists.
    pub fn get_order(&self, buyer: Address) -> (Address, Address, U256, U256, bool) {
        let o = self.orders.get(buyer);
        (
            o.token_in.get(),
            o.token_out.get(),
            o.amount_in.get(),
            o.min_out.get(),
            o.active.get(),
        )
    }

    /// Escrow the buyer's `amountIn` of tokenIn for a swap into `tokenOut`
    /// wanting at least `minOut`. No risk checks here — `execute()` owns
    /// those; commit only takes custody (transferFrom buyer → guard).
    pub fn commit(
        &mut self,
        token_in: Address,
        token_out: Address,
        amount_in: U256,
        min_out: U256,
    ) -> Result<(), GuardError> {
        let buyer = self.vm().msg_sender();
        if amount_in.is_zero() {
            return Err(EscrowAmountZero {}.into());
        }
        {
            let o = self.orders.get(buyer);
            if o.active.get() {
                return Err(EscrowOrderActive { buyer }.into());
            }
        }

        // Checks-effects-interactions: the order is stored before the token
        // call, so a reentrant token finds an active escrow (and any revert
        // rolls the whole tx back — nothing persists).
        let mut o = self.orders.setter(buyer);
        o.token_in.set(token_in);
        o.token_out.set(token_out);
        o.amount_in.set(amount_in);
        o.min_out.set(min_out);
        o.active.set(true);

        // Take custody. A failing transferFrom reverts the tx untouched.
        let guard = self.vm().contract_address();
        self.erc20(
            token_in,
            SEL_TRANSFER_FROM,
            words([buyer.into(), guard.into(), amount_in.into()]),
        )?;
        Ok(())
    }

    /// Run every check on BOTH tokens, then settle:
    /// pull the counterparty's fill (tokenOut → buyer, exactly `minOut`) and
    /// release the escrow (tokenIn → counterparty, exactly `amountIn`).
    /// No-arg: operates on the caller's (the buyer's) active order.
    ///
    /// All five red flags revert strictly before any movement; no error is
    /// ever swallowed, so any later failure rolls the whole tx back — funds
    /// move only on success (journeys.md:32, plan task 3).
    pub fn execute(&mut self) -> Result<(), GuardError> {
        let buyer = self.vm().msg_sender();

        {
            let o = self.orders.get(buyer);
            if !o.active.get() {
                return Err(EscrowNoOrder { buyer }.into());
            }
        }

        // Effects before interactions: deactivate first so reentrant tokens
        // (transfer hooks) find no active escrow. Reads are snapshotted
        // before the write.
        let (token_in, token_out, amount_in, min_out) = {
            let o = self.orders.get(buyer);
            (
                o.token_in.get(),
                o.token_out.get(),
                o.amount_in.get(),
                o.min_out.get(),
            )
        };
        self.orders.setter(buyer).active.set(false);

        // Checks — fixed order, tokenIn then tokenOut (deterministic which
        // flag fires when several are true).
        self.check_token(token_in, buyer)?;
        self.check_token(token_out, buyer)?;

        // Settlement — buyer first, then the escrow release. Both calls are
        // strict: revert data passes through, an explicit `false` reverts.
        self.counterparty_fill(token_out, buyer, min_out)?;
        self.release_escrow(token_in, amount_in)?;
        Ok(())
    }

}

// ---- internal helpers (NOT part of the ABI — kept out of `#[public]`,
// which ABI-wraps every fn in its impl block) ----

impl Guard {
    /// All deterministic red-flag checks for ONE token, in guard order:
    /// record → revoked → paused → blocklisted → impl-mismatch. The probe
    /// readings feed the pure `red_flag` decision table; probes that cannot
    /// produce an answer degrade to "no flag" — they never revert
    /// (spec.md:28). The record staticcall is the exception because a
    /// registry that will not answer must fail CLOSED (no record).
    fn check_token(&self, token: Address, buyer: Address) -> Result<(), GuardError> {
        // 1+2: Canonical Registry record — fail closed.
        let (status, _, _, record_impl, registrar, _, _) = self.read_record(token)?;

        // 3: global pause — on the token PROXY (calibrated target).
        let (paused_ok, paused) = self.static_bool(token, SEL_PAUSED, &[]);

        // 4+5: beacon-shaped token → blocklist + implementation probes.
        // Unresolvable shape degrades silently (never reverts).
        let (blocked_ok, blocked, resolved_impl) = match self.resolve_beacon(token) {
            Some(beacon) => {
                let (ok, blocked) = self.static_bool(beacon, SEL_IS_BLOCKED, &enc_word(buyer));
                let impl_ = self.static_addr(beacon, SEL_IMPLEMENTATION, &[]);
                (ok, blocked, impl_)
            }
            None => (false, false, None),
        };

        match red_flag(
            status,
            registrar,
            record_impl,
            (paused_ok, paused),
            (blocked_ok, blocked),
            resolved_impl,
        ) {
            Some(flag) => Err(red(flag)),
            None => Ok(()),
        }
    }

    /// Pull the counterparty's fill: tokenOut `transferFrom(counterparty →
    /// buyer, minOut)` against the counterparty's standing allowance.
    fn counterparty_fill(
        &self,
        token_out: Address,
        buyer: Address,
        min_out: U256,
    ) -> Result<(), GuardError> {
        let counterparty = self.counterparty.get();
        self.erc20(
            token_out,
            SEL_TRANSFER_FROM,
            words([counterparty.into(), buyer.into(), min_out.into()]),
        )
    }

    /// Release the escrowed tokenIn to the counterparty. The guard holds
    /// exactly `amountIn` (custody taken at commit; paused/blocklist checks
    /// just passed), so this transfer settles the swap.
    fn release_escrow(&self, token_in: Address, amount_in: U256) -> Result<(), GuardError> {
        let counterparty = self.counterparty.get();
        self.erc20(
            token_in,
            SEL_TRANSFER,
            words([counterparty.into(), amount_in.into()]),
        )
    }

    /// Read a token's verification record from the registry.
    /// Decode failure or call failure = fail closed: GUARD_NO_RECORD.
    fn read_record(&self, token: Address) -> Result<(u8, U256, u64, Address, Address, u64, B256), GuardError> {
        let registry = self.registry.get();
        let ret = match call::static_call(self.vm(), Probe, registry, &record_calldata(token)) {
            Ok(ret) => ret,
            // A registry that will not answer stops the swap — fail closed.
            Err(_) => return Err(red(GUARD_NO_RECORD)),
        };
        if ret.len() != 224 {
            // A record always encodes as exactly 7 static words.
            return Err(red(GUARD_NO_RECORD));
        }
        let w = |i: usize| {
            let mut word = [0u8; 32];
            word.copy_from_slice(&ret[i * 32..(i + 1) * 32]);
            word
        };
        let addr = |i: usize| {
            let mut a = [0u8; 20];
            a.copy_from_slice(&w(i)[12..]);
            Address::new(a)
        };
        let status = w(0)[31];
        let risk_flags = U256::from_be_bytes(w(1));
        let verified_at = u64::from_be_bytes(w(2)[24..].try_into().unwrap());
        let impl_ = addr(3);
        let registrar = addr(4);
        let revoked_at = u64::from_be_bytes(w(5)[24..].try_into().unwrap());
        let reason = B256::from(w(6));
        Ok((status, risk_flags, verified_at, impl_, registrar, revoked_at, reason))
    }

    /// STATICCALL a token/beacon and read a bool — infallible probe posture
    /// (spike/probe): `(ok, value)`; empty returndata reads (true, false).
    fn static_bool(&self, to: Address, sel: u32, args: &[u8]) -> (bool, bool) {
        match self.static_word(to, sel, args) {
            (true, Some(word)) => (true, word[31] != 0),
            (true, None) => (true, false),
            (false, _) => (false, false),
        }
    }

    /// STATICCALL and decode a returned address — None on revert/odd shape.
    fn static_addr(&self, to: Address, sel: u32, args: &[u8]) -> Option<Address> {
        self.static_word(to, sel, args)
            .1
            .map(|word| {
                let mut a = [0u8; 20];
                a.copy_from_slice(&word[12..]);
                Address::new(a)
            })
    }

    /// STATICCALL `sel ++ args`, decode one 32-byte word if present.
    fn static_word(&self, to: Address, sel: u32, args: &[u8]) -> (bool, Option<[u8; 32]>) {
        match call::static_call(self.vm(), Probe, to, &calldata(sel, args)) {
            Ok(ret) if ret.len() == 32 => {
                let mut word = [0u8; 32];
                word.copy_from_slice(&ret);
                (true, Some(word))
            }
            Ok(_) => (true, None),
            Err(_) => (false, None),
        }
    }

    /// Resolve a token's embedded beacon by calibrated SHAPE (plan Revised
    /// note 2a): read the token's code, find the PUSH32 zero-padded beacon
    /// word anchored by the implementation() selector that follows it.
    /// None when the shape is absent — the guard then skips the beacon-bound
    /// checks instead of guessing.
    fn resolve_beacon(&self, token: Address) -> Option<Address> {
        extract_beacon_from_code(&self.vm().code(token))
    }

    /// Strict ERC-20 movement: revert data passes through untouched; an
    /// explicit `false` return reverts with TokenTransferFailed; empty
    /// returndata is accepted (non-standard but non-refusing tokens).
    fn erc20(&self, token: Address, sel: u32, args: Vec<u8>) -> Result<(), GuardError> {
        let ret = match call::call(self.vm(), TokenCall, token, &calldata(sel, &args)) {
            Ok(ret) => ret,
            Err(e) => {
                // Never swallowed: the token's raw revert data is carried
                // losslessly (stylus error enums require `SolError`
                // variants, so raw bubbling is impossible).
                let revert_data = match e {
                    CallError::Revert(data) => data,
                    CallError::AbiDecodingFailed(_) => alloc::vec![],
                };
                return Err(TokenCallFailed { token, revertData: revert_data.into() }.into());
            }
        };
        if ret.is_empty() || ret.get(31) == Some(&1) {
            Ok(())
        } else {
            Err(TokenTransferFailed { token }.into())
        }
    }
}

/// The deterministic red-flag decision for ONE token, in guard order
/// (journeys.md:32): record → revoked → paused → blocklisted → impl-mismatch.
/// Pure and host-independent, so the full truth table is unit-testable
/// natively (stylus-test 0.10.9 cannot replay multi-staticcall flows with
/// per-call bytes — verify.md loose end 5; the live wiring of these readings
/// is asserted on-chain by the task-5 receipts).
///
/// `paused`/`blocked` arrive as `(call_ok, value)` pairs — a probe that
/// reverts reads `(false, _)` and degrades to "no flag" (heuristics never
/// revert, spec.md:28). `resolved_impl` is None when the beacon answered
/// with nothing decodable or was unreachable: the impl check is skipped, it
/// never guesses. Returns the byte-exact revert string to raise, or None.
fn red_flag(
    status: u8,
    registrar: Address,
    record_impl: Address,
    (paused_ok, paused): (bool, bool),
    (blocked_ok, blocked): (bool, bool),
    resolved_impl: Option<Address>,
) -> Option<&'static str> {
    if registrar.is_zero() || status > STATUS_REVOKED {
        // Unknown status values are no-record, never guessed (wire.md).
        return Some(GUARD_NO_RECORD);
    }
    if status == STATUS_REVOKED {
        return Some(GUARD_RECORD_REVOKED);
    }
    if paused_ok && paused {
        return Some(GUARD_PAUSED);
    }
    if blocked_ok && blocked {
        return Some(GUARD_BLOCKLISTED);
    }
    if let Some(impl_) = resolved_impl {
        if impl_ != record_impl {
            return Some(GUARD_IMPL_MISMATCH);
        }
    }
    None
}

/// A deterministic red-flag revert with its exact pinned string
/// (classic `Error(string)` ABI data — what viem surfaces verbatim).
fn red(reason: &str) -> GuardError {
    Revert {
        reason: reason.to_string(),
    }
    .into()
}

/// `sel ++ args` calldata.
fn calldata(sel: u32, args: &[u8]) -> Vec<u8> {
    let mut data = Vec::with_capacity(4 + args.len());
    data.extend_from_slice(&sel.to_be_bytes());
    data.extend_from_slice(args);
    data
}

/// `getRecord(token)` calldata.
fn record_calldata(token: Address) -> Vec<u8> {
    calldata(SEL_GET_RECORD, &enc_word(token))
}

/// Left-pad an address / uint into one ABI word.
fn enc_word(value: impl Into<AbiWord>) -> [u8; 32] {
    value.into().0
}

/// Concatenate ABI words into calldata args.
fn words(values: impl IntoIterator<Item = AbiWord>) -> Vec<u8> {
    values
        .into_iter()
        .fold(Vec::with_capacity(32), |mut acc, w| {
            acc.extend_from_slice(&w.0);
            acc
        })
}

/// Private wrapper so `enc_word` accepts addresses and U256 uniformly.
struct AbiWord([u8; 32]);
impl From<Address> for AbiWord {
    fn from(a: Address) -> Self {
        let mut w = [0u8; 32];
        w[12..].copy_from_slice(a.as_slice());
        AbiWord(w)
    }
}
impl From<U256> for AbiWord {
    fn from(v: U256) -> Self {
        AbiWord(v.to_be_bytes())
    }
}

/// Extract the embedded beacon from a token's runtime code by the genuine
/// forwarder's calibrated shape: a PUSH32 (`0x7f`) whose word is a
/// zero-padded 20-byte address, anchored by the `implementation()` selector
/// bytes appearing within the following 16 bytes. Pure and host-independent;
/// unit-tested against the genuine bytecode captured in spike/evidence.
///
/// Shape-based by design — never codehash equality: the step-6 replicas
/// embed a different beacon at the same shape, and the joint fingerprint
/// with step 4 stays shape-based (plan Revised note 2a).
///
/// Cost: one linear scan over the token's own code. Genuine forwarders are
/// 283 bytes, so the scan is unmeasurable in practice; a pathological
/// large-code token only costs ink proportional to its own code size.
fn extract_beacon_from_code(code: &[u8]) -> Option<Address> {
    let mut i = 0usize;
    while i + 33 <= code.len() {
        if code[i] == 0x7f && code[i + 1..i + 13].iter().all(|&b| b == 0) {
            let word_end = i + 33;
            let window_end = (word_end + 16).min(code.len());
            if code[word_end..window_end]
                .windows(SEL_IMPLEMENTATION_BYTES.len())
                .any(|w| w == SEL_IMPLEMENTATION_BYTES)
            {
                let mut address = [0u8; 20];
                address.copy_from_slice(&code[i + 13..i + 33]);
                return Some(Address::new(address));
            }
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod test {
    use super::*;
    use stylus_sdk::testing::TestVM;

    const REGISTRY: Address = const { Address::new([0x11; 20]) };
    const MM: Address = const { Address::new([0x22; 20]) };
    const BUYER: Address = const { Address::new([0x33; 20]) };
    const TOKEN_IN: Address = const { Address::new([0x44; 20]) };
    const TOKEN_OUT: Address = const { Address::new([0x55; 20]) };
    const BEACON: Address = const { Address::new([0x66; 20]) };
    const IMPL: Address = const { Address::new([0x77; 20]) };
    const OTHER_IMPL: Address = const { Address::new([0x88; 20]) };
    const GUARD_ADDR: Address = const { Address::new([0x99; 20]) };

    /// The genuine 283-byte forwarder bytecode (spike/evidence/p_proxy.hex,
    /// live Robinhood Chain 4663) — the extraction test fixture.
    const GENUINE_FORWARDER_HEX: &str =
        "6080604052600a600c565b005b60186014601a565b609d565b565b5f7f000000000000000000000000e10b6f6b275de231345c20d14ab812db62151b006001600160a01b0316635c60da1b6040518163ffffffff1660e01b8152600401602060405180830381865afa1580156076573d5f5f3e3d5ffd5b505050506040513d601f19601f820116820180604052508101906098919060ba565b905090565b365f5f375f5f365f845af43d5f5f3e80801560b6573d5ff35b3d5ffd5b5f6020828403121560c9575f5ffd5b81516001600160a01b038116811460de575f5ffd5b939250505056fea2646970667358221220e296a217a10765339b402e91ead95bcbcf74e678d7c756705f097d88cc2554b064736f6c63430008210033";

    fn hex_bytes(hex: &str) -> Vec<u8> {
        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect()
    }

    fn word_addr(a: Address) -> Vec<u8> {
        enc_word(a).to_vec()
    }

    fn word_u256(v: U256) -> Vec<u8> {
        enc_word(v).to_vec()
    }

    /// Encode a getRecord returndata tuple: 7 static words.
    fn record_return(status: u8, impl_: Address, registrar: Address) -> Vec<u8> {
        let mut ret = alloc::vec![0u8; 224];
        ret[31] = status;
        ret[32..64].copy_from_slice(&[0u8; 32]); // riskFlags = 0
        ret[64 + 31] = 0; // verifiedAt = 0
        ret[3 * 32 + 12..4 * 32].copy_from_slice(impl_.as_slice());
        ret[4 * 32 + 12..5 * 32].copy_from_slice(registrar.as_slice());
        ret
    }

    fn get_record_calldata(token: Address) -> Vec<u8> {
        calldata(SEL_GET_RECORD, &word_addr(token))
    }

    /// Guard with constructor run; caller = BUYER; guard's own address set.
    fn vm_with_guard() -> (TestVM, Guard) {
        let vm = TestVM::default();
        vm.set_sender(BUYER);
        vm.set_contract_address(GUARD_ADDR);
        let mut guard = Guard::from(&vm);
        guard.constructor(REGISTRY, MM).expect("constructor");
        (vm, guard)
    }

    /// Mock a VERIFIED record for `token` (impl = IMPL) on the registry.
    fn mock_record(vm: &TestVM, token: Address) {
        vm.mock_static_call(
            REGISTRY,
            get_record_calldata(token),
            Ok(record_return(STATUS_VERIFIED, IMPL, MM)),
        );
    }

    /// Commit TOKEN_IN → TOKEN_OUT for the buyer, accepting the escrow
    /// transferFrom (exact calldata = buyer → guard, amount).
    fn commit_order(vm: &TestVM, guard: &mut Guard, amount: U256, min_out: U256) {
        vm.mock_call(
            TOKEN_IN,
            calldata(
                SEL_TRANSFER_FROM,
                &words([BUYER.into(), GUARD_ADDR.into(), amount.into()]),
            ),
            U256::ZERO,
            Ok(word_u256(U256::from(1u8))),
        );
        guard
            .commit(TOKEN_IN, TOKEN_OUT, amount, min_out)
            .expect("commit");
    }

    /// Install the genuine-shape forwarder code on a token, with its
    /// embedded beacon pointing at BEACON.
    fn install_beacon_token(vm: &TestVM, token: Address) {
        let hex_addr: alloc::string::String = BEACON
            .as_slice()
            .iter()
            .map(|b| alloc::format!("{b:02x}"))
            .collect();
        let patched = GENUINE_FORWARDER_HEX.replace(
            "e10b6f6b275de231345c20d14ab812db62151b00",
            &hex_addr,
        );
        vm.set_code(token, hex_bytes(&patched));
    }

    /// Assert the error is a red flag with its byte-exact pinned string.
    fn assert_red(result: Result<(), GuardError>, expected: &str) {
        match result {
            Err(GuardError::Revert(Revert { reason })) => {
                assert_eq!(reason, expected, "red-flag string must be byte-exact");
            }
            other => panic!("expected red flag {expected}, got {other:?}"),
        }
    }

    // ---- constructor / views ----

    #[test]
    fn constructor_rejects_zero_config() {
        let vm = TestVM::default();
        vm.set_sender(BUYER);
        let mut guard = Guard::from(&vm);
        assert!(matches!(
            guard.constructor(Address::ZERO, MM),
            Err(GuardError::Revert(_))
        ));
        assert!(matches!(
            guard.constructor(REGISTRY, Address::ZERO),
            Err(GuardError::Revert(_))
        ));
    }

    #[test]
    fn constructor_binds_registry_and_counterparty() {
        let (_, guard) = vm_with_guard();
        assert_eq!(guard.registry(), REGISTRY);
        assert_eq!(guard.counterparty(), MM);
    }

    // ---- commit / escrow ----

    #[test]
    fn commit_escrows_via_transfer_from_and_stores_order() {
        let (vm, mut guard) = vm_with_guard();
        commit_order(&vm, &mut guard, U256::from(100u8), U256::from(90u8));
        let (token_in, token_out, amount_in, min_out, active) = guard.get_order(BUYER);
        assert!(active);
        assert_eq!((token_in, token_out), (TOKEN_IN, TOKEN_OUT));
        assert_eq!((amount_in, min_out), (U256::from(100u8), U256::from(90u8)));
    }

    #[test]
    fn commit_rejects_zero_amount() {
        let (_, mut guard) = vm_with_guard();
        assert!(matches!(
            guard.commit(TOKEN_IN, TOKEN_OUT, U256::ZERO, U256::ZERO),
            Err(GuardError::EscrowAmountZero(_))
        ));
        assert!(!guard.get_order(BUYER).4);
    }

    #[test]
    fn commit_rejects_second_active_order() {
        let (vm, mut guard) = vm_with_guard();
        commit_order(&vm, &mut guard, U256::from(100u8), U256::ZERO);
        assert!(matches!(
            guard.commit(TOKEN_IN, TOKEN_OUT, U256::from(1u8), U256::ZERO),
            Err(GuardError::EscrowOrderActive(_))
        ));
    }

    #[test]
    fn commit_propagates_failing_transfer_from() {
        // A registered Err mock simulates the reverting token (a mock MISS
        // reads as success-with-empty-returndata in stylus-test). The raw
        // revert data passes through (Call variant); on-chain, atomicity
        // then rolls the stored order back.
        let (vm, mut guard) = vm_with_guard();
        vm.mock_call(
            TOKEN_IN,
            calldata(
                SEL_TRANSFER_FROM,
                &words([BUYER.into(), GUARD_ADDR.into(), U256::from(1u8).into()]),
            ),
            U256::ZERO,
            Err(alloc::vec![]),
        );
        assert!(matches!(
            guard.commit(TOKEN_IN, TOKEN_OUT, U256::from(1u8), U256::ZERO),
            Err(GuardError::TokenCallFailed(_))
        ));
    }

    // ---- execute: red flags, byte-exact ----
    //
    // stylus-test 0.10.9 quirk (spike findings): the BYTES every staticcall
    // returns come from the LAST-registered mock — only the returndata
    // LENGTH is per-call. Multi-staticcall flows therefore cannot see
    // per-call values natively, so (verify.md loose end 5) the flags that
    // depend on probe answers are asserted through the pure `red_flag`
    // decision table below, and the LIVE WIRING (right probe to right
    // target, settle accounting) is asserted on-chain by the task-5
    // receipts. The flags that fire before any probe (NO_RECORD family,
    // RECORD_REVOKED) run through the real execute() flow here.

    #[test]
    fn execute_without_order_reverts() {
        let (_, mut guard) = vm_with_guard();
        assert!(matches!(
            guard.execute(),
            Err(GuardError::EscrowNoOrder(_))
        ));
    }

    #[test]
    fn no_record_fails_closed_on_registry_failure() {
        // Registry staticcall misses (no mock) → GUARD_NO_RECORD, not a
        // pass-through: a registry that will not answer must stop the swap.
        let (vm, mut guard) = vm_with_guard();
        commit_order(&vm, &mut guard, U256::from(100u8), U256::ZERO);
        assert_red(guard.execute(), GUARD_NO_RECORD);
    }

    #[test]
    fn no_record_on_zero_registrar_tuple() {
        let (vm, mut guard) = vm_with_guard();
        commit_order(&vm, &mut guard, U256::from(100u8), U256::ZERO);
        vm.mock_static_call(
            REGISTRY,
            get_record_calldata(TOKEN_IN),
            Ok(record_return(STATUS_VERIFIED, IMPL, Address::ZERO)),
        );
        assert_red(guard.execute(), GUARD_NO_RECORD);
    }

    #[test]
    fn unknown_status_is_no_record_never_guessed() {
        let (vm, mut guard) = vm_with_guard();
        commit_order(&vm, &mut guard, U256::from(100u8), U256::ZERO);
        vm.mock_static_call(
            REGISTRY,
            get_record_calldata(TOKEN_IN),
            Ok(record_return(7, IMPL, MM)),
        );
        assert_red(guard.execute(), GUARD_NO_RECORD);
    }

    #[test]
    fn record_revoked_reverts_byte_exact() {
        let (vm, mut guard) = vm_with_guard();
        commit_order(&vm, &mut guard, U256::from(100u8), U256::ZERO);
        vm.mock_static_call(
            REGISTRY,
            get_record_calldata(TOKEN_IN),
            Ok(record_return(STATUS_REVOKED, IMPL, MM)),
        );
        assert_red(guard.execute(), GUARD_RECORD_REVOKED);
    }

    #[test]
    fn read_record_decodes_the_seven_fields() {
        // Byte-faithful decode (single staticcall — the quirk-safe split).
        let (vm, guard) = vm_with_guard();
        let mut ret = alloc::vec![0u8; 224];
        ret[31] = STATUS_VERIFIED;
        ret[32..64].copy_from_slice(&U256::from(0xdead_u64).to_be_bytes::<32>());
        ret[64..96].copy_from_slice(&U256::from(1_700_000_000u64).to_be_bytes::<32>());
        ret[3 * 32 + 12..4 * 32].copy_from_slice(IMPL.as_slice());
        ret[4 * 32 + 12..5 * 32].copy_from_slice(MM.as_slice());
        ret[5 * 32 + 24..6 * 32].copy_from_slice(&U256::from(123u64).to_be_bytes::<32>()[24..32]);
        ret[6 * 32..].copy_from_slice(&B256::repeat_byte(0xab).0);
        vm.mock_static_call(REGISTRY, get_record_calldata(TOKEN_IN), Ok(ret));
        let (status, risk_flags, verified_at, impl_, registrar, revoked_at, reason) =
            guard.read_record(TOKEN_IN).expect("decode");
        assert_eq!(status, STATUS_VERIFIED);
        assert_eq!(risk_flags, U256::from(0xdead_u64));
        assert_eq!(verified_at, 1_700_000_000);
        assert_eq!(impl_, IMPL);
        assert_eq!(registrar, MM);
        assert_eq!(revoked_at, 123);
        assert_eq!(reason, B256::repeat_byte(0xab));
    }

    #[test]
    fn read_record_short_returndata_is_no_record() {
        let (vm, guard) = vm_with_guard();
        vm.mock_static_call(
            REGISTRY,
            get_record_calldata(TOKEN_IN),
            Ok(alloc::vec![0u8; 32]),
        );
        assert_red(guard.read_record(TOKEN_IN).map(|_| ()), GUARD_NO_RECORD);
    }

    /// The decision table for the two probe-independent record flags plus
    /// ordering: a revoked record must win over any probe answer.
    #[test]
    fn red_flag_record_checks_precede_probes() {
        assert_eq!(
            red_flag(
                STATUS_REVOKED,
                MM,
                IMPL,
                (true, true),
                (true, true),
                Some(OTHER_IMPL)
            ),
            Some(GUARD_RECORD_REVOKED)
        );
        assert_eq!(
            red_flag(7, MM, IMPL, (true, true), (true, true), Some(OTHER_IMPL)),
            Some(GUARD_NO_RECORD)
        );
        assert_eq!(
            red_flag(STATUS_VERIFIED, Address::ZERO, IMPL, (true, true), (true, true), None),
            Some(GUARD_NO_RECORD)
        );
    }

    /// Byte-exact decision for every probe-dependent flag (journeys.md:32).
    #[test]
    fn red_flag_probe_flags_are_byte_exact() {
        // paused on the proxy
        assert_eq!(
            red_flag(STATUS_VERIFIED, MM, IMPL, (true, true), (true, false), None),
            Some(GUARD_PAUSED)
        );
        // blocklisted on the resolved beacon
        assert_eq!(
            red_flag(STATUS_VERIFIED, MM, IMPL, (true, false), (true, true), None),
            Some(GUARD_BLOCKLISTED)
        );
        // implementation ≠ record, on the resolved beacon
        assert_eq!(
            red_flag(
                STATUS_VERIFIED,
                MM,
                IMPL,
                (true, false),
                (true, false),
                Some(OTHER_IMPL)
            ),
            Some(GUARD_IMPL_MISMATCH)
        );
        // resolution succeeded AND matches → no flag
        assert_eq!(
            red_flag(
                STATUS_VERIFIED,
                MM,
                IMPL,
                (true, false),
                (true, false),
                Some(IMPL)
            ),
            None
        );
    }

    /// Heuristics never revert (spec.md:28): probes that cannot answer —
    /// reverted call, empty returndata, unreachable beacon — degrade to
    /// "no flag"; the impl check is skipped when the beacon does not answer.
    #[test]
    fn red_flag_degrades_probes_that_cannot_answer() {
        // every probe reverts or is unreachable
        assert_eq!(
            red_flag(STATUS_VERIFIED, MM, IMPL, (false, false), (false, false), None),
            None
        );
        // probes answer but impl resolution fails → impl check skipped
        assert_eq!(
            red_flag(STATUS_VERIFIED, MM, IMPL, (true, false), (true, false), None),
            None
        );
        // paused unreachable while blocklist answers clean
        assert_eq!(
            red_flag(STATUS_VERIFIED, MM, IMPL, (false, false), (true, false), Some(IMPL)),
            None
        );
    }

    // ---- beacon extraction ----

    #[test]
    fn extraction_finds_the_genuine_embedded_beacon() {
        let beacon = extract_beacon_from_code(&hex_bytes(GENUINE_FORWARDER_HEX));
        assert_eq!(
            beacon,
            Some(Address::new(
                hex_bytes("e10b6f6b275de231345c20d14ab812db62151b00")
                    .try_into()
                    .unwrap()
            ))
        );
    }

    #[test]
    fn extraction_is_shape_based_not_codehash_based() {
        // Same shape, different embedded beacon — the step-6 replica case.
        install_beacon_helper_assert();
    }

    fn install_beacon_helper_assert() {
        let patched = GENUINE_FORWARDER_HEX.replace(
            "e10b6f6b275de231345c20d14ab812db62151b00",
            "1111111111111111111111111111111111111111",
        );
        assert_eq!(
            extract_beacon_from_code(&hex_bytes(&patched)),
            Some(Address::new([0x11; 20]))
        );
    }

    #[test]
    fn extraction_fails_closed_without_the_shape() {
        // No PUSH32-with-beacon + selector anchor → None → checks skip.
        assert_eq!(extract_beacon_from_code(&[]), None);
        assert_eq!(extract_beacon_from_code(&alloc::vec![0x60, 0x00, 0xf3]), None);
        // PUSH32 zero-padded word but no implementation() anchor nearby.
        let mut decoy = alloc::vec![0x7f];
        decoy.extend_from_slice(&[0u8; 12]);
        decoy.extend_from_slice(&[0xabu8; 20]);
        decoy.extend_from_slice(&alloc::vec![0x63, 0xde, 0xad, 0xbe, 0xef]);
        assert_eq!(extract_beacon_from_code(&decoy), None);
    }

    // ---- pinned surface ----

    #[test]
    fn pinned_selectors_match_abi_ts() {
        // Byte-pin against packages/shared/abi.ts (SELECTORS + PROBE_SELECTORS).
        let cases: [(&str, u32); 8] = [
            ("commit(address,address,uint256,uint256)", 0x498ab631),
            ("execute()", 0x61461954),
            ("getRecord(address)", 0x617fba04),
            ("paused()", 0x5c975abb),
            ("isBlocked(address)", 0xfbac3951),
            ("implementation()", 0x5c60da1b),
            ("transferFrom(address,address,uint256)", 0x23b872dd),
            ("transfer(address,uint256)", 0xa9059cbb),
        ];
        for (sig, pinned) in cases {
            let word = stylus_sdk::crypto::keccak(sig.as_bytes());
            assert_eq!(
                u32::from_be_bytes([word[0], word[1], word[2], word[3]]),
                pinned,
                "selector drift on {sig}"
            );
        }
    }

    #[test]
    fn red_flags_encode_as_error_string() {
        // The five flags must ride classic Error(string) data — viem's
        // ContractFunctionRevertedError surfaces the `reason` verbatim.
        let err = red(GUARD_PAUSED);
        match err {
            GuardError::Revert(Revert { reason }) => assert_eq!(reason, "GUARD_PAUSED"),
            other => panic!("wrong variant {other:?}"),
        }
    }
}
