/**
 * Guarded-swap client-side preview + execution seam (plan task 3, J2). The
 * guard panel previews the exact checks the contract re-runs at execution
 * (journeys.md:31-32): record / paused / blocklist / impl-match, in guard
 * order, first deterministic red flag = the verbatim revert reason
 * (GUARD_REVERT_REASONS from vetted-shared — never retranscribed).
 *
 * The probe transport is an interface so step 2's calibrated
 * PROBE_SELECTORS bytes flow through the live source without edits here, and
 * mock mode (fixture records + sentinel probes) stays independent of the
 * calibration merge.
 */
import type { PublicClient } from "viem";
import {
  ContractFunctionRevertedError,
  EstimateGasExecutionError,
  UserRejectedRequestError,
  createPublicClient,
  http,
  parseAbi,
} from "viem";
import {
  GUARD_REVERT_REASONS,
  PROBE_SELECTORS,
  REGISTRY_ABI,
  type GuardRevertReason,
  type RegistryRecord,
} from "vetted-shared";

import { VETTED_CHAIN } from "./chains";
import { isLiveApi } from "./api";
import { formatDate, shortAddr } from "./format";
import { decodeRegistryRecord } from "./registry";
import {
  GUARD_FIXTURES,
  MOCK_BLOCKLISTED_ADDR,
  MOCK_IMPL_MISMATCH_ADDR,
  MOCK_PAUSED_ADDR,
  MOCK_REVOKED_ADDR,
  MOCK_REVOKED_RECORD,
  MOCK_TX,
  MOCK_VERIFIED_ADDR,
  SCAN_FIXTURES,
} from "./mockData";

export type { GuardRevertReason } from "vetted-shared";
export const GUARD_REASONS = GUARD_REVERT_REASONS;

/** One guard-panel row — `.check-row` per swap.html. */
export interface GuardCheckRow {
  label: string;
  detail: string;
  ok: boolean;
  /** Set when this row is a deterministic red flag — the reason the guard will revert, verbatim. */
  revertReason: GuardRevertReason | null;
}

export interface GuardPreview {
  rows: GuardCheckRow[];
  firstRevert: GuardRevertReason | null;
}

/** Probe transport seam — live (viem) and mock (fixtures + sentinels) impls. */
export interface GuardProbeSource {
  /** Wire record or null = no record (never guessed — wire.md). */
  getRecord(token: string): Promise<RegistryRecord | null>;
  /** null = probe unavailable (advisory "unknown"; heuristics never revert — spec.md:28). */
  paused(token: string): Promise<boolean | null>;
  isBlocked(token: string, buyer: string): Promise<boolean | null>;
  /** EIP-1967 impl slot, else beacon; null when neither. */
  resolvedImpl(token: string): Promise<string | null>;
}

export interface GuardPreviewParams {
  token: string;
  buyer: string;
}

const implLabel = (addr: string | null): string => (addr ? shortAddr(addr) : "—");

export async function previewGuard(
  source: GuardProbeSource,
  { token, buyer }: GuardPreviewParams,
): Promise<GuardPreview> {
  const record = await source.getRecord(token);

  let recordRow: GuardCheckRow;
  if (!record) {
    recordRow = {
      label: "Live verification record in Canonical Registry",
      detail: "no record",
      ok: false,
      revertReason: "GUARD_NO_RECORD",
    };
  } else if (record.status === "REVOKED") {
    recordRow = {
      label: "Live verification record in Canonical Registry",
      detail: `revoked ${formatDate(record.revokedAt)}`,
      ok: false,
      revertReason: "GUARD_RECORD_REVOKED",
    };
  } else {
    recordRow = {
      label: "Live verification record in Canonical Registry",
      detail: `verified ${formatDate(record.verifiedAt)}`,
      ok: true,
      revertReason: null,
    };
  }

  const paused = await source.paused(token);
  const pausedRow: GuardCheckRow = {
    label: "Token not paused",
    detail: paused === null ? "probe unavailable — the guard still enforces on-chain" : `paused() → ${paused}`,
    ok: paused !== true,
    revertReason: paused === true ? "GUARD_PAUSED" : null,
  };

  const blocked = await source.isBlocked(token, buyer);
  const blockRow: GuardCheckRow = {
    label: "Buyer not blocklisted",
    detail:
      blocked === null ? "probe unavailable — the guard still enforces on-chain" : `isBlocked(buyer) → ${blocked}`,
    ok: blocked !== true,
    revertReason: blocked === true ? "GUARD_BLOCKLISTED" : null,
  };

  const impl = record ? await source.resolvedImpl(token) : null;
  const implMatches = !!record && impl !== null && impl.toLowerCase() === record.impl.toLowerCase();
  const implRow: GuardCheckRow = {
    label: "Implementation matches registry record",
    detail:
      !record
        ? "—"
        : impl === null
          ? "probe unavailable — the guard still enforces on-chain"
          : implMatches
            ? `${implLabel(impl)} = ${implLabel(record.impl)}`
            : `${implLabel(impl)} ≠ ${implLabel(record.impl)}`,
    ok: !record || implMatches,
    revertReason: !!record && impl !== null && !implMatches ? "GUARD_IMPL_MISMATCH" : null,
  };

  const rows = [recordRow, pausedRow, blockRow, implRow];
  const firstRevert = rows.find((r) => r.revertReason !== null)?.revertReason ?? null;
  return { rows, firstRevert };
}

