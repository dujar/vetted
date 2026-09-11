---
topic: goplus-api
version: api.gopluslabs.io v1, probed live 2026-09-11
checked: 2026-09-11
sources:
  - https://docs.gopluslabs.io/reference/tokensecurityusingget_1.md
  - https://docs.gopluslabs.io/reference/response-details.md
  - live calls: supported_chains + token_security/4663, 2026-09-11
---

- ENDPOINT ROT: `/api/v1/token-security/{chain_id}` (hyphens — the form in the spec's
  memory) is DEAD. It returns 404 for EVERY chain including Ethereum mainnet (verified
  2026-09-11). Current shape, verbatim from the OpenAPI docs:
  `GET https://api.gopluslabs.io/api/v1/token_security/{chain_id}?contract_addresses=0x…`
  (underscores; comma-separated addresses).
- Supported chains: `/api/v1/supported_chains` (underscore; hyphen variant also 404s).
  Returns 45 chains including `{"name":"Robinhood","id":"4663"}` (live 2026-09-11).
- WEEK-1 SPIKE RESULT, pre-answered live 2026-09-11 (P / "Everpure • Robinhood Token",
  0x1Cdad396DB64BDa184d5182A97Dd9B3C62100b7D): token_security/4663 returns code 1 OK but
  only 16 fields — token_name/symbol, is_open_source=1, is_proxy=1, creator_address,
  holder_count=8, etc. EVERY risk field is absent: no is_honeypot, transfer_pausable,
  is_blacklisted, is_whitelisted, can_take_back_ownership, owner_change_balance,
  hidden_owner, slippage_modifiable, trading_cooldown.
- Why: docs state the risk fields "will not be returned … sometimes when `is_proxy`: 1".
  GoPlus structurally cannot flag the beacon-proxy stock-token pattern — coverage arrived,
  correctness on this chain's pattern is nil. Demo copy: use the enforcement/revocation/
  power-disclosure branch; "first *correct* coverage", never "first coverage".
- Field names when they DO return (response-details): `is_honeypot`, `transfer_pausable`,
  `is_blacklisted`, `is_whitelisted`, `can_take_back_ownership`, `owner_change_balance`,
  `hidden_owner`, `selfdestruct`, `external_call`, `slippage_modifiable`,
  `personal_slippage_modifiable`, `trading_cooldown`, `owner_address`, `buy_tax`/`sell_tax`.
  Values "1"/"0"; absence = unknown, not negative.
- Rate limits: docs now describe a CU (compute-unit) account model — `cu_per_minute`,
  `cu_per_day`, overage pricing — tied to a console key sent as
  `Authorization: Bearer <token>`. Unauthenticated calls still work (all probes this
  session used no key). Exact unauthenticated per-IP limit: UNVERIFIED — current docs no
  longer state the old number; don't cite one in Q&A.
- All docs pages serve markdown via a `.md` suffix (docs.gopluslabs.io/llms.txt indexes
  them) — scrape that, not the JS site.
