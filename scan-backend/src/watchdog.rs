//! `/watchdog` — the censorship watchdog widget's numbers: cumulative
//! sequencer-filterer runs in ONE RPC read vs the published 6-week baseline
//! (two numbers, no stored history — spec.md:37).
//!
//! DEGRADED BY RECONCILIATION (2026-09-19, the plan's spike-grade check):
//! no L1 read reconciles with the published numbers. Measured live:
//!
//! - SequencerInbox 0xBd0D…ba96 `batchCount()` = 258,707 (one eth_call) vs
//!   the published 6,092 cumulative for 2026-08 — an order of magnitude apart;
//! - SequencerBatchDelivered-style events run ~2,630/day vs the published
//!   ~150/day (and the event signature is a fork-specific one, not the
//!   upstream nitro set — the observed pair includes
//!   InboxMessageDelivered(uint256,bytes) at 0xff64905f…).//! The published figures (xroot, third-party, 2026-08) evidently count
//! something else that is not on-chain enumerable. Per the plan's
//! pre-committed escape, the endpoint ships the degrade path — baseline +
//! provenance link — and flags the coordinator. The live read stays available
//! in tests/live_integration.rs for the day the mechanism is pinned.
//!
//! Wire semantics (wire.md "Watchdog API"): `runs: 0` with a `provenanceUrl`
//! means the live count is UNAVAILABLE — never a measured zero.

use vetted_shared::watchdog::WatchdogStats;

/// The published daily baseline (spec.md:37; ~150/day, measured 2026-08).
pub const BASELINE_PER_DAY: u64 = 150;

/// Where the published baseline comes from — the chain's own screening
/// statement ("any transaction associated with a sanctioned address will be
/// excluded from inclusion", docs.robinhood.com/chain/differences-from-ethereum).
pub const BASELINE_PROVENANCE: &str =
    "https://docs.robinhood.com/chain/differences-from-ethereum";

/// L1 SequencerInbox (knowledge robinhood-chain.md:42-43) — kept for the
/// deferred live mechanism, referenced by the degraded provenance trail.
pub const SEQUENCER_INBOX_L1: &str = "0xbd0d173eeb87d57a09521c24388a12789f33ba96";

/// The degrade shape every chain gets until a reconciled counter is pinned.
pub fn stats(chain_id: u64) -> WatchdogStats {
    WatchdogStats {
        chain_id,
        runs: 0, // 0 = count unavailable, NOT a measured zero (wire.md)
        baseline_per_day: BASELINE_PER_DAY,
        provenance_url: Some(BASELINE_PROVENANCE.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn degrade_shape_carries_baseline_and_provenance() {
        let s = stats(4663);
        assert_eq!(s.chain_id, 4663);
        assert_eq!(s.runs, 0);
        assert_eq!(s.baseline_per_day, 150);
        assert!(s.provenance_url.as_deref().unwrap().starts_with("https://docs.robinhood.com/"));
    }
}
