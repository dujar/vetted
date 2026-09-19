# demo/assets.md — the demo cast (step 6)

The 5-minute demo's live cast on **Robinhood Chain 4663** (spec.md:38 Scope 5):
one genuine-pattern stock-token replica — beacon proxy, hidden per-address
blocklist ON THE BEACON, global pause — plus two impostor twins sharing its
name/symbol. Everything is a **demo artifact**: the twins carry no liquidity
and no holder base, the replica's company ("Aurelia Industries") is a
placeholder, and this file is the honest label for both (spec.md:80 risk 6).
The tool's own IMPOSTOR rule flags the twins — the decoys are live evidence.

- Pattern check list: `contracts/replicas/FINGERPRINT.md` (composed with
  step 4; the scanner must accept the replica and reject both twins).
- Contracts/tests/deploy: `contracts/replicas/` (own foundry project),
  `scripts/deploy/replicas/deploy.sh`.
- Probe targets (calibrated, `packages/shared/abi.ts`): `paused()`
  0x5c975abb → the token PROXY; `isBlocked(address)` 0xfbac3951 → the
  resolved BEACON (reverts through the proxy — the hidden-blocklist signature).

## The cast

Shared name/symbol across all three tokens — `Aurelia Industries • Robinhood
Token` / `AURE`, 18 decimals (genuine naming shape, knowledge
robinhood-stock-tokens.md:23-31). Matching name/ticker at a different
contract address is the docs' own impostor definition; that mimicry passing
while structure fails is the demo's compare beat.

| role | contract | 4663 (canonical) | 46630 (rehearsal) |
|---|---|---|---|
| replica impl v1 | `DemoStockToken` | PENDING DEPLOY | PENDING DEPLOY |
| replica impl v2 — **upgrade target** | `DemoStockToken` (same code, new address) | PENDING DEPLOY | PENDING DEPLOY |
| replica beacon (pause + blocklist state lives here) | `DemoBeacon` | PENDING DEPLOY | PENDING DEPLOY |
| replica proxy — **the canonical demo token** | `DemoBeaconProxy` | PENDING DEPLOY | PENDING DEPLOY |
| twin 1 — impostor "naive copy" | `ImpostorPlain` | PENDING DEPLOY | PENDING DEPLOY |
| twin 2 — impostor "self-proxy" (proxy + impl) | `ImpostorSelfProxy` / `ImpostorSelfProxyImpl` | PENDING DEPLOY | PENDING DEPLOY |
| admin / minter (demo operator) | EOA | deployer key | spike throwaway key |

Addresses are appended here + `deployments/<chain>.json` immediately after
each broadcast (46630/421614 rehearsal: step 6 appends a `replicas` field;
4663: step 6 creates `4663.json` with the `replicas` field, step 7 appends
its core deploy).

**Why two impls:** the registry records the implementation as an ADDRESS. The
demo's revocation beat upgrades `upgradeTo(implV2)` under the FIXED proxy —
same code, different address — which is exactly the drift the registry's
watchdog keys on (step-6 verify loose end 2; step 9 scripts the tx).

## Twins — what a scanner must see

- **ImpostorPlain** — pause + blocklist held IN the token (the naive copy of
  the leaked pattern; also the spike replica's documented anti-pattern) plus
  a fabricated registry-style `uid()`. Fails F2/F3/F4 (no EIP-1967 beacon
  layout, no implementation()-answering forwarder, no beacon to answer the
  blocklist probe).
- **ImpostorSelfProxy** — an EIP-1967 IMPL-slot proxy (the wrong slot):
  `paused()` answers on its proxy like the genuine shape, but the beacon slot
  is empty and the impl slot is set. Fails F2/F3/F4. It exists because the
  fingerprint must require beacon-slot-set/impl-slot-empty, not just
  "some proxy".

## Beat choreography (step 9 drives these; admin = demo operator key)

```bash
# 0. scan (read-only, no wallet): replica → VERIFIED*; twins → IMPOSTOR
#    (*registry record is step 9's seeding; until then the replica scans as
#    pattern-match + UNVERIFIED-record per the engine's degraded branch)

# 1. hidden blocklist beat — state set on the BEACON, invisible on the token
cast send $BEACON "setBlocked(address,bool)" $BUYER true --private-key $ADMIN --rpc-url $RPC
cast call $BEACON "isBlocked(address)" $BUYER --rpc-url $RPC          # true
cast call $PROXY "isBlocked(address)" $BUYER --rpc-url $RPC           # REVERTS (hidden)

# 2. guarded swap refuses the blocklisted buyer (step-3 guard, 4663)

# 3. global pause — beacon state, proxy-visible probe
cast send $BEACON "pause()" --private-key $ADMIN --rpc-url $RPC
cast call $PROXY "paused()" --rpc-url $RPC                            # true

# 4. THE BEAT: upgrade under a fixed proxy → registry auto-revokes →
#    guard refuses the formerly verified token at execution
cast send $BEACON "upgradeTo(address)" $IMPL_V2 --private-key $ADMIN --rpc-url $RPC
cast call $PROXY "paused()" --rpc-url $RPC      # still true — beacon state survived
```

## Deploy + verify

```bash
# rehearsal (46630 — chains table in packages/shared/wire.md):
DEPLOY_KEY=0x… RPC_URL=https://rpc.testnet.chain.robinhood.com \
  ./scripts/deploy/replicas/deploy.sh
# mainnet (4663) — same wrapper, RPC https://rpc.mainnet.chain.robinhood.com

# verify (knowledge/contract-verification.md — Blockscout, trailing /api/):
forge verify-contract <addr> src/DemoStockToken.sol:DemoStockToken \
  --chain-id 4663 --rpc-url $RH_RPC_URL --verifier blockscout \
  --verifier-url https://robinhoodchain.blockscout.com/api/
# 46630: --verifier-url https://explorer.testnet.chain.robinhood.com/api/
# 421614: --chain-id 421614 (Etherscan v2 + ETHERSCAN_API_KEY)
# mainnet explorer may sit behind a Cloudflare challenge for server-side
# clients → fall back to Blockscout's browser-UI verification.
```

Funding reality (2026-09-19): the throwaway key `0x151e…8E4C` reads 0 wei on
46630, 4663 and 421614 (faucets headless-blocked, `spike/evidence/
funding_blockers_20260919.txt`); operator funding was still pending at step
close, so the 4663 addresses above are PENDING — the deploy itself is
rehearsed green on a local chain and the code path is identical per chain.
Step 9's dry runs re-exercise these assets on the funded key.
