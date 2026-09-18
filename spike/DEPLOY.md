# Spike deploy runbook — finishing the gate (one funding action + ~15 min)

The gate is decided by `spike/findings.md`. Everything except the gas receipts is
done and evidenced. This is the exact unblock: fund ONE address, then run the
numbered commands. An unfunded run cannot fabricate receipts — that is the whole
point of this file.

## 0. Fund (the only manual step)

Spike key (UNTRACKED — lives in `spike/.env`, never commit it):

    address: 0x151e9f57F31310aFeBBB60c222c14badCf938E4C

- Arbitrum Sepolia (421614) — the scratch/mirror env; PASS formula per verify.md
  loose end 5 accepts gas measured here (activation is already proven on 4663):
  try https://faucets.chain.link/arbitrum-sepolia or https://www.alchemy.com/faucets/arbitrum-sepolia
  (login-gated — fund from any holder wallet if faucets refuse).
- Robinhood Chain 4663 (real chain, small amounts): bridge from Ethereum via
  https://portal.arbitrum.io/bridge?destinationChain=robinhood-chain&sourceChain=ethereum
  (~10 min, L1 gas). Probe deploy quotes ~0.0001 ETH total (wasm data fee
  0.000092 + execution).
- 46630 optional; no faucet works headlessly (evidence/funding_blockers_20260919.txt).

## 1. Deploy the Stylus probe (two txs per chain, automatic)

    cd spike/probe
    source ../.env   # or: export PK=0x…
    cargo stylus deploy --endpoint https://sepolia-rollup.arbitrum.io/rpc      --private-key $SPIKE_DEPLOY_KEY
    cargo stylus deploy --endpoint https://rpc.mainnet.chain.robinhood.com     --private-key $SPIKE_DEPLOY_KEY

`check` already passed on both endpoints (evidence/stylus_check_4663_421614.log,
metadata hash 3ab96a63…0359). Note each chain's probe address.

## 2. Deploy the slot helper + canary + replica beat (Solidity, disposable)

    cd spike/replica && forge install   # restores stripped lib/forge-std
    DEPLOY_KEY=$SPIKE_DEPLOY_KEY forge script Deploy --rpc-url <RPC> --broadcast
    # then deploy Extsload (raw 0x5c helper) only if the canary says the opcode exists:
    DEPLOY_KEY=$SPIKE_DEPLOY_KEY forge script DeployCanary --rpc-url <RPC> --broadcast

(If `DeployCanary` does not exist yet, the 42-byte canary in
`src/ExtSloadProbe.sol::runtime()` + `src/RawDeploy.sol` is the recipe.)

## 3. Seed + run the measured suite (the receipt that decides ≤200,000)

    PROBE=0x…            # from step 1
    P=0x1Cdad396DB64BDa184d5182A97Dd9B3C62100b7D
    BEACON=0xe10b6f6b275de231345c20d14ab812db62151b00
    HELPER=0x…           # extsload helper from step 2 (suite degrades gracefully without it)

    # setup tx (NOT measured): seed the record with the live impl
    cast send $PROBE 'seedRecord(uint8,uint256,uint64,address,address,uint64,bytes32)' \
      1 0 1700000000 0xb35490d6f9163de4f80d88dc75c3516eb64c5ae2 \
      0x151e9f57F31310aFeBBB60c222c14badCf938E4C 0 0x0000000000000000000000000000000000000000000000000000000000000000 \
      --private-key $SPIKE_DEPLOY_KEY --rpc-url <RPC>

    # MEASURED: one tx = the full suite (record read + paused + blocklist + impl-vs-record)
    TX=$(cast send $PROBE 'runSuite(address,address,address,address)' $P $BEACON \
      0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045 $HELPER \
      --private-key $SPIKE_DEPLOY_KEY --rpc-url <RPC> --json | jq -r .transactionHash)
    cast receipt $TX --json | jq .gasUsed      # ← the spec number (spec.md:51): ≤ 200000?

    # individual probes (eth_call, free) — expected values:
    cast call $PROBE 'probePaused(address)' $P --rpc-url <RPC>          # (true, false) on 4663
    cast call $PROBE 'probeBlocklist(address,address)' $BEACON 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045 --rpc-url <RPC>   # (true, false) on 4663
    cast call $PROBE 'probeCode(address)' $P --rpc-url <RPC>            # (283, 0x6c1fdd40…)
    cast call $PROBE 'probeResolve(address,address)' $P $HELPER --rpc-url <RPC>  # (BEACON, 0xb35490d6…, true)

## 4. Flip the GATE line

Set `spike/findings.md` line 1 to the literal
`GATE: PASS — Stylus everywhere` (or FAIL if any step above contradicts the
expectations), paste the gas actuals into the Gas section, commit.
Then the merge-time edit applies: replace the PROBE_SELECTORS placeholders in
`packages/shared/abi.ts` with `{ paused: "0x5c975abb", blocklist: "0xfbac3951" }`.
