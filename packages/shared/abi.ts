/**
 * Registry + guarded-swap ABI — pinned here before any contract exists so
 * steps 3/5/8 build against the same surface. Step 3's plan text is the source
 * for the signatures; they must match the deployed contracts byte-for-byte
 * (verify.md loose end 1). See wire.md for the prose contract.
 */
import { parseAbi } from "viem";

/** Canonical Registry (step 3, contracts/core/registry). */
export const REGISTRY_ABI_SIGNATURES = [
  "function getRecord(address token) view returns ((uint8 status, uint256 riskFlags, uint64 verifiedAt, address impl, address registrar, uint64 revokedAt, bytes32 reason))",
  "function verify(address token, uint256 riskFlags, address impl)",
  "function revoke(address token, string reason)",
] as const;

/** Guarded swap (step 3, contracts/core/guard). */
export const GUARD_ABI_SIGNATURES = [
  "function commit(address tokenIn, address tokenOut, uint256 amountIn, uint256 minOut)",
  "function execute()",
] as const;

/** Parsed, ready for viem public clients (steps 5/8 read through this). */
export const REGISTRY_ABI = parseAbi([...REGISTRY_ABI_SIGNATURES]);
export const GUARD_ABI = parseAbi([...GUARD_ABI_SIGNATURES]);

/**
 * 4-byte selector constants — the on-chain read surface as opaque bytes, for
 * consumers that build raw calls (step 8's e2e, step 4's probe client).
 * Verified against the signatures by tests/roundtrip.test.ts.
 */
export const SELECTORS = {
  getRecord: "0x617fba04",
  verify: "0x73c7cf61",
  revoke: "0xafd0224b",
  commit: "0x498ab631",
  execute: "0x61461954",
} as const;

/** Guard revert reasons — verbatim ABI revert strings (journeys.md:32). */
export const GUARD_REVERT_REASONS = [
  "GUARD_NO_RECORD",
  "GUARD_RECORD_REVOKED",
  "GUARD_PAUSED",
  "GUARD_BLOCKLISTED",
  "GUARD_IMPL_MISMATCH",
] as const;

// ============================================================================
// PROBE_SELECTORS — RESERVED BLOCK (step 2 replaces at merge)
// ----------------------------------------------------------------------------
// Placeholder bytes until the week-1 spike calibrates the real probe selectors
// (paused() + the per-address buyer-blocklist probe) on live 4663 tokens
// (step-2 plan task 5, merge-time append). Consumers read the exported
// constant — never hardcode the placeholder value — so step 2's replacement
// propagates everywhere at once.
// ============================================================================
export const PROBE_SELECTORS = {
  paused: "0x00000000",
  blocklist: "0x00000000",
} as const;
