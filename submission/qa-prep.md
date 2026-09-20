# submission/qa-prep.md — judges' Q&A prep

Rehearsed answers, in the order they are likely to be asked. Q1 is carried
to Q&A by coordinator decision (plan Revised 2026-09-21 note 5) — it is the
first thing a careful reviewer of the guard will find, and it is disclosed
in the README security note before they ask.

## Judged criteria → where the evidence lives

| criterion | evidence |
|---|---|
| Contract quality | `contracts/core/` (Canonical Registry + Guarded Swap, Rust/Stylus); 10,000-case fuzz/invariant suites with fixed ChaCha seeds (registry: model-based write-semantics property; guard: red-flag priority totality, probe-degradation monotonicity, extraction robustness); on-chain receipt methodology + ≤200k gas budget (`docs/gas-report.md`); published criteria + threat model (`docs/criteria.md`, `docs/threats.md`) |
| PMF | the registry's public read interface (`getRecord`, selector `0x617fba04`, never reverts) as a primitive wallets/aggregators consume; the watchdog widget as the recurring-audience hook; the J3 registry page |
| Real problem solving | live 4663 verdicts — including on the issuer's own token (deployed before this project existed, never touched by us); the honest degraded branch (rate-limited probes → UNVERIFIED with partial evidence, never fabricated); impostors answered by enforcement, not by guessing |
| Innovation / demo | the beat no incumbent has: beacon-upgrade → registry auto-revokes → the guard refuses the formerly verified token at execution time (`demo/runbook.md` beat 6); revocation-as-safety |

## Q1 — the guard's `erc20()` junk-decode (CARRIED, disclosed)

**The flaw.** `contracts/core/guard/src/lib.rs:497`: the `erc20()` helper
decodes a token call's returndata as a bool by checking only byte 31. Any
payload whose 31st byte is set — e.g. 64 bytes of junk — decodes as `true`.

**Consequence, precisely bounded.** A token that lies this way can make a
pre-execution check (e.g. allowance) read "approved" when the custody
`transferFrom` did not move funds into escrow: the buyer is left with a
phantom active order. When that order is later executed against a lying
token, `execute()` reverts whole — **funds are safe**; the loss case is a
do-it-over, not a loss of assets. It cannot steal, redirect, or strand a
counterparty's funds, and it cannot produce a wrong verdict (the scanner's
evidence path does not use this helper).

**Why it shipped.** Found in step-7 review, after the audit-fuzz suite was
green. Tightening the decode is a behavior change to a frozen, reviewed
contract; at freeze we disclose instead of hot-fixing — the review rule
("product code frozen except blocking fixes") wins over the itch. It is a
non-blocking flaw with a bounded, funds-safe worst case; blocking on it
would have traded a disclosed edge case for an unreviewed diff.

**The fix path.** Strict ABI bool decode (reject non-`0x00…01`/`0x00…00`
payloads, wrong-length returndata) + regression tests pinning junk bytes at
every offset — queued as the first post-freeze reviewed change (v1.1), not a
tag-time edit.

## Q2 — a fresh official redeployment reads IMPOSTOR until verified

