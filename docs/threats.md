# Threat model & trust assumptions

Scope: the Vetted on-chain surface — the **Canonical Registry** and the **Guarded Swap**
(`contracts/core/`, Rust/Stylus) — as deployed for v1 on Robinhood Chain 4663 (Arbitrum
Sepolia mirror). The scan backend and frontend are covered by their own reviews; where they
touch this model (canonical fetch, drift cron) the assumption they make on-chain is stated
here. The published consumer-facing half of this document is
[criteria.md](criteria.md); this file is the "what do you trust and what happens when it
breaks" half.

## 1. Assets at risk

1. **Buyer funds in an escrowed swap** — the only user money the contracts ever custody.
2. **Verdict integrity** — the registry's records are the primitive third parties are
   invited to integrate; a false record is a false fact other systems may rely on.

Everything else (scan reports, watchdog counts, UI copy) is advisory and explicitly not
load-bearing for funds.

## 2. Trust roots

| root | what it can break | why it is accepted |
|---|---|---|
| The **registrar key** (project-held, single writer) | verdict integrity (§3) | v1 trust root, bounded below; roadmap: multisig + timelock |
| The **issuer's live canonical list** (docs.robinhood.com/chain/contracts) | which tokens get VERIFIED | it is the issuer's own claim about its own tokens — the only sensible ground truth for "is this the real one" |
| The **Robinhood Chain sequencer** | ordering/inclusion of every tx | the chain itself; see §6 |
| The **MM key** (counterparty in the swap) | swap liveness, not safety (§4.4) | the fills are the project's own market-making |

Nothing else is trusted: user wallets sign only their own swaps, token contracts are
adversaries by assumption, and the registry never trusts the guard or vice versa (the guard
is a plain consumer of `getRecord`).

## 3. Registrar-key compromise — blast radius

The registrar is the system's single privileged identity, so its compromise is the
interesting case. What an attacker holding the registrar key CAN do:

- **Write false VERIFIED records** — bless arbitrary tokens, including impostors.
- **Revoke genuine records** — deny service to real tokens.
- **Re-point the registrar** (`transferRegistrar`) — lock the project out until the
  project uses... nothing on-chain; this is the one irreversible action (§3.2).

What the same attacker CANNOT do, no matter what:

- **Steal escrowed funds.** The guard never gives the registrar custody, pause, or admin
  power over the swap. Settlement moves tokens only between buyer and MM counterparty, in
  exact amounts, after the checks.
- **Bypass the live probes.** `GUARD_PAUSED` and `GUARD_BLOCKLISTED` are read from the
  token at execution time, not from the record. A token the registrar blessed cannot swap
  while paused or while the buyer is blocklisted.
- **Force a swap of a drifted token.** `GUARD_IMPL_MISMATCH` compares the token's live
  implementation against the record's `impl`. A false `impl` pointer makes the check fail
  (revert), not pass — the failure mode of lying in the `impl` field is denial of service,
  never a settled bad swap.
- **Fabricate history.** Every write emits `Verify`/`Revoke` events — the lie is public
  the moment it is written, and `verifiedAt`/`registrar`/`revokedAt` attribute it.

So the honest summary: **registrar compromise buys verdict lies and revocation DoS — not
fund theft.** The guard is constructed at deploy with a fixed registry address, has no
admin functions, no upgrade path, and no registrar-aware code; its trust in the registrar
is exactly "the record's four fields are accurate claims about the issuer's list".

### 3.1 Why a single registrar is acceptable for v1

- **The writer is a program, not a person.** The key lives only as a Cloudflare Worker
  secret (the drift cron and verification writes sign with it); it is never in the repo,
  the frontend, or any client. No human key ceremony to leak.
- **Writes follow fetched ground truth.** Verification is not an opinion; it is a
  transcription of the issuer's live list, re-fetchable by anyone at any time. A wrong
  record is detectable by any third party with the same public source.
- **Bounded blast radius (§3).** The worst case is wrong metadata and DoS — the funds path
  does not pass through the registrar.
