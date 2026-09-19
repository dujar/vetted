#!/usr/bin/env bash
# Step-6 rehearsal/production deploy wrapper (runbook: demo/assets.md).
# Deploys the full demo cast — replica (v1 + upgrade target + beacon + proxy)
# and both impostor twins — to one chain.
#
#   DEPLOY_KEY=0x… RPC_URL=https://… ./scripts/deploy/replicas/deploy.sh
#
# DEPLOY_KEY: the throwaway spike key (spike/.env, never committed) for
# rehearsals; ADMIN env var optionally overrides the admin (defaults to the
# deployer). Deps first: cd contracts/replicas and run the three clone
# commands in contracts/replicas/README.md.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
: "${DEPLOY_KEY:?set DEPLOY_KEY}"
: "${RPC_URL:?set RPC_URL}"
cd "$ROOT/contracts/replicas"
forge script script/DeployReplicas.s.sol:DeployReplicas --rpc-url "$RPC_URL" --broadcast --slow
echo "--- append the logged addresses to deployments/ + demo/assets.md ---"
