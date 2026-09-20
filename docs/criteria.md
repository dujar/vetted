# Verification criteria — the Canonical Registry

This is the published criteria page the [Vetted registry UI](https://github.com/dujar/vetted)
links from the registry panel (the "who writes" card). It states, exactly, what it means for
a token to carry a **verification record** in the Vetted Canonical Registry, who can write
one, and what a record does and does not assert. The registry is the on-chain ground truth
for the [guarded swap](#how-the-guard-consumes-a-record); its rows are written after a check
against the issuer's own live list — the registry stores and exposes, it does not judge.

**Wording note:** this system is the *Canonical Registry*, its rows are *verification
records*, and the single writer is the *registrar*. It is deliberately **not** called an
"attestation" — that word means something else on-chain (EIP-712 trade quotes) and we will
not borrow its credibility.

---

## 1. What a verification record is

One record per token address, stored on-chain (Robinhood Chain 4663 production; Arbitrum
Sepolia 421614 mirror), with seven fields:

| field | type | meaning |
|---|---|---|
| `status` | `uint8` | `0` = VERIFIED, `1` = REVOKED. Any other value is treated as no record, never guessed. |
| `riskFlags` | `uint256` | Bitfield of the project's risk findings at verification time (e.g. upgradeable-beacon, transfer-hook, pause-capable — the powers the power report discloses). |
| `verifiedAt` | `uint64` | Unix timestamp of the most recent `verify`. |
| `impl` | `address` | The implementation pointer the registrar recorded — the guard compares this against the token's live implementation on every swap. |
| `registrar` | `address` | The key that wrote the record. **`registrar == address(0)` means no record exists** — that is how "unknown token" is encoded (a read never reverts). |
| `revokedAt` | `uint64` | When the record was revoked; `0` while VERIFIED. |
| `reason` | `bytes32` | `keccak256(utf8(reason))` of the revocation. The full human-readable text rides the `Revoke` event (J3 renders it); the hash is what fits a record slot. |

Read interface (the primitive other contracts consume):

```solidity
getRecord(address token)
  returns (uint8 status, uint256 riskFlags, uint64 verifiedAt,
           address impl, address registrar, uint64 revokedAt, bytes32 reason)
// selector 0x617fba04 — never reverts; registrar == 0 means "no record"
```

Wallets, aggregators and frontends can gate on this record on-chain. Consumers MUST treat
`registrar == 0` or `status > 1` as no-record — the registry never guesses, and neither
should you.

## 2. Who writes, and against what ground truth

Every write path (`verify`, `revoke`, `transferRegistrar`) requires the caller to be the
**registrar** — a single project-held key whose address is readable on-chain
(`registrar()`). Ground truth is the **issuer's live canonical list**
(docs.robinhood.com/chain/contracts, fetched at scan/verify time, never baked in):

A token is written VERIFIED only when **all** of the following hold:

1. **Positive evidence, exact address.** The token's name and symbol appear on the issuer's
   live list *at that exact contract address*. Mimicry is not identity: a same-name/symbol
   token at a different address is an impostor by definition, and stays unverified.
2. **Implementation bound.** The token's current implementation (resolved through its
   beacon, for the genuine proxy pattern) is recorded in `impl`.
3. **Powers disclosed.** The `riskFlags` bitfield records the token's known powers —
   global pause, per-address blocklist, upgradeability — so verification never launders a
   capability the issuer holds over holders.

Revocation (`revoke`) is registrar-only, requires an existing record (revocation never
creates rows), and sets `status = REVOKED` with a human-readable reason. Re-verification
after a re-check reinstates a record.

## 3. Revocation policy — revocation is the feature

A verification record can go stale, and a stale one must not be load-bearing. The record is
revoked when:

- **Beacon upgrade / implementation drift.** The project backend polls every record's
  `impl` against the token's live implementation (Workers cron, every 15 minutes; a manual
  endpoint exists for the demo beat) and revokes on drift — automatically, registrar-signed.
- **Issuer-list change.** A token removed from, or re-addressed on, the issuer's live list.
- **Registrar error.** A wrong verification is one `revoke` away; the same pen that wrote
  it can withdraw it.

Between a drift event and its revocation there is a window; the guard's live
implementation-vs-record check closes it at swap time (see §5). Revocation is a safety
mechanism, not a failure state: every verify emits `Verify(...)`, every revoke emits
`Revoke(token, reason)` — the full audit trail is public on-chain.

## 4. What VERIFIED does **not** mean

- **Not an audit.** No one read the code line-by-line for correctness or solvency.
- **Not a liquidity or price guarantee.** The record says the token is the issuer's, not
  that it is worth anything.
- **Not issuer intent.** The issuer changing their own contract later invalidates the
  record (§3) rather than implicating the registry.
- **Not a deep scan of arbitrary tokens.** The depth boundary, stated in the UI: full
  verdicts require the known Robinhood stock-token pattern; every other contract gets
  UNVERIFIED plus generic structural heuristics, labeled advisory. Heuristic findings warn
  in a report; they never revert a swap.

## 5. How the guard consumes a record

The guarded swap escrow contract checks BOTH swap tokens, in a fixed order, before any
funds move. Five deterministic red flags revert with byte-exact strings:

| # | flag | trigger |
|---|---|---|
| 1 | `GUARD_NO_RECORD` | no record (or unreadable registry) for a swap token — **fails closed** |
| 2 | `GUARD_RECORD_REVOKED` | `status == REVOKED` |
| 3 | `GUARD_PAUSED` | live `paused()` probe true on the token proxy |
| 4 | `GUARD_BLOCKLISTED` | live `isBlocked(buyer)` probe true on the token's **resolved beacon** (where the genuine pattern keeps its blocklist) |
| 5 | `GUARD_IMPL_MISMATCH` | resolved live implementation ≠ the record's `impl` |

The full guard execution — both records, all probes, and settlement — is budgeted at
≤ 200,000 gas ([gas report](gas-report.md)). Probes that cannot produce an answer degrade to
"no flag" (heuristics never revert); only the registry read fails closed. The full trust
model is [threats.md](threats.md).

## 6. Evidence discipline

Every verdict line the product renders links its evidence on screen: the verification or
revocation transaction, the storage slots probed, the bytecode compared, or the probe
result. Verdicts are never asserted without a link; absent evidence renders as honest
UNVERIFIED, never as a quiet pass.