- **Public audit trail.** Every write is an event; the UI shows the registrar address so
  integrators know whose claims they read.
- **Recovery is one transaction.** `transferRegistrar` re-points writes; the backend keys
  can be rotated by the operator and the pen re-assigned.

The accepted residual risk: between a compromise and its detection, the attacker can mint
false VERIFIED records, and `transferRegistrar` by the attacker is final (only the attacker
could transfer it back). v1 accepts this because the registry gates a demo-scale swap
surface with no protocol-owned liquidity; the roadmap answer is a multisig/timelocked
registrar and, later, issuer-signed writes so the registry's trust root converges with the
issuer's.

### 3.2 Revocation as the safety mechanism

Revocation is designed as the recovery tool, not an edge case:

- The drift cron auto-revokes on beacon upgrades — the demo's "upgrade → auto-revoke →
  guard refuses" beat is the safety design working, on purpose, on the main chain.
- A wrong or compromised verification is one revoke away from inert; the guard refuses
  REVOKED deterministically, before any movement.
- Fail-closed bias overall: the guard's registry read failing **closed** (`GUARD_NO_RECORD`)
  is the only failure that stops a swap; unknown tokens are UNVERIFIED in the scanner and
  no-record in the guard — the system's default answer to "I don't know" is always "no".

## 4. Guard attack surface (fresh-eyes audit, step 7)

### 4.1 Reentrancy

- **Probes cannot reenter meaningfully**: every probe is a `STATICCALL` — it cannot mutate
  guard or escrow state even if the "token" is attacker code.
- **`execute()` order**: the order is deactivated *before* any checks or settlement, so a
  reentrant `execute()` (from a transfer hook in the settlement calls) finds no active
  escrow and reverts `EscrowNoOrder`; a reentrant `commit()` finds the (still stored) order
  active and reverts `EscrowOrderActive`.
- **`commit()` order**: the order is stored *before* the custody `transferFrom`. A reentrant
  call from a malicious `tokenIn` sees an active order — it cannot double-escrow (active
  check) and any revert anywhere rolls the entire transaction back (EVM atomicity), so the
  stored-but-unfunded state is unreachable outside the reverted tx.
- **Both settlement calls are strict**: revert data passes through untouched; an explicit
  `false` reverts (`TokenTransferFailed`). No error is swallowed anywhere in the guard, so
  every failure unwinds the whole swap.

### 4.2 Adversarial tokens

| token behavior | outcome |
|---|---|
| `transferFrom`/`transfer` reverts | raw revert data surfaces in `TokenCallFailed`; tx reverts, zero movement |
| returns `false` (non-reverting ERC-20) | `TokenTransferFailed`; tx reverts |
| empty returndata (non-standard) | accepted — deliberate, non-refusing tokens are not punished |
| fee-on-transfer | guard receives less than `amountIn`, so the exact-amount release reverts — whole tx reverts, **no partial escrow can persist** |
| lies on its probe surface (e.g. `paused()` answers `false` while paused) | probe degrade is fail-**open** for that one heuristic — accepted (§4.3); the registry record and the impl-binding check still apply |
| unresolvable / no beacon shape | blocklist + impl checks **skip**, never guess (shape-extraction failure degrades silently) |

### 4.3 Fail-open vs fail-closed, deliberately split

- **Registry read: fail closed.** A registry that will not answer stops the swap
  (`GUARD_NO_RECORD`) — the record is the safety-critical input.
- **Live probes: fail open (degrade to "no flag").** Heuristics never revert (spec
  discipline: reverts only on deterministic red flags). A token that breaks its own probe
  surface cannot DoS the guard, and the guard cannot be tricked into a *revert-griefing*
  denial of service via probe weirdness — worst case, the swap settles without that
  heuristic's protection, which is exactly the honest-degrade semantics the scanner's
  report shows the user.

### 4.4 Escrow / MM-key trust

