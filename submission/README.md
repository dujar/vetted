# submission/ — the submission package

Target: uploaded by **2026-10-03 EOD SGT** (≥24h ahead of the 2026-10-04
23:59 SGT close, spec.md:69). This directory is the package's table of
contents and the operator's upload checklist; the package itself is the
public repo + live URLs + the recorded video.

**Regime at package build (2026-09-21): UNFUNDED.** The operator key reads
0 wei on 4663/46630/421614 (re-checked at step-10 start). Everything that
does not need chain state is DONE and in this package; everything that does
is the explicit post-funding checklist — it does not block the package from
being finalized the moment funding lands.

## Funding go/no-go

- **Go/no-go date: 2026-09-29.** If the funded run (go-list) has not
  executed by then, funding the window (go-list + smoke + recording +
  package) no longer fits before 2026-10-03 — at that point the coordinator
  either escalates (find a funded wallet the same day) or the **declared
  unfunded variant ships**: the anvil-rehearsed demo harness as the measured
  wall-clock source (`demo/runbook.md` §5), `docs/gas-report.md`
  PASS-PENDING-FUNDS honesty copy, no contract-address section, and the
  README security note's PENDING clause standing as written.
- The unfunded variant is a *declared* ship state, not a failure mode: the
  contracts, engine, harness and docs are complete; the missing piece is one
  funded operator session (`submission/post-funding-checklist.md`).

## Package contents

| item | where | state |
|---|---|---|
| Repo link | https://github.com/dujar/vetted (public) | live |
| README (final) | [`/README.md`](../README.md) — carries the PROMINENT security note (guard `erc20()` junk-decode disclosure + known limits) | done |
| Security disclosure | README "Security note" section + [`qa-prep.md`](qa-prep.md) Q1 | done |
| Q&A prep | [`qa-prep.md`](qa-prep.md) — judged-criteria mapping, the erc20() answer, why-not-EAS, why-not-GoPlus/Blockaid, residual risks | done |
| Video | script: [`/demo/video-script.md`](../demo/video-script.md) (1:1, timed); final take + `demo/demo.gif` | post-funding |
| Live URLs | below | live (2 of 4 product URLs; contract URLs post-funding) |
| Contract addresses | `deployments/4663.json` (created by the funded run) | PENDING-FUNDS |
| Criteria / threats / gas | `/docs/criteria.md`, `/docs/threats.md`, `/docs/gas-report.md` | done (gas §4 PENDING-FUNDS by design) |
| Production state audit | [`production-state-audit-2026-09-21.md`](production-state-audit-2026-09-21.md) | recorded (unfunded state) |
| Post-funding completion | [`post-funding-checklist.md`](post-funding-checklist.md) | staged, executable top-to-bottom |

## Live URLs (as of 2026-09-21)

| URL | state |
|---|---|
| Frontend (Pages) — https://vetted-1un.pages.dev | HTTP 200, step-5/8 bundle |
| Scan backend (Worker) — https://vetted-scan-backend.dujar-coding.workers.dev | `/health` ok; watchdog degrade sentinel exact; `/scan` honest-degrade verified |
| Canonical-fetch fallback mirror — https://vetted-spike-canonical-fetch.dujar-coding.workers.dev | `/assets` ok (194 assets) |
| 4663 explorer (contracts) — https://robinhoodchain.blockscout.com | resolves; contract pages PENDING-FUNDS (deploy + verify land with the funded run) |
| 421614 mirror explorer — https://sepolia.arbiscan.io | resolves; mirror deploy PENDING-FUNDS (4663 takes priority if only one chain is funded — record the mirror check N/A, don't force it) |

## Contract addresses

Filled by the funded run into `deployments/4663.json` (registry, guard,
replicas) and read into the frontend env — see the post-funding checklist
steps 1–2. Until then this section honestly reads **PENDING-FUNDS**; the
unfunded variant ships without it.

## Repo visibility (coordinator decision, 2026-09-21)

The repo stays **public with the full `.agent-workbench/` build narrative**
(plans, verify verdicts, review rounds, reconciliation records) — accepted
as public evidence of the build process, no scrub. Contingency: if this
decision ever flips to scrub, the P-receipt asset is COPIED into
`submission/` rather than linked at its workbench path (judge caveat,
recorded in the checklist).

## Freeze + release tag rules

- Product code is **frozen except blocking fixes** from this step onward.
  The three dead test helpers in the guard test target
  (`contracts/core/guard/src/lib.rs` — `BEACON` :655, `mock_record` :706,
  `install_beacon_token` :733) print dead-code warnings under
  `cargo test -p guard`; they are dispositioned follow-up cleanup, NOT a
  freeze blocker, and are deliberately not touched at tag time.
- `v1.0.0` is cut at the END of packaging, on the submitted tree, via
  `scripts/release/cut-release-tag.sh` (verifies clean tree + CI green on
  the main push of the tagged commit + a clean-checkout build). CI is read
  off the main push of the tagged commit.
- A blocking fix after the tag NEVER re-cuts v1.0.0 — it bumps **v1.0.1**
  with the same script, and the submission links point at the new tag.

## Upload checklist (operator; portal mechanics are not a plan dependency)

- [ ] Post-funding checklist complete (or the unfunded variant declared by
      the coordinator after 2026-09-29).
- [ ] Final video rendered from the best ≤5:00 take; hosted where the form
      asks (or attached); `demo/demo.gif` cut from beats 4–6 and the README
      link uncommented.
- [ ] Form fields: repo link, live URLs (table above), contract addresses
      (or the declared unfunded note), team/contact, track eligibility
      (Robinhood Chain project — 4663 deploy is the reserved-prize key;
      unfunded variant must say so explicitly and point at the staged
      deploy path).
- [ ] `v1.0.0` tagged, pushed, CI green at the tagged commit.
- [ ] Submission confirmation screenshot/receipt saved to `submission/`.
