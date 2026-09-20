# Step 6 — demo replica assets

status:     merged
branch:     step-6-replica-assets (merged into main as 8c80aad via --no-ff; merge suite green on main: forge 14/14, contracts wasm+native, scan-backend 3, shared 15, frontend 84; pushed)
reconciled: 2026-09-20 (folded into plans 7–9 — commits 4b52e6a + 7283ffa; record: reconciliation/2026-09-20-steps-3-4-and-6-findings.md; judge 2026-09-20: 2 gaps, both fixed, accepted)
deployed:   not deployed — 4663/46630/421614 addresses PENDING on funding (throwaway key 0x151e…8E4C read 0 wei on all three at close, 2026-09-19); deploy rehearsed green on local anvil; runbook in demo/assets.md + below

## What was built

The demo cast in `contracts/replicas/` (own foundry project, solc 0.8.28 + OZ
contracts-upgradeable v5.4.0, never imported by the Rust core): `DemoBeacon`
(AccessControl beacon carrying `paused` + `isBlocked` + `implementation()`/
`upgradeTo` — the GENUINE layout, blocklist state on the beacon per plan
Revised note 1), `DemoBeaconProxy` (forwarder embedding the beacon as a
zero-padded PUSH32 immediate, STATICCALL `implementation()` 0x5c60da1b,
DELEGATECALL result — the genuine `p_proxy.hex` shape the step-3 guard
extracts by bytecode shape; writes the EIP-1967 beacon slot once, impl slot
stays empty), `DemoStockToken` (ERC-20 18 decimals; transfer hook reads
pause/blocklist from the beacon resolved via its own proxy slot — fail-closed;
`paused()` answers on the proxy; `isBlocked` reverts through the proxy, the
calibrated hidden-blocklist signature), two impostor twins (`ImpostorPlain` —
anti-pattern copy with in-token pause/blocklist + fake uid();
`ImpostorSelfProxy` — wrong-slot EIP-1967 proxy whose `paused()` answers but
has no beacon), `FINGERPRINT.md` (the composed step-4/step-6 check list, F1–F5,
with explicitly excluded impossible checks) implemented by
`test/Fingerprint.t.sol` (replica passes every check; each twin fails its
named checks while name/symbol mimicry passes), `test/Behavior.t.sol` (all
demo beats + admin gating + the Revised-note-2 probe-target acceptance),
`script/DeployReplicas.s.sol` + `scripts/deploy/replicas/deploy.sh` (v1 +
UPGRADE-TARGET impls per verify loose end 2 — rehearsed green on local anvil,
broadcast receipts committed), `demo/assets.md` (manifest: roles, chains,
twin failure modes, step-9 beat choreography, verify commands), CI `replicas`
job (verify loose end 4), deployments/README.md writer column for step 6
(verify loose end 3). All six verify loose ends applied as prescribed;
14/14 forge tests green after rebase on origin/main.

## Where the plan was wrong

- `forge install` does NOT restore deps from a project-local `.gitmodules`
  inside a git repo: it silently no-ops (and with real submodule installs it
  hoists lib/ to the GIT ROOT, which clobbered the shared root when first
  run — undone). Fix: deps pinned as PLAIN FILES (`foundry.lock` +
  `.gitmodules` committed, `lib/` gitignored) and restored by three pinned
  plain clones (documented in contracts/replicas/README.md, used by CI).
  The nested dep `openzeppelin-contracts-upgradeable/lib/openzeppelin-contracts`
  @ v5.4.0 is also required.
- Plan task 1's "gitignore out/, cache/" was a no-op (root .gitignore already
  covers them unanchored) — replaced with project-local `lib/` ignore per
  verify loose end 5.
- OZ-v5 API traps cost a round each: init fns are `onlyInitializing`
  (constructor calls revert NotInitializing — beacon constructor grants roles
  directly); ERC20Upgradeable has no `supportsInterface`; `string` immutables
  don't exist (twin 2 inits through its proxy); NatSpec `@param` must match
  the exact identifier (`admin_`, not `admin`).
- 421614 rehearsal dropped: no headless funding path exists (spike evidence);
  the runbook keeps the Etherscan-v2 recipe for when the operator funds it.

## What the next step needs to know

- **Step 4 (matcher) must mirror FINGERPRINT.md exactly** — checks F1 solc
  0.8.x metadata, F2 beacon-slot-set + impl-slot-empty, F3 shape-extraction of
  the embedded beacon (scan for a 20-byte window whose staticcall
  `implementation()` returns code; never codehash equality), F4 calibrated
  probe targets (paused→proxy answers; isBlocked→resolved beacon answers and
  reverts through the proxy), F5 selector subset. Divergence either way is
  the blocking defect the verify loose end named.
- **Step 3 (guard)**: beacon extraction by forwarder bytecode shape is proven
  against the compiled replica (test_guard_extraction_path) — the runtime
  embeds the beacon exactly like the genuine PUSH32 zero-padded shape.
- **Step 9**: `demo/assets.md` §"Beat choreography" has the exact cast
  commands; the cast deploys CLEAN (unpaused, nobody blocked, 1M AURE to
  admin) so beats are driven live; implV2's address is the upgrade beat's
  target (already deployed, listed in the manifest).
- Deploy order when funding lands: 46630 rehearsal (throwaway key,
  `scripts/deploy/replicas/deploy.sh`) → append addresses to
  `deployments/46630.json` `replicas` field + demo/assets.md → 4663 (creates
  `deployments/4663.json` with `replicas` field) → forge verify per
  demo/assets.md (Blockscout, trailing `/api/`; browser-UI fallback for the
  mainnet Cloudflare challenge). 421614 needs Etherscan-v2 + key.

## Review round 1 (APPROVED) — non-blocking notes, disposition

1. empty `reconciled:` header — BY DESIGN per this process: the dispatch
   requires findings.md WITHOUT it; the post-merge reconcile round fills it
   (as in steps 1/2/5). Routed to reconcile, not a defect.
2. nested OZ dep not in foundry.lock — FIXED: added
   `lib/openzeppelin-contracts-upgradeable/lib/openzeppelin-contracts` @
   v5.4.0/c64a1ed to foundry.lock (single pin source); forge test green after.
3. foundry-action unpinned — FIXED: `version: v1.8.1` input matches local.
4. run-latest.json churn + duplicate section comment — FIXED: untracked
   `broadcast/**/run-latest.json` (+ gitignored; run-<ts>.json stays the
   stable receipt), stray comment removed.

## Out of scope, left broken

- No live rehearsal/verify receipts: funding pending at close (final poll 0
  wei on 46630/4663/421614). Unblock: operator funds the throwaway key; any
  builder with the key can run the runbook above — code path is
  chain-identical and locally rehearsed.
- Env note: anvil's `-m test test test` subcommand form is invalid in the
  installed foundry 1.8.1 (plain `anvil` works).
- The twins are intentionally ugly (require-string reverts, owner-only
  mint) — plausibility, not quality; do not "fix" them.

## Shared-worktree incident (record for the caller)

The shared checkout at the repo root is being used by steps 3 and 4
concurrently: mid-build, another builder's branch switching + reset wiped my
first commit's tracked files and clobbered the `step-6-replica-assets` ref
(recovered from reflog, commit c3a92b3). I moved to a dedicated worktree
(`/home/bitslicer/projects/vetted-step6-wt`) and all work/commits live there.
My tracked-file scope stayed inside my file scope; I did not touch the
scan-backend/packages modifications seen in the shared tree (steps 3/4 work).
Note for merge: `ci.yml` + `deployments/README.md` edits (verify loose ends
3+4 authorize them; no parallel step owned those files).
