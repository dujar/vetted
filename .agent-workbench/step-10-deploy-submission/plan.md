# Step 10 — Production deploy, final smoke, submission (buffered before 2026-10-04 23:59 SGT)

## Resources
- spec:    ../product/spec.md   (pre-submission checklist spec.md:71; constraints spec.md:69; risks spec.md:73–80)
- knows:   ../knowledge/robinhood-chain.md, ../knowledge/arbitrum-sepolia.md, ../knowledge/goplus-api.md, ../knowledge/eas.md, ../knowledge/contract-verification.md  (rehearsed Q&A ammo; verification re-check recipes :12–38)
- learned: ../step-7-contract-hardening/findings.md, ../step-9-demo-harness/findings.md

## Stack
As built; no new dependencies.

## Goal
After this step the project is submitted with ≥24h of buffer: everything deployed and source-verified at production URLs, the demo arc smoke-green on those URLs, the spec's pre-submission checklist done, and the submission package uploaded.

## Scope
- Touches: deploy orchestration scripts, `submission/**`, release tag. Product code frozen except blocking fixes.

## User journey
N/A — operational close-out.

## Screens
N/A.

## Tasks
1. Freeze + tag `v1.0.0`; CI green at the tag; clean-checkout build verified.
2. Production state check: 4663 core + replicas deployed and source-verified (step 7 / step 6 artifacts; re-check verify status per knowledge/contract-verification.md — 4663 = Blockscout with the CF-challenge browser-UI fallback :19–27, 421614 = Etherscan v2 with `chainid=421614`, one key, v1 endpoints dead :27–38), worker + Pages at production URLs, secrets/env audit (REGISTRAR_KEY — its address must equal the deployed registry's registrar, pinned by step 7's deploy task; WalletConnect projectId; chain RPCs), 421614 mirror consistent.
3. Full demo-arc smoke against production URLs — step 9's runbook, one pass, timed.
4. Pre-submission checklist (spec.md:71): browser spot-check Token Sniffer + De.Fi on a genuine 4663 token (their capabilities were unverified at research time — adjust Q&A if either catches the beacon pattern); rehearse the EAS answer with the knowledge/eas.md addresses and the GoPlus answer with the spike screenshot (enforcement / revocation / power-disclosure copy branch).
5. Submission package: final video, README, repo link, live URLs, contract addresses — uploaded targeting **2026-10-03 EOD SGT**, ≥24h ahead of the close (spec.md:69).
6. Post-submit: archive findings, note residual risks for judges' Q&A (fresh-redeployment-reads-IMPOSTOR lag = spec.md:79, twins-on-production mitigation = spec.md:80).

Check: submission confirmation recorded; all public URLs resolve (Pages, Worker, 4663 explorer, mirror explorer); the tagged release builds from clean checkout.

## Open questions
- Submission-portal mechanics (form fields, video hosting) — operator handles from the checklist; no plan dependency.
