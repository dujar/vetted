# Step 6 — Demo replica set: faithful-pattern stock token + impostor twins (Solidity, on 4663)

## Resources
- spec:    ../product/spec.md   (Scope 5 spec.md:38; Risk 6 mitigations spec.md:80; demo beats spec.md:39)
- journeys: ../product/journeys.md  (compare deep link journeys.md:9; J2 revert beat :32)
- knows:   ../knowledge/robinhood-stock-tokens.md  (genuine naming/decimals :20–33; the beacon+blocklist+pause pattern's provenance is xroot.dev, third-party — the replicas ARE its verification path :34–39), ../knowledge/contract-verification.md  (Blockscout recipes + the 4663 Cloudflare-challenge footgun :19–27)
- learned: ../step-2-spike-gate/findings.md  (calibrated fingerprint the replicas must reproduce; 4663 gas verified; beacon smoke result — must exist before this step starts)
- learned: ../step-1-repo-scaffold/findings.md  (scaffold MERGED at 5e2ff35 — live-probed RPCs, deployments append-point rules, `demo/` reserved)

> **Revised** — step-1 findings reconciliation (2026-09-11):
> 1. Task 7's rehearsal targets: RPC `https://rpc.testnet.chain.robinhood.com` (46630, `eth_chainId → 0xb626`) and the 421614 endpoint are already live-probed and pinned in `packages/shared/wire.md` chains table + `frontend/src/lib/chains.ts` — no endpoint discovery.
> 2. `deployments/<chain>.json` appends: the pinned writers per `deployments/README.md` are step 3 (`421614.json`/`46630.json`) and step 7 (`4663.json`); this step runs in parallel with step 3 and appends replica fields to the same files — append-only, take the small conflicts at merge (tracker Next §4).
> 3. `demo/` exists in the tree (`.gitkeep` added in review round 1) — `demo/assets.md` is a create, not a dir bootstrap.

## Stack
Solidity 0.8.x + OpenZeppelin Upgradeable (beacon), Foundry for tests + deploys. Replicas are **demo assets, deliberately isolated from the Rust core** (spec.md:38) — a separate foundry project under `contracts/replicas/`, never imported by core.

## Goal
After this step, 4663 carries the demo cast: one genuine-pattern replica (beacon proxy, hidden per-address blocklist in the transfer modifier, global pause) that the registry will verify as canonical, plus two impostor twins sharing its name/symbol — so every demo beat (impostor scan, guarded refusal, upgrade→revocation→refusal) runs against real live addresses, and the decoys are themselves live IMPOSTOR evidence (spec.md:80).

## Scope
- Creates: `contracts/replicas/**` (own foundry project — dir reserved by step 1), `scripts/deploy/replicas/`, `demo/assets.md` (the demo-cast manifest: which address is the canonical replica, which are twins).
- Out of scope: core contracts (step 3); registrar seeding of the replica's canonical record (step 9 — needs the deployed registry); anything in the core toolchain.

## User journey
N/A — demo assets; consumed by J1's compare beat and J2's revert beat (spec.md:39).

## Screens
N/A.

## Tasks
1. Foundry scaffold in `contracts/replicas/` (`forge init`, OZ remappings); gitignore `out/`, `cache/`.
2. `DemoStockToken` impl: ERC-20, 18 decimals (knowledge robinhood-stock-tokens.md:31); global pause flag + `paused()`; per-address blocklist checked inside a transfer-hook modifier with a non-obvious setter (the hidden-modifier pattern per the step-2-calibrated fingerprint); admin-gated everywhere. Name/symbol follow the genuine shape `"{Company} • Robinhood Token"` (knowledge :24–25) but with a demo company (placeholder "Aurelia Industries") so repo + narration can label the cast honestly (spec.md:80).
3. Beacon: `UpgradeableBeacon` + `BeaconProxy` deploy script; `upgradeTo()` path — the demo's revocation trigger beat depends on it.
4. Fidelity test: compiled runtime bytecode's selector set + beacon slot layout match the step-2-calibrated fingerprint — the scanner's signature-match MUST hit the deployed replica.
5. Impostor twins ×2: same name/symbol, different mechanics (plain ERC-20; self-proxy variant); no record anywhere. Tests assert they fail the fingerprint's distinguishing checks the way the real impostor does.
6. Behavior tests: pause stops transfers; blocklisted address reverts; upgrade swaps impl; non-admin cannot upgrade/pause/blocklist.
7. Deploy: rehearsal to 46630/421614, then **4663** (gas verified in step 2); `forge verify` per knowledge/contract-verification.md — `--verifier blockscout` with the trailing `/api/` verifier-url (4663: `https://robinhoodchain.blockscout.com/api/`; 46630: `https://explorer.testnet.chain.robinhood.com/api/`); if the mainnet explorer's Cloudflare challenge blocks the server-side verify, fall back to Blockscout's browser-UI verification; addresses appended to `deployments/<chain>.json` + `demo/assets.md`.

Check: `forge test` green; replica + twins verified on the 4663 explorer; manifest lists every address; running the step-2 fingerprint procedure against live bytecode matches the replica and rejects both twins.

## Open questions
- RESOLVED 2026-09-11: `forge verify` against Robinhood's Blockscout is now covered by knowledge/contract-verification.md (recipe in task 7); residual unknown — whether forge verify passes the mainnet Cloudflare challenge from a dev machine is unverified, so make verification a same-day check after the 4663 deploy and use the browser-UI fallback if it blocks.
- Demo company name — coordinator may want to pick; replaceable in one constant before the 4663 deploy.
