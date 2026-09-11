# Step 1 — repo-scaffold: verification

Checked against: the empty repo (only `.agent-workbench/`, branch main, 1 commit), all
cited knowledge files, step-3/step-2/step-4/step-5 plan texts, `components.html`,
`theme.css`, `journeys.md`, `spec.md`, and the live environment.

## Hard gates

Neither trips. No UI surface is created (primitives derive from existing
`.agent-workbench/product/components.html` + `theme.css`, both present and real markup);
plan explicitly rules the user-journey gate N/A for infrastructure — correct.

## Verified clean

- All five registry/guard signatures cited from step 3 exist in step-3 plan.md:
  `getRecord(token)` / `revoke(token,reason)` (:28), `commit(tokenIn,tokenOut,amountIn,minOut)`
  / `execute()` / five `GUARD_*` strings (:30, = journeys.md:32). Only `verify` is not
  actually fixed — see loose end 1.
- Version pins all match knowledge: stylus-sdk/cargo-stylus 0.10.9, ≥0.10.8 fragment-verify
  fix, Rust ≥1.91, wasm32 single target, worker 0.8.5, vite 8.3/react 19.3/tailwind 4.3.3,
  wagmi 3.7.7 + viem 2.56.3 with AppKit excluded (knowledge frontend-stack.md:14–19 path 2),
  no postcss config, no protoc step. No contradictions found.
- Toolchains present in this environment: rustup 1.29.0, rustc 1.97.1 (≥1.91),
  wasm32-unknown-already-installed, node 26.3.0, npm 11.16.0, wrangler 4.131.1 via npx,
  gh 2.79.0 with `repo` + `workflow` scopes (repo create + push + CI workflows will work).
- `PROBE_SELECTORS` handoff is well defined: step-2 plan.md:16,30 commits the merge-time
  append into the step-1-reserved block; step-5 plan.md:9,30 consumes via the constant.
- Line references to knowledge files check out (stylus-toolchain.md:16–17,24–25,32–33,37–38;
  workers-rust.md:19–20; frontend-stack.md:14–19,26–29; robinhood-chain.md:14–21;
  arbitrum-sepolia.md:11–13 — modulo loose end 3).

## Loose ends

1. [dangling reference] `verify(token,…)` has no fixed signature — the "…" is in both plans
   where:    plan task 6 / Scope (plan.md:21) ← step-3 plan.md:28
   evidence: step 1 claims signatures are "fixed from step-3's plan text", but step-3's own
             text writes `verify(token, …)` with an ellipsis. `abi.ts` needs exact
             human-readable entries (viem parseAbi cannot encode "…"); the implementer
             would have to invent params step 3 then must match.
   fix:      fix the exact parameter list now (the record fields imply it, e.g.
             `verify(address token, uint256 riskFlags, address impl)`), carried identically
             in both plans before build.