The counterparty (MM) key grants the guard a standing `tokenOut` allowance; the guard pulls
exactly `minOut` from it and releases exactly `amountIn` to it. A malicious or dead MM can
make swaps **fail** (never fund, overdraw is impossible — `transferFrom` is allowance- and
balance-bounded) — liveness risk, zero safety risk. One active order per buyer prevents
escrow-row overwrites; the guard holds at most one escrow per buyer at a time, so the
release amount is always exactly what custody took.

### 4.5 Error taxonomy — completeness check

Every revert site in the guard maps to exactly one error:

| error | site |
|---|---|
| `GUARD_ZERO_CONFIG` (`Error(string)`) | constructor, zero registry/counterparty |
| `EscrowAmountZero` | `commit`, zero amount |
| `EscrowOrderActive` | `commit`, active order exists |
| `EscrowNoOrder` | `execute`, no active order |
| `GUARD_NO_RECORD` … `GUARD_IMPL_MISMATCH` (`Error(string)`, byte-exact, pinned in `packages/shared/abi.ts`) | `check_token` via the pure decision table, fixed order |
| `TokenTransferFailed` | settlement ERC-20 returned `false` |
| `TokenCallFailed` | settlement ERC-20 reverted (raw revert data carried) |

The five pinned strings are reserved for red flags; internal invariants use the dedicated
`Escrow*`/`Token*` errors, so a client can pattern-match the five without false positives.
The registry's taxonomy is three errors (`NotRegistrar`, `NoRecord`, `ZeroRegistrar`), each
with exactly one trigger; its read path never reverts.

### 4.6 Overflow / arithmetic posture

The guard performs **no arithmetic on user values**: `amountIn`/`minOut` are moved verbatim
(escrowed amount is released exactly; `minOut` is the buyer's chosen exact fill — the v1 MM
honors quoted fills, so there is no slippage computation and no oracle to disagree with).
The only arithmetic anywhere is the 16-byte scan window offset in beacon extraction (on
code length, `usize`, saturating via `min`) and ABI word slicing (`try_into` with fixed
sizes). Record timestamps are `uint64` — monotone, coarse, and only ever stored, never
compared against each other on-chain. `riskFlags` is an opaque `uint256` copied verbatim.

## 5. Deployment posture

- **Immutable, non-upgradeable.** Neither contract has an upgrade path, proxy, or admin
  function. The safety mechanism for a bad record is revocation (§3.2), not contract
  surgery. Deployment is `cargo stylus deploy` via the pinned script
  (`scripts/deploy/core/deploy.sh`), registrar = deployer at construction, production
  registrar handed over by `transferRegistrar` immediately after (runbook in the step-7
  findings).
- **Size headroom.** Robinhood Chain caps Stylus runtimes at 96 KB (init 192 KB). Measured
  at the step-7 quality pass: registry and guard WASM both land in the ~11–12 KB range —
  under 13% of the runtime cap, and below the 24 KB compressed size that would trigger
  Stylus contract fragmentation (which would complicate verification; see the gas report).
- **Source-verified** on the chain explorers post-deploy (Blockscout; browser-UI fallback
  for the mainnet Cloudflare challenge).

## 6. Chain assumptions (Robinhood Chain 4663)

- **Sequencer screening is official**: transactions from sanctioned addresses are excluded
  from inclusion (docs.robinhood.com/chain/differences-from-ethereum) — the watchdog
  widget's premise, cited by the product, not by the contracts.
- **Ordering is first-come-first-served; fees do not reorder.** There is no priority-auction
  MEV on this chain; `commit`/`execute` sequencing is not auction-extractable.
- **No randomness is used** — `prevrandao` is constant on this chain; the contracts draw no
  entropy at all.
- **`block.timestamp` is the only clock** the registry uses (record timestamps), accepted as
  coarse and monotone; nothing depends on sub-block time.
- **ArbWasm/Stylus precompiles are protocol contracts** on 4663 (activation live-proven in
  the week-1 spike; `cargo stylus check` exit 0 against mainnet).