Ground truth is the issuer's live list, at exact addresses. A fresh official
redeployment sits at a new address the list doesn't carry yet: same
name/symbol at a different address is mimicry **by the issuer's own
definition** (the docs page's rule), so IMPOSTOR is the honest reading until
the registrar verifies the new address. That lag equals the docs page's own
update lag — the incumbent answer has the same property with no tooling at
all — and it closes with one registrar `verify` (or automatically, if the
backend's canonical-list re-check drives it). Positive evidence only; the
verdict links the list rows it compared.

## Q3 — impostor twins live on the production chain

The demo cast is self-deployed on 4663 because that's where real users and
real ground truth are — lookalikes already exist in the wild on this chain,
and ours are labeled demo artifacts (`demo/assets.md`, README demo-cast
labeling, on camera if asked): no liquidity, no holder base, placeholder
issuer. They carry no new risk — and they double as live evidence: the
engine never guesses IMPOSTOR-red on them (they're off the issuer list, so
they scan UNVERIFIED with the depth boundary stated), and the guard refuses
them on-chain at execution (`GUARD_NO_RECORD`).

## Q4 — why not EAS?

Checked against the canonical deployments registry (2026-09-11):
EAS **is** deployed on Arbitrum Sepolia (EAS proxy
`0x2521021fc8BF070473E1e1801D3c7B4aB701E1dE`, SchemaRegistry
`0x45CB6Fa0870a8Af06796Ac15915619a0f22cd475`) and is **not** deployed on
Robinhood Chain 4663 at all (`eas-contracts` `deployments/` lists 26 chains,
no 4663; spike evidence `spike/evidence/eas_arbitrum_sepolia.json`).
Independently of availability: EAS stores attestations — it does not
interpret, enforce, or revoke them. This registry is load-bearing inside the
swap: the guard reads the record in-tx, binds it to live probe results, and
honors revocation tied to beacon upgrades — semantics EAS does not provide.
EAS could become an integration path underneath a later version; the
revocation + probe-binding semantics are the product. (Wording discipline:
never "attestation" — on Ondo's chain that word means EIP-712 trade quotes.)

## Q5 — why not GoPlus?

GoPlus **does** list 4663 (`supported_chains` returns `Robinhood / 4663`,
live-verified) — we say so; "first coverage" is a banned phrase. The spike
re-test against the issuer's genuine token (`spike/evidence/goplus_token_security_4663_P.json`,
live 2026-09-11) returned only 16 descriptive fields with **every risk field
absent** — no `is_honeypot`, `transfer_pausable`, `is_blacklisted`,
`hidden_owner`, … — and GoPlus's own docs say risk fields "will not be
returned … sometimes when `is_proxy`: 1". The stock tokens are beacon
proxies: **coverage arrived, correctness on this chain's pattern is
structurally absent.** Our line: *first correct coverage-shaped answer for
this chain's pattern — coverage without enforcement was already here;
enforcement was not.* What carries the scan half is the power disclosure,
revocation, and enforcement — the three things a generic scanner doesn't do
here. (Rehearse with the saved JSON on screen.)

## Q6 — why not Blockaid (or another B2B scanner)?

Blockaid is enterprise B2B: no 4663 support, no public per-token verdicts,
and its homepage phrase "trust layer" is banned in all our copy. Nothing to
integrate for a retail holder on this chain; our verdicts are public,
evidence-linked, and enforced on-chain.

## Q7 — Token Sniffer / De.Fi (pre-submission spot-check, spec.md:71)

Their capabilities were unverified at research time — the checklist is a
browser spot-check on a genuine 4663 token (the issuer's own
`0x1Cdad396DB64BDa184d5182A97Dd9B3C62100b7D` works unfunded), recorded in
the post-funding checklist. **Rule:** until the spot-check runs, cite no
specifics about either tool; if either turns out to catch the beacon+
hidden-modifier pattern, the Q&A copy shifts to the enforcement/revocation/
power-disclosure branch (same as the GoPlus branch) and this file gets one
sentence added under Q5.

## Q8 — what if the registrar key is compromised?

`docs/threats.md` is the full model; the one-line answer: the registrar can
lie about verdicts and revoke records (a DoS on trust, visible and
recoverable — re-verification reinstates), but it cannot move user funds:
the guard escrow only ever settles swaps the users signed, probes are
read-only, and there is no admin path into token balances. Single-registrar
is a deliberate v1 simplicity; the blast radius is bounded and published.

## Q9 — the watchdog numbers don't match the published figures

Correct, and we published the discrepancy instead of smoothing it: our L1
read of the SequencerInbox counter (`0xBd0D173E…ba96`) measured 258,707
batches with delivery-style events at roughly 2,630/day (2026-09-20) against
the published ~150/day six-week baseline. No read reconciles the two, so the
widget ships the designed degrade sentinel (`runs: 0` + `provenanceUrl` =
"unavailable", never a fake zero) and narrates measured figures WITH
provenance. Pinning a reconciled counter mechanism is the known follow-up.

## Q10 — how do you know a swap stays under 200,000 gas?

We don't estimate — we prove it from broadcast receipts. The deploy script's
receipt stage runs 11 on-chain `execute()` transactions (five byte-exact
reverts, degraded skip, clean settle), hashes them against expected rows,
and enforces per-row `gasUsed ≤ 200,000`, aborting the deploy on any miss
(`docs/gas-report.md` §2). Unfunded at submission, the file honestly reads
PASS-PENDING-FUNDS with the no-funds measurements committed (wasm sizes,
ArbWasm data fees, cost model — §3); the receipts are never invented.

## Rehearsal ritual (before recording / before judging)

- [ ] Q1 spoken aloud once, ending on "funds safe, disclosed, fix queued".
- [ ] Q4 with the two Sepolia addresses on screen; Q5 with
      `spike/evidence/goplus_token_security_4663_P.json` open.
- [ ] No "trust layer", no "first coverage", no "attestation".
- [ ] Q7 status re-checked: spot-check done → no specifics needed; not done
      → still no specifics.