// ============================================================================
// Live source — viem reads against the vetted chain (VETTED_CHAIN — 4663 in
// the product; VITE_CHAIN_ID re-points the bundle at a scratch chain for the
// e2e live pass). Registry ABI from vetted-shared, probe selectors from the
// exported PROBE_SELECTORS constant; probe failures degrade to "probe
// unavailable", never to a guessed pass.
// ============================================================================

const EIP1967_IMPL_SLOT =
  "0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc" as const;
const EIP1967_BEACON_SLOT =
  "0xa3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50" as const;
const BEACON_ABI = parseAbi(["function implementation() view returns (address)"]);

export class ViemGuardProbeSource implements GuardProbeSource {
  constructor(
    private readonly client: PublicClient,
    private readonly registryAddress: `0x${string}`,
  ) {}

  async getRecord(token: string): Promise<RegistryRecord | null> {
    try {
      const raw = await this.client.readContract({
        address: this.registryAddress,
        abi: REGISTRY_ABI,
        functionName: "getRecord",
        args: [token as `0x${string}`],
      });
      return decodeRegistryRecord(raw);
    } catch {
      return null;
    }
  }

  async paused(token: string): Promise<boolean | null> {
    return this.probeBool(token, PROBE_SELECTORS.paused, "0".repeat(64));
  }

  async isBlocked(token: string, buyer: string): Promise<boolean | null> {
    return this.probeBool(token, PROBE_SELECTORS.blocklist, buyer.slice(2).toLowerCase().padStart(64, "0"));
  }

  private async probeBool(token: string, selector: string, argsWord: string): Promise<boolean | null> {
    try {
      const res = await this.client.call({
        to: token as `0x${string}`,
        data: `${selector}${argsWord}` as `0x${string}`,
      });
      if (!res.data) return false;
      return BigInt(res.data) !== 0n;
    } catch {
      return null; // probe unavailable — advisory, never a guessed pass (spec.md:28)
    }
  }

  async resolvedImpl(token: string): Promise<string | null> {
    try {
      const impl = await this.client.getStorageAt({ address: token as `0x${string}`, slot: EIP1967_IMPL_SLOT });
      if (impl !== undefined && BigInt(impl) !== 0n) return impl.slice(0, 42);
      const beacon = await this.client.getStorageAt({ address: token as `0x${string}`, slot: EIP1967_BEACON_SLOT });
      if (beacon === undefined || BigInt(beacon) === 0n) return null;
      const beaconAddr = `0x${beacon.slice(-40)}` as `0x${string}`;
      return await this.client.readContract({
        address: beaconAddr,
        abi: BEACON_ABI,
        functionName: "implementation",
      });
    } catch {
      return null;
    }
  }
}

// ============================================================================
// Mock source — fixture records + sentinel probes (verify.md loose ends 4/5).
// ============================================================================

const ZERO_REASON = `0x${"0".repeat(64)}`;

function mockRecord(impl: string): RegistryRecord {
  return {
    status: "VERIFIED",
    riskFlags: "0",
    verifiedAt: SCAN_FIXTURES.VERIFIED.record!.verifiedAt,
    impl,
    registrar: SCAN_FIXTURES.VERIFIED.record!.registrar,
    revokedAt: 0,
    reason: ZERO_REASON,
  };
}

/** The impostor's actual implementation — mockup "0xc30d…e812". */
const IMPOSTOR_IMPL = "0xc30d00000000000000000000000000000000e812";

export class MockGuardProbeSource implements GuardProbeSource {
  async getRecord(token: string): Promise<RegistryRecord | null> {
    await Promise.resolve();
    const key = token.toLowerCase();
    if (key === MOCK_VERIFIED_ADDR.toLowerCase()) return SCAN_FIXTURES.VERIFIED.record!;
    if (key === MOCK_REVOKED_ADDR.toLowerCase()) return MOCK_REVOKED_RECORD;
    if (key === MOCK_IMPL_MISMATCH_ADDR.toLowerCase()) {
      // record still points at the canonical impl; actual implementation differs → GUARD_IMPL_MISMATCH
      return mockRecord(SCAN_FIXTURES.VERIFIED.record!.impl);
    }
    if (key === MOCK_PAUSED_ADDR.toLowerCase() || key === MOCK_BLOCKLISTED_ADDR.toLowerCase()) {
      return mockRecord(SCAN_FIXTURES.VERIFIED.record!.impl);
    }
    // impostor twin and unknown tokens have no record — GUARD_NO_RECORD, never guessed
    return null;
  }

  async paused(token: string): Promise<boolean | null> {
    await Promise.resolve();
    return token.toLowerCase() === MOCK_PAUSED_ADDR.toLowerCase();
  }

