/**
 * Scratch-deployment state shared by the config (bundle env) and the gated
 * live specs (test.skip decisions). null = no funded scratch deployment —
 * the honest default today (operator key 0 wei; step-3 findings).
 */
import path from "node:path";
import { fileURLToPath } from "node:url";

import { readScratchDeployment, type ScratchDeployment } from "./deployments";

const here = path.dirname(fileURLToPath(import.meta.url));
export const REPO_ROOT = path.resolve(here, "..", "..");

export const SCRATCH: ScratchDeployment | null = readScratchDeployment(REPO_ROOT);

/** The chain the live server's bundle targets (VITE_CHAIN_ID). */
export const SERVER_CHAIN_ID = SCRATCH?.chainId ?? 4663;
