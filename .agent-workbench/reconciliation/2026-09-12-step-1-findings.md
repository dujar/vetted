# Reconciliation — 2026-09-12 — step-1 repo-scaffold findings

Round 1. The pass began 2026-09-11 (an earlier reconcile-agent run, interrupted
mid-flight); its plan edits were found intact and uncommitted in the working
tree, verified claim-by-claim against the merged scaffold, and committed by
this round. `> **Revised**` notes inside the plans carry the 2026-09-11 date
of the interrupted pass.

Findings source: `../step-1-repo-scaffold/findings.md` (review APPROVED
round 1, 4 notes disposed in-file). It is the only findings.md in the tree.
Step 1 `done` (merged to main at 5e2ff35); steps 2–10 all `planned`.

## Verification performed

Every factual claim in the folded-in edits was checked against the repo, not
against the findings prose:

- Crate header shape (`contracts/core/hello/src/lib.rs`): two `cfg_attr`
  lines (`no_main`, `no_std`) + `#[macro_use] extern crate alloc;` — as cited
  in steps 2 and 3.
- `packages/shared/abi.ts`: `REGISTRY_ABI` / `GUARD_ABI` / `SELECTORS` /
  `GUARD_REVERT_REASONS` exported literals; SELECTORS match the pinned values
  (getRecord `0x617fba04`, verify `0x73c7cf61`, revoke `0xafd0224b`, commit
  `0x498ab631`, execute `0x61461954`); `PROBE_SELECTORS` reserved block at
  :49/:57 — cited in steps 2, 3, 5, 8.
- `frontend/src/components/verdicts.ts` — the duplicated verdict union is
  under `components/`, not `lib/`; the step-5 switch comment is in-file.
- `frontend/src/lib/wagmi.ts`: lazy `getWagmiConfig()`, env-gated on
  `VITE_WALLETCONNECT_PROJECT_ID` defaulting to `""` (zero connectors, no
  throw) — step 5 note 2, step 10.
- `frontend/src/lib/chains.ts` + `packages/shared/wire.md` chains table:
  4663 (`eth_chainId → 0x1237`), 46630 (`rpc.testnet.chain.robinhood.com`,
  `→ 0xb626`), 421614 (viem `arbitrumSepolia` re-export) — cited in steps
  3, 4, 5, 6, 7, 8.
- `scan-backend/wrangler.toml`: `worker-build@^0.8` → `build/index.js`
  (knowledge workers-rust.md "no separate build step" is stale) — steps 2, 4.
- `deployments/README.md` writer pins: `421614.json`/`46630.json` → step 3;
  `4663.json` → step 7 — steps 3, 6, 7, 8.
- `contracts/Cargo.toml`: members `core/*`, release profile at workspace
  root — step 3 note 2.
- `e2e/.gitkeep`, `demo/.gitkeep` present — steps 6, 8.
- Task-number references in every Revised note match that plan's own Tasks
  section (step 3: T1/T3/T5; step 4: T1/T4; step 5: T1/T3; step 6: T7;
  step 8: T1/T3/T6).

## Per-step changes (each carries a `> **Revised**` marker in the plan)

- **step-2-spike-gate**: learned line → step-1 findings; probe crate must
  copy the hello crate header; fetch script uses `worker-build@^0.8`
  (knowledge stale) and deploy credentials are proven; RPC discovery work
  dropped (chains pre-probed); merge-time `PROBE_SELECTORS` replacement must
  keep the shared vitest suite green.
- **step-3-contracts-core**: learned line; crate header for registry+guard
  (T1/T3); workspace members glob — no workspace edit; 46630 RPC already
  probed (T5); pinned writer of `deployments/421614.json`/`46630.json` with
  the step-6 parallel-append conflict note; signatures must match `abi.ts`
  byte-for-byte; serde `#[serde(rename)]` settles wire revert naming
  (inherited by the GATE:FAIL Solidity branch).
- **step-4-scan-backend**: learned line; the deployed hello worker already
  lives in `scan-backend/` — reuse its crate shape and `wrangler.toml`,
  `worker-build@^0.8` path, credentials proven; `wire.md` source-of-truth +
  append-point rules + serde rename; RPCs pre-probed (T1).
- **step-5-frontend-screens**: learned line now points at step-1 findings
  (was "none"); verdicts.ts mirror is at `components/` — switch to
  `vetted-shared` at api-client wiring; zero-connector boot behavior when
  `VITE_WALLETCONNECT_PROJECT_ID` is unset (the only outstanding operator
  action anywhere); chains.ts consume-don't-redefine; `abi.ts` constants
  import-only.
- **step-6-replica-assets**: learned line; rehearsal RPCs pre-probed (T7);
  deployments append rules + parallel-writer conflict note; `demo/` exists
  (`.gitkeep`) — `demo/assets.md` is a create.
- **step-7-contract-hardening**: learned line; pinned writer of
  `deployments/4663.json` (append-only); 4663 RPC pre-probed.
- **step-8-e2e-journeys**: learned line; import `SELECTORS` /
  `GUARD_REVERT_REASONS` literals, never retranscribe (T3); fixture-loader
  writers + 46630 RPC (T1); `e2e/` survives clean checkout, on-demand run
  mode deliberate (T6).
- **step-9-demo-harness**: learned line; runbook URLs from `wire.md` +
  `deployments/{worker,pages}.json`; worker+pages live since step 1;
  local-TLS probe caveat before rewriting runbook fallbacks.
- **step-10-deploy-submission**: learned line; env audit needs no
  deploy-credential work (OAuth token scoped); the only operator action is
  the WalletConnect projectId; off-machine re-check before declaring a URL
  dead.

- `step-feature-state.md`: step 1 row planned → done — caller bookkeeping
  from the merge, found in the working tree and preserved (not this round's
  edit; not modified further).

## Judge round and closure

- plan-judge-agent round 2 (dispatched by the caller after a3159d7; verdict:
  `.agent-workbench/plan-judgment.md`): **SHIPPABLE** — "the reconciled set is
  internally consistent, the dependency graph matches actual file behavior,
  the gate branch survives, and the three round-1 gaps are closed in merged
  or planned text. No new gaps introduced by a3159d7." Its three round-1 gaps
  (step-8 4663 dependency, step-1-ships-read-ABI, registrar-key handoff) were
  all verified closed — the ABI and handoff gaps by scaffold/plan text this
  reconciliation folded in.
- `reconciled: 2026-09-12` stamped on `../step-1-repo-scaffold/findings.md`
  after the verdict (commit with this line).
- Review note 2 (step-1 plan.md:45 stale `blocked-auth` parenthetical) stays
  ACCEPTED AS IS per the findings disposition — a done step's plan is a
  historical record and was not touched.
- Operator action (not a plan item): `VITE_WALLETCONNECT_PROJECT_ID`
  (WalletConnect Cloud projectId).