2. [dangling reference] Registry record pinned with 5 fields; step 3 defines 7
   where:    plan Scope (plan.md:20) vs step-3 plan.md:28
   evidence: step 1 pins `{status, risk flags, verifiedAt, implementation pointer, registrar}`
             per spec.md:35; step 3's record is `{u8 status, u256 riskFlags, u64 verifiedAt,
             address impl, address registrar, u64 revokedAt, bytes32 reason}`. J3's REVOKED
             drill-in shows reason + revocation evidence (journeys.md:42) — step 5 consumes
             step-1's types/fixtures and would miss `revokedAt`/`reason`.
   fix:      pin the seven-field shape from step-3 plan.md:28 in step 1's wire-contract bullet.

3. [dangling reference] "three verified RPCs" — only two RPC URLs exist in knowledge
   where:    plan task 5 (`src/lib/chains.ts`) / Scope (plan.md:20)
   evidence: grep across `.agent-workbench/knowledge/` finds RPC URLs only for 4663
             (rpc.mainnet.chain.robinhood.com) and 421614 (sepolia-rollup.arbitrum.io/rpc);
             for 46630 only the explorer URL exists (contract-verification.md:15,21) —
             robinhood-chain.md:15 confirms the testnet probe (0xb626) but never records its
             RPC URL.
   fix:      add to task 5: fetch the 46630 RPC from docs.robinhood.com/chain/connecting and
             live-probe `eth_chainId == 0xb626` before writing chains.ts (or ship two
             verified RPCs + a marked placeholder, resolved in step 3's window).

4. [dangling reference] Primitives "Card" and "skeleton" are not in components.html
   where:    plan task 5
   evidence: components.html sections are: header/nav, network chip, buttons, input+action,
             verdict badges, panel, probe checklist, diff table, stat widget, banners,
             registry table — no card, no skeleton pattern (grep: 0 hits).
   fix:      map Card → the existing `.panel`, and either drop skeleton or state it is built
             from theme tokens (advisory), so the implementer isn't inventing design.

5. [unhandled call site] "route unit test" will panic under host `cargo test`
   where:    plan task 4 (CI backend job runs plain `cargo test`)
   evidence: `worker::Request`/`Response` wrap js-sys objects; wasm-bindgen imported fns
             panic on non-wasm targets, so a test invoking the `#[event(fetch)]` handler or
             constructing a `Request` fails in CI even though the crate compiles.
   fix:      specify the test targets a pure fn (path/health → body+status), with the fetch
             handler as a thin wrapper over it.

6. [no check] Deploy success criteria (tasks 4, 8, Check line) have no fallback for the
   known-bad wrangler credentials
   where:    plan tasks 4, 8; Check (plan.md:42); Open questions (plan.md:45)
   evidence: live probe `wrangler whoami`: logged in, but token scopes are only
             `user (read)` + `offline_access` — insufficient for workers/pages deploys; the
             plan acknowledges deploys are operator-blocked yet the Check line still
             requires "wrangler deploy + pages deploy succeed".
   fix:      define the fallback state: attempt deploys; on auth failure write
             `deployments/worker.json`/`pages.json` with `"status":"blocked-auth"` plus the
             operator's one-liner (re-login with workers+pages scopes or
             `CLOUDFLARE_API_TOKEN`), pass the step on CI-green + tests, mark deploys
             pending in the tracker row.

7. [deferred decision] Public vs private repo left to "operator confirms" blocks task 1
   where:    plan task 1; Open questions (plan.md:46)
   evidence: `gh repo create` needs the visibility flag; gh is authenticated and ready
             (scopes verified), nothing else waits on this.
   fix:      decide public now — the plan already recommends it — and delete the question.

8. [deferred decision] `rust-toolchain.toml` "(stable ≥1.91)" is not a valid file value
   where:    plan task 2
   evidence: rust-toolchain.toml channels are exact (`stable` or a pinned version); "≥1.91"
             cannot be expressed, and `channel = "1.91"` would pin old against knowledge
             stylus-toolchain.md:16–17 ("use current stable, don't pin old").
   fix:      write `channel = "stable"` (local rustc 1.97.1 already satisfies ≥1.91).

9. [deferred decision] WalletConnect projectId "from env" — empty-env behavior unspecified
   where:    plan Scope (plan.md:22); Open questions (plan.md:45)
   evidence: no projectId exists; wagmi's walletConnect connector throws on empty projectId
             at config construction, so a module-scope config can break dev server and any
             test that transitively imports it ("npm test passes locally" is then env-bound).
   fix:      one clause: config lives in its own module created lazily/guarded with a
             placeholder env default; VerdictBanner test imports only the component.

10. [dangling reference] Scan request shape pinned by step 1 but only defined in step 4
   where:    plan Scope (plan.md:20) vs step-4 plan.md:29
   evidence: step 1 pins "scan request/response" as wire contract; the only concrete request
             definition anywhere is step-4's `GET /scan?chainId&addr`. types.ts shaped as a
             POST body would diverge from the backend step.
   fix:      state the request as `{chainId, addr}` query params (step-4 plan.md:29) in the
             wire-contract bullet.