  async isBlocked(token: string, _buyer: string): Promise<boolean | null> {
    await Promise.resolve();
    return token.toLowerCase() === MOCK_BLOCKLISTED_ADDR.toLowerCase();
  }

  async resolvedImpl(token: string): Promise<string | null> {
    await Promise.resolve();
    if (token.toLowerCase() === MOCK_IMPL_MISMATCH_ADDR.toLowerCase()) return IMPOSTOR_IMPL;
    const record = await this.getRecord(token);
    return record?.impl ?? null;
  }
}

export function getGuardProbeSource(): GuardProbeSource {
  if (isLiveApi()) {
    const registry = import.meta.env.VITE_REGISTRY_ADDRESS as `0x${string}` | undefined;
    if (!registry) {
      throw new Error("VITE_API_MODE=live requires VITE_REGISTRY_ADDRESS (the deployed registry — deployments/<chain>.json)");
    }
    const client = createPublicClient({
      chain: VETTED_CHAIN,
      transport: http(VETTED_CHAIN.rpcUrls.default.http[0]),
    });
    return new ViemGuardProbeSource(client, registry);
  }
  return new MockGuardProbeSource();
}

// ============================================================================
// Execution — request/result, error classification (verbatim reasons), and
// the mock executor (derives its outcome from the same preview, so the
// preview and the on-chain story never disagree).
// ============================================================================

export interface GuardExecutionRequest {
  tokenIn: string;
  tokenOut: string;
  amountIn: string;
  buyer: string;
}

export type GuardExecutionResult =
  | { kind: "settled"; txHash: string; gasUsed: string; amountOut: string }
  | { kind: "reverted"; reason: GuardRevertReason; txHash: string };

export type GuardExecutor = (req: GuardExecutionRequest) => Promise<GuardExecutionResult>;

export class WalletRejectedError extends Error {
  constructor() {
    super("rejected in wallet");
  }
}
export class GasFailureError extends Error {
  constructor() {
    super("gas failure");
  }
}
export class GuardRevertedError extends Error {
  constructor(
    public readonly reason: GuardRevertReason,
    public readonly txHash?: string,
  ) {
    super(reason);
  }
}

/** Guard fixtures give the verbatim reason its user-facing sentence. */
export function guardReasonDescription(reason: GuardRevertReason): string {
  return GUARD_FIXTURES[reason].description;
}

function findInCauseChain(e: unknown, pred: (err: Error) => boolean): Error | null {
  let cur: unknown = e;
  for (let depth = 0; cur instanceof Error && depth < 8; depth += 1) {
    if (pred(cur)) return cur;
    cur = cur.cause;
  }
  return null;
}

/** Surface the guard's verbatim revert string from a viem revert error, if present. */
export function extractRevertReason(e: unknown): GuardRevertReason | null {
  const reverted = findInCauseChain(e, (err) => err instanceof ContractFunctionRevertedError);
  if (!reverted) return null;
  const details = (reverted as InstanceType<typeof ContractFunctionRevertedError>).details ?? "";
  return GUARD_REVERT_REASONS.find((r) => details.includes(r)) ?? null;
}

export type ExecuteFailure =
  | { kind: "rejected" }
  | { kind: "gas" }
  | { kind: "reverted"; reason: GuardRevertReason | null };

/** Map wallet/viem failures onto the swap screen's nothing-sent / refused states (journeys.md:34). */
export function classifyExecuteError(e: unknown): ExecuteFailure {
  if (findInCauseChain(e, (err) => err instanceof UserRejectedRequestError) !== null) {
    return { kind: "rejected" };
  }
  if (
    findInCauseChain(
      e,
      (err) => err instanceof EstimateGasExecutionError || err.name === "TransactionExecutionError",
    ) !== null
  ) {
    return { kind: "gas" };
  }
  const reason = extractRevertReason(e);
  if (reason !== null || findInCauseChain(e, (err) => err instanceof ContractFunctionRevertedError) !== null) {
    return { kind: "reverted", reason };
  }
  return { kind: "rejected" }; // unknown broadcast failure — nothing was sent either way
}

/** Mock executor: preview determines the outcome, so preview and refusal always agree. */
export function createMockGuardExecutor(source: GuardProbeSource): GuardExecutor {
  return async (req) => {
    await Promise.resolve();
    const preview = await previewGuard(source, { token: req.tokenOut, buyer: req.buyer });
    if (preview.firstRevert !== null) {
      return { kind: "reverted", reason: preview.firstRevert, txHash: MOCK_TX.revert };
    }
    const amount = Number(req.amountIn.replace(/,/g, "")) || 0;
    const known = [MOCK_VERIFIED_ADDR, MOCK_REVOKED_ADDR].some((a) => a.toLowerCase() === req.tokenOut.toLowerCase());
    return { kind: "settled", txHash: MOCK_TX.settle, gasUsed: "231,504", amountOut: (amount * (known ? 0.000784 : 1)).toFixed(2) };
  };
}
