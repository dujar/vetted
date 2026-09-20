# Step 6 — replica assets: plan verification (2026-09-19)

Verified against the repo at 43efd99 (main, clean). No CLAUDE.md/AGENTS.md exists. Read in full:
plan.md incl. both `> **Revised**` blocks; spike/findings.md; step-1 findings; step-3/4/7/9 plans;
reconciliation/2026-09-19; plan-judgment.md; step-feature-state.md; knowledge robinhood-stock-tokens.md
+ contract-verification.md; spike/evidence/calibration_4663.json; packages/shared/abi.ts;
.github/workflows/ci.yml; .gitignore; contracts/Cargo.toml; deployments/README.md; spike/replica/*.

Hard gates: neither trips. Screens N/A is honest (contracts + manifest, no new surface). "User
journey N/A" is correct for demo assets — the manifest's consumers (step 8 fixture loader reads
`demo/assets.md`, step 9 runbook) are named in their own plans.

Dangling references: none. All cited spike artefacts exist (`spike/findings.md`,
`spike/evidence/calibration_4663.json`, `spike/replica/{src,script,test}`, `spike/DEPLOY.md`);
`abi.ts:60-63` PROBE_SELECTORS match the plan's `0x5c975abb`/`0xfbac3951` and the calibrated targets
(paused → proxy, isBlocked → beacon) match abi.ts:49-59 and step-4 Revised note 1; verifier URLs in
task 7 match knowledge/contract-verification.md:13-15 verbatim; RPC pins claimed in Revised note 1
exist (wire.md:89-90, frontend/src/lib/chains.ts:16,29); naming/decimals match knowledge
robinhood-stock-tokens.md:23-31; journeys.md:9 compare deep link and :32 revert reasons verified.
`contracts/Cargo.toml` members = `core/*` with an explicit comment that `replicas/` is step 6's own
foundry project, not a member — no cargo-workspace collision with step 3.

## Loose ends

```
[deferred decision] Joint fingerprint composition with step 4 is not concretely defined anywhere
  where:    plan task 3 ("subject to Revised note 3"), task 4, task 5 / plan.md:18,41-42
  evidence: step-4 plan Revised note 3 and step-6 Revised note 3 each defer to the other ("composed
            jointly"); no repo file states the hit/miss list. The raw calibration makes two candidate
            checks impossible for ANY replica: exact codehashes (proxy 0x6c1fdd40… embeds the genuine
            beacon address 0xe10b6f…1b00 — calibration_4663.json "structure" field; a replica's own
            beacon address differs, so its bytecode can never byte-match) and uid() == /rhj/assets id
            (spike/findings.md:25 calls uid() "the strongest single check"; "Aurelia Industries" is
            not among the 194 assets). Plan tasks 2-3 never mention uid() — if step 4's composed list
            keeps it, the replica is UNVERIFIED forever and the compare beat breaks; if it drops it,
            tasks 2-3 are fine. The twins' deterministic failure likewise depends on which checks
            compose "match" (plain ERC-20 fails trivially; the self-proxy twin fails only if
            beacon-slot-set/impl-slot-empty is required). plan-judgment.md:67's "step 8's e2e detects
            divergence" is detection after the build, not a definition.
  fix:      before tasks 3-5, write the composed check list into one named artifact (e.g. step 4's
            matcher fixtures + mirrored in step 6's forge test), explicitly excluding codehash
            byte-equality and uid()==registry-id; minimum viable pattern list: EIP-1967 beacon slot
            set + impl slot empty; paused() answers on proxy, isBlocked(address) answers on the
            resolved beacon (calibrated targets); required selector subset from
            calibration_4663.json fingerprint_selectors; solc 0.8.x. Both twins fail on slot layout +
            probe-target checks.
[missing prerequisite] Step 9's upgrade beat has no live upgrade target
  where:    plan tasks 3, 6, 7 / step-9-demo-harness/plan.md:26,34 (task 3 "script the beacon-upgrade tx")
  evidence: step 6 builds and unit-tests upgradeTo, but task 7 deploys only "replica + twins" — no
            second implementation is deployed to 4663 or listed in demo/assets.md; step 9's creates
            are demo/** + scripts/seed-demo/ only (its plan and tracker row 9), so it cannot deploy
            contracts. Nobody owns the address upgradeTo points at during the demo.
  fix:      add to task 7: deploy the upgrade-target impl (a second DemoStockToken instance — the
            registry records impl as an address, so same-code-different-address drifts the record)
            alongside the cast and list it in demo/assets.md.
[untouched sibling] deployments/4663.json writer pin vs task 7's 4663 deploy
  where:    plan task 7 / deployments/README.md:13 / step-feature-state.md row 6
  evidence: README pins writer of 4663.json = step 7 (later window, Sep 27-29); step 6 deploys to
            4663 and appends "deployments/<chain>.json". Revised note 2 and plan-judgment.md:41 cover
            only the 421614/46630 parallel-with-step-3 pair; tracker row 6's files-in-scope omits
            deployments/*.json entirely.
  fix:      one line in task 7 naming the 4663 destination (step 6 creates 4663.json with a replicas
            field, or a dedicated deployments file) + update the README writer column accordingly.
[no check] forge tests are not runnable in CI — plan adds no job
  where:    plan task 1, Check line / .github/workflows/ci.yml
  evidence: the contracts CI job is cargo-only (wasm build + stylus native tests); no job runs forge;
            the plan's "forge test green" has no CI wiring and ci.yml is not in the plan's creates.
  fix:      add a `replicas` job to ci.yml (foundry toolchain action + forge build/test in
            contracts/replicas). No other parallel step owns ci.yml, so no collision.
[untouched sibling] lib/ is not gitignored; task 1's gitignore instruction is a no-op
  where:    plan task 1 ("gitignore out/, cache/") / .gitignore:16-17
  evidence: root .gitignore already ignores cache/ and out/ unanchored (covers contracts/replicas);
            no rule ignores lib/ anywhere, and the dispatch says never commit lib/ (forge-std + OZ).
            Proven pattern: spike/replica/.gitmodules is tracked (project-local, pins lib/forge-std —
            `forge install` restores from it), lib/ itself untracked.
  fix:      task 1: keep the project-local .gitmodules tracked, add contracts/replicas/lib/ to
            .gitignore (root or project-local); drop the redundant out/cache instruction.
[deferred decision] Task 7 deploy mechanics: key source and funding fallback unstated in plan text
  where:    plan task 7, Check line
  evidence: task 7 never names the deploy key (spike/.env throwaway key) nor the 4663 mainnet funding
            dependency (every faucet headless-blocked, spike/evidence/funding_blockers_20260919.txt;
            operator funding in flight). The Check line hard-requires "verified on the 4663
            explorer" with no ordering/fallback; the dispatch's "build-first/deploys-last with
            runbook fallback" language does not appear in the plan (Revised note 4 covers gas only).
  fix:      one sentence in task 7: build+test first; rehearse on 46630 once the throwaway key is
            funded; run the 4663 deploy + verify when mainnet funds land; if unfunded at step close,
            land code+tests and record the pending deploy in findings.
```

Not findings (checked, clean): demo/ exists with .gitkeep (Revised note 3 correct); demo/assets.md has
consumers (step-8 plan task 1, step-9); file-scope split with steps 3/4 is disjoint per
plan-judgment.md:41,67 and contracts/Cargo.toml; knowledge files contain no contradiction with the
plan; the spike/replica harness is correctly treated as the anti-pattern, not a wheel to reuse.
The "Demo company name" open question carries a default and an owner — acceptable.
