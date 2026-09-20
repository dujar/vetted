# Gas report — the guarded swap's ≤ 200,000-gas budget

**Status: PASS-PENDING-FUNDS** (step-2 precedent: the claim is made honestly, the receipt
that settles it is named, and the unblock is a single funded operator run — never a
fabricated number). Verified as of 2026-09-21: the operator key (0x151e…8E4C) reads zero
balance on 4663, 46630 and 421614, so **no funded on-chain run exists anywhere yet** — the
≤ 200,000 assertion (spec.md:51) is checked on-chain by the deploy script's gas gate the
moment the first funded run executes (runbook: `findings.md` in the step-7 workbench dir;
`spike/DEPLOY.md` §0 for funding). This file is the committed half: the budget definition,
the exact receipt methodology, everything measurable without funds, and the drift policy
the actuals land into.

## 1. The budget and what it covers

spec.md:51 fixes the spike gate at **≤ 200,000 gas per swap** for the full probe suite.
For the shipped product that number is applied to **one full `execute()` call** — both
tokens' checks and the settlement — composed of, per token:

| step | calls |
|---|---|
| record read (fail-closed) | 1 registry staticcall (`getRecord`, 224-byte return) |
| global pause | 1 staticcall to the token proxy (`paused()`) |
| beacon resolution | code read of the token (hostio, no call) + shape scan |
| blocklist | 1 staticcall to the **resolved beacon** (`isBlocked(buyer)`) — the genuine pattern keeps blocklist state on the beacon, so the probe targets it, not the token (`packages/shared/abi.ts` PROBE_SELECTORS, calibrated live) |
| implementation pointer | 1 staticcall to the beacon (`implementation()`) |
| settlement (once, not per token) | 2 ERC-20 calls: `transferFrom(MM → buyer, minOut)` + `transfer(MM, amountIn)`, exact amounts |

All probes forward all gas and move no ETH (read-only). Worst case measured path: both
tokens beacon-shaped with records present — 8 staticcalls + 2 token transfers + 2 code
reads, inside the 200k budget.

## 2. Receipt methodology (how the number is settled)

The assertion lives **on-chain, from broadcast receipts** — never from estimates, never
from the native test host (stylus-test does not meter EVM gas; a repo test claiming a gas
number would be fabrication):

1. The first funded run goes **end-to-end** through `scripts/deploy/core/deploy.sh`
   (stage D runs `RunReceipts.s.sol` on-chain: the five red-flag executes, the degraded
   skip, and the clean settle — 11 guard `execute()` transactions; the forged
   `expectRevert`s assert the byte-exact strings and the probe→target wiring on-chain,
   which the native tests cannot).
2. Stage E joins the broadcast `receipts` array to `transactions` **by hash**, converts
   hex `gasUsed` → decimal, and enforces **per-row**: expected status (rows 1/3/5/7/9
   reverted) AND `gasUsed ≤ 200,000`. Any failure aborts the deploy.
3. The per-row actuals are committed into `deployments/<chain>.json` under `receipts`
   (append-only), and the table in §4 below is filled from them — with the drift
   explanation per §5.

## 3. Measured now (no funds required — committed evidence)

WASM size and activation data fee are chain-independent dry-run measurements
(`cargo stylus check` exit 0 against the live RPC of each chain; checked on **both**
421614 and 4663 today):

| contract | wasm size (compressed) | ArbWasm data fee (quoted, 20% bump included) | vs 96 KB runtime cap | vs 24 KB fragmentation threshold |
|---|---|---|---|---|
| Canonical Registry | 11.2 KB (11,173 B) | 0.000084 ETH | 11.4% | well under — no Stylus contract fragmentation, `verify` unaffected by the pre-0.10.8 fragmented-contract bug |
| Guarded Swap | 11.9 KB (11,939 B) | 0.000090 ETH | 12.4% | same |
| (week-1 spike probe, `spike/evidence/stylus_check_4663_421614.log`) | 12.4 KB (12,429 B) | 0.000092 ETH | 12.9% | same |

Deployment cost model per chain, for funding sizing: two Stylus deploys (each = deploy +
activation; the data fees above + execution — call it ≈ 0.0005 ETH total) + the foundry
mock set + 13 receipt transactions. A few tenths of an ETH-equivalent covers the whole
session with margin; the drift-revoke registrar key needs only revoke gas afterwards.

**Why the actuals will land far under 200k:** the swap's hot path is 8 staticcalls and 2
storage-slot-scale writes; Stylus hostios (extcodecopy-equivalent code reads, staticcalls)
price near raw EVM cost, and the only state the guard mutates is one 5-slot order struct.
The 200k budget was fixed at spec time against the *whole probe suite in one tx*; the
product swap adds two token transfers on top of the probes. The receipts exist to prove
that, not to hope for it.

## 4. Actuals table (filled by the first funded run — PASS-PENDING-FUNDS)

| receipt (deploy order) | expected | gasUsed 421614 | gasUsed 4663 |
|---|---|---|---|
| `GUARD_NO_RECORD` | reverted, ≤ 200k | pending | pending |
| `DEGRADED_SETTLE` (extraction fails → skip, settle) | ok, ≤ 200k | pending | pending |
| `GUARD_RECORD_REVOKED` | reverted, ≤ 200k | pending | pending |
| `REVOKED_CLEANUP` | ok, ≤ 200k | pending | pending |
| `GUARD_PAUSED` | reverted, ≤ 200k | pending | pending |
| `PAUSED_CLEANUP` | ok, ≤ 200k | pending | pending |
| `GUARD_BLOCKLISTED` | reverted, ≤ 200k | pending | pending |
| `BLOCKLISTED_CLEANUP` | ok, ≤ 200k | pending | pending |
| `GUARD_IMPL_MISMATCH` | reverted, ≤ 200k | pending | pending |
| `MISMATCH_CLEANUP` | ok, ≤ 200k | pending | pending |
| `SETTLE` (clean two-party swap) | ok, ≤ 200k | pending | pending |

Receipt rows 1/3/5/7/9 also assert what the native suite cannot: the **zero movement on
every revert path** (buyer and MM token balances unchanged by the reverted executes —
EVM atomicity against the receipts) and the **probe→target wiring** (pause probed at the
proxy, blocklist at the resolved beacon). Cite them next to `contracts/core/guard`'s fuzz
suite when judging.

If funding has not landed at close, this file ships as-is (PASS-PENDING markers, this
section, the §3 evidence) and the deploy + handoff stay named blockers in the step-7
findings — per the standing fallback. The receipts are never invented.

## 5. Drift policy

When the funded run lands: fill §4 from `deployments/<chain>.json` `receipts`, then
explain any drift against §3's expectations in place. Expected drift axes, pre-registered:

- **Activation fee ≠ execution gas.** The §3 data fees are one-time storage-prepayment
  quoted by ArbWasm; the §4 rows are pure execution — do not sum them.
- **Estimate-vs-receipt**: `--estimate-gas` quotes (used for funding size) include
  safety margins; receipts are the truth.
- **Reverted ≠ free**: a red-flag revert refunds most but not all gas; reverted rows are
  expected to be the *cheapest* rows, not zero.
- **Chain deltas**: 4663 vs 421614 gasUsed should be near-identical (same ArbOS line);
  material divergence is itself a finding (different ArbOS/ArbWasm versions) and gets
  recorded, not smoothed over.

Related committed receipt sets: step-2's local replica-beat receipts (`spike/` broadcast,
chain 412346) and step-6's rehearsal broadcast receipts (`contracts/replicas/`) cover the
Solidity replica deploys, not the guard's budget; they are demo assets, not part of this
report's ledger.
