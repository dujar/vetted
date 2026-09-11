---
topic: eas
version: eas-contracts deployments @ master, checked 2026-09-11
checked: 2026-09-11
sources:
  - https://github.com/ethereum-attestation-service/eas-contracts/tree/master/deployments
  - https://raw.githubusercontent.com/ethereum-attestation-service/eas-contracts/master/deployments/arbitrum-sepolia/EAS.json
---

- Arbitrum Sepolia (421614): EAS IS deployed — EAS proxy
  0x2521021fc8BF070473E1e1801D3c7B4aB701E1dE, SchemaRegistry
  0x45CB6Fa0870a8Af06796Ac15915619a0f22cd475 (canonical eas-contracts repo
  deployments/arbitrum-sepolia/, fetched 2026-09-11). The spec's "EAS availability …
  unverified (docs hosts broken)" is now resolved for Sepolia: available, addresses above.
- Robinhood Chain (4663): EAS is NOT deployed — the repo's deployments/ directory lists 26
  chain folders and none is 4663 (checked 2026-09-11; other Arbitrum chains present:
  arbitrum-one, arbitrum-nova, arbitrum-sepolia). Only path on 4663 would be
  self-deploying the EAS contracts — irrelevant for v1 given the spec's revocation/probe
  semantics make the bespoke Canonical Registry the product either way.
- Doc-host note for the Q&A answer: docs.easscan.org did not resolve (DNS ENOTFOUND) from
  this environment on 2026-09-11 — the "broken docs host" observation was reproducible;
  the canonical source that always worked is the GitHub deployments directory above.
- Spike bullet "EAS availability: direct probe on both chains" is answered; no on-chain
  probe needed unless you want to double-check nothing was deployed without a repo PR.
- Unchanged: EAS stores attestations only — no interpretation/enforcement/revocation;
  the spec's "why not EAS" argument stands as written.
