# Fingerprint — "structurally matches the genuine Robinhood stock-token pattern"

Single named artifact for the composed check list (step-6 verify loose end 1,
resolving the step-4 ⇄ step-6 joint-composition deferral, plan Revised note 3).
Step 4's matcher composes from THIS list; step 6's `test/Fingerprint.t.sol`
implements it against the deployed replica (must PASS every check) and both
impostor twins (must FAIL on the named checks). Ground truth:
`spike/evidence/calibration_4663.json`, `spike/findings.md`,
`packages/shared/abi.ts` PROBE_SELECTORS.

## Checks — ALL must pass

- **F1 solc 0.8.x** — runtime bytecode's CBOR metadata tail carries version
  major `0`, minor `8`. (Genuine tokens: solc 0.8.33; replica pins 0.8.28 —
  the check is the 0.8.x family, not an exact compiler.)
- **F2 EIP-1967 layout** — on the token proxy: beacon slot
  (`0xa3f0…3d50`) is SET, implementation slot (`0x3608…2bbc`) is EMPTY.
  Beacon-proxy layout, byte-verified live on P and CRM (calibration
  `slot_reads`). This is the check both twins fail: ImpostorPlain has no
  proxy at all; ImpostorSelfProxy sets the IMPL slot and leaves the beacon
  slot empty.
- **F3 beacon resolvable from forwarder bytecode SHAPE** — the proxy's
  runtime bytecode embeds the beacon address as an immediate (PUSH32
  zero-padded, same shape as the genuine 283-byte forwarder,
  `p_proxy.hex`), and STATICCALLing `implementation()` (0x5c60da1b) on the
  embedded address returns an address with code. Resolution is
  shape/offset-based — the guard (step 3) extracts it exactly this way.
  NEVER codehash equality: the embedded beacon address differs per deploy,
  so no replica's forwarder can byte-match the genuine one (0x6c1fdd40…).
- **F4 calibrated probe targets** (`packages/shared/abi.ts`):
  `paused()` 0x5c975abb answers on the token PROXY;
  `isBlocked(address)` 0xfbac3951 answers on the RESOLVED BEACON (and
  reverts on the proxy/impl — the hidden-blocklist semantics, live-verified
  in calibration `live_calls_on_P`).
- **F5 required selector subset** — token (via proxy): name 0x06fdde03,
  symbol 0x95d89b41, decimals 0x313ce567, totalSupply 0x18160ddd,
  balanceOf 0x70a08231, transfer 0xa9059cbb, transferFrom 0x23b872dd,
  approve 0x095ea7b3, allowance 0xdd62ed3e, paused 0x5c975abb.
  Beacon: implementation 0x5c60da1b, upgradeTo 0x3659cfe6, paused
  0x5c975abb, pause 0x8456cb59, unpause 0x3f4ba83a, isBlocked 0xfbac3951,
  hasRole 0x91d14854, grantRole 0x2f2ff15d, DEFAULT_ADMIN_ROLE 0xa217fddf,
  supportsInterface 0x01ffc9a7. (Subset of calibration
  `fingerprint_selectors`; role surface = the observed AccessControl
  beacon dispatcher.)

## Deliberately excluded (would reject EVERY legitimate replica)

- **Proxy/impl/beacon codehash byte-equality** — the genuine forwarder
  embeds beacon 0xe10b…1b00 and the genuine impl carries its own metadata;
  no independently deployed replica can byte-match. Step 4 must NOT require
  it (agreed shape-based resolution instead — coordinator decision 1).
- **`uid()` == /rhj/assets registry id** — the demo company "Aurelia
  Industries" is not among the 194 registry assets, so a replica can never
  satisfy it. The replica does not implement `uid()` at all; canonical
  linkage is step 9/registry work, not a pattern check.
- **permit / nonces / uiMultiplier** — present on the genuine impl
  dispatcher, absent from the demo beats; optional, not required.

## Twin failure modes (deterministic)

| twin | passes | fails |
|---|---|---|
| ImpostorPlain | F5 name/symbol/decimals (mimicry) | F2 (no proxy slots), F3 (no embedded beacon), F4-blocklist (no resolved beacon to answer) |
| ImpostorSelfProxy | F5 name/symbol, `paused()` answers on its proxy (looks upgradeable) | F2 (impl slot SET, beacon slot EMPTY), F3 (embedded address is an impl, does not answer `implementation()`), F4-blocklist |

Name/symbol mimicry passing while structure fails is the point — it is the
real impostor's shape (knowledge robinhood-stock-tokens.md: "a token with a
matching name/ticker but a different contract address is not a Robinhood
Stock Token").
