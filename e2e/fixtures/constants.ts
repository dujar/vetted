/**
 * e2e constants — every product-shaped literal comes from `vetted-shared` or
 * the golden fixtures (plan Revised 2026-09-11 note 1: never retranscribe
 * strings or selectors). Only the mock SENTINEL addresses are mirrored here:
 * they are display fixtures owned by `frontend/src/lib/mockData.ts`, whose
 * module imports Vite-style JSON + cannot load in the Playwright process
 * without a bundler — the mirrors are kept in one place and asserted against
 * the shared fixtures where they derive from them.
 */
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

import { GUARD_REVERT_REASONS, SELECTORS } from "vetted-shared";

/** ScanResponse fixture shape — only the fields the specs assert on. */
interface ScanFixture {
  addr: string;
  verdict: string | null;
  degraded: boolean;
  notice: string | null;
  powerReport: { check: string; result: string; severity: string; evidenceUrl: string }[];
  record: { status: string; impl: string; registrar: string; reason: string } | null;
  revocationTx: string | null;
}

/** Golden fixtures, read verbatim — the scan specs assert against these. (Node ESM wants import
 * attributes for JSON modules; readFileSync is the boring, loader-proof route.) */
const fixtureJson = (name: string): ScanFixture =>
  JSON.parse(readFileSync(new URL(`../../packages/shared/fixtures/verdict/${name}.json`, import.meta.url), "utf8"));

export const FIXTURES = {
  VERIFIED: fixtureJson("VERIFIED"),
  IMPOSTOR: fixtureJson("IMPOSTOR"),
  UNVERIFIED: fixtureJson("UNVERIFIED"),
  REVOKED: fixtureJson("REVOKED"),
} as const;

export { GUARD_REVERT_REASONS, SELECTORS };

/** The VERIFIED fixture's own address — the mock canonical (NVIDIA). */
export const MOCK_VERIFIED_ADDR = FIXTURES.VERIFIED.addr;
/** The IMPOSTOR fixture's address — the mock twin (also mock GUARD_NO_RECORD sentinel). */
export const MOCK_IMPOSTOR_ADDR = FIXTURES.IMPOSTOR.addr;
/** The UNVERIFIED fixture's address — degraded:true payload (issuer list down). */
export const MOCK_DEGRADED_ADDR = FIXTURES.UNVERIFIED.addr;

/** `pad("d5da")` → `0x00000000…d5da` — mirrors frontend/src/lib/mockData.ts's sentinels. */
const pad = (hex: string): string => `0x${hex.padStart(40, "0")}`;
const padEnds = (first4: string, last4: string): string => `0x${first4}${"0".repeat(32)}${last4}`;

/** Mock-only sentinels (frontend/src/lib/mockData.ts) — one address per journey state. */
export const MOCK_REVOKED_ADDR = padEnds("9f8c", "d2e5");
export const MOCK_NOT_CONTRACT_ADDR = pad("dead");
export const MOCK_RPC_ERROR_ADDR = pad("bad1");
export const MOCK_PAUSED_ADDR = pad("fa5e");
export const MOCK_BLOCKLISTED_ADDR = pad("b10c");
export const MOCK_IMPL_MISMATCH_ADDR = pad("d1f5");
export const MOCK_USDG_ADDR = pad("d5da");

/** The guard's deterministic red flags, one mock sentinel token each (journeys.md:32). */
export const GUARD_SENTINELS: Record<(typeof GUARD_REVERT_REASONS)[number], string> = {
  GUARD_NO_RECORD: MOCK_IMPOSTOR_ADDR,
  GUARD_RECORD_REVOKED: MOCK_REVOKED_ADDR,
  GUARD_PAUSED: MOCK_PAUSED_ADDR,
  GUARD_BLOCKLISTED: MOCK_BLOCKLISTED_ADDR,
  GUARD_IMPL_MISMATCH: MOCK_IMPL_MISMATCH_ADDR,
};

/** Live scan-backend worker (step 4) — override with VETTED_E2E_WORKER_URL. */
export const WORKER_URL =
  process.env.VETTED_E2E_WORKER_URL ?? "https://vetted-scan-backend.dujar-coding.workers.dev";

/** Local server ports (env-overridable — parallel builders share this machine). */
export const MOCK_PORT = Number(process.env.VETTED_E2E_MOCK_PORT ?? 4883);
export const LIVE_PORT = Number(process.env.VETTED_E2E_LIVE_PORT ?? 4884);

/** Public-RPC hosts the frontend's viem transports target (packages/shared/wire.md chains table). */
export const RPC_HOST_SUFFIXES = ["rpc.mainnet.chain.robinhood.com", "rpc.testnet.chain.robinhood.com", "sepolia-rollup.arbitrum.io"];

/** Honest-degrade retry policy (step-4 guidance / plan Revised 2026-09-20 note 2): RPC_RETRYABLE → backoff retry. */
export const RETRY_BACKOFF_MS = 500;
export const RETRY_MAX_ATTEMPTS = 3;

/** Scratch-chain RPCs, for the gated live specs' Node-side reads/sends (viem). */
export const SCRATCH_RPCS: Record<number, string> = {
  46630: "https://rpc.testnet.chain.robinhood.com",
  421614: "https://sepolia-rollup.arbitrum.io/rpc",
};

/**
 * e2e-only buyer key. Default is anvil's well-known key #0 — a PUBLIC throwaway
 * that only ever holds scratch-chain test funds (the seed script funds it);
 * override with VETTED_E2E_BUYER_KEY. Never a production key.
 */
export const BUYER_ADDRESS = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";
export const BUYER_KEY = process.env.VETTED_E2E_BUYER_KEY ?? "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

/** Sentinel registry/guard addresses for the rpc-intercepted live specs (no deployments needed). */
export const SENTINEL_REGISTRY = pad("5e15aa");
export const SENTINEL_GUARD = pad("beef17");

/** hex quantity for a chain id — stub wallet + rpc stub share it. */
export const chainIdHex = (chainId: number): string => `0x${chainId.toString(16)}`;
