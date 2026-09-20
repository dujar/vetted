# deployments/

Append-point directory — one JSON file per deploy target, written only by
deploy steps. Never rewrite another step's entry: read + append fields, or
add a new file.

| file | target | writer |
|---|---|---|
| `worker.json` | Cloudflare Worker — scan backend (hello in step 1, engine in step 4) | 1, 4 |
| `pages.json` | Cloudflare Pages — frontend static shell (hello in step 1, screens in step 5, step-8 redeploy) | 1, 5, 8 (step-8 pages redeploy pin) |
| `421614.json` | Arbitrum Sepolia scratch contract deploys | 3, 6 (replicas field) |
| `46630.json` | Robinhood testnet scratch contract deploys | 3, 6 (replicas field) |
| `4663.json` | Robinhood Chain mainnet deploys — step 6 creates it with the `replicas` field (demo cast), step 7 appends the core deploy (registrar handoff) | 6 (replicas field), 7 |
