# deployments/

Append-point directory — one JSON file per deploy target, written only by
deploy steps. Never rewrite another step's entry: read + append fields, or
add a new file.

| file | target | writer |
|---|---|---|
| `worker.json` | Cloudflare Worker — scan backend (hello in step 1, engine in step 4) | 1, 4 |
| `pages.json` | Cloudflare Pages — frontend static shell (hello in step 1, screens in step 5) | 1, 5 |
| `421614.json` | Arbitrum Sepolia scratch contract deploys | 3 |
| `46630.json` | Robinhood testnet scratch contract deploys | 3 |
| `4663.json` | Robinhood Chain mainnet core deploy (registrar handoff) | 7 |
