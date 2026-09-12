/**
 * Guard preview/execution tests (J2) — one case per deterministic revert
 * reason (journeys.md:32) plus the all-clear path, error classification for
 * the nothing-sent states, and preview/execution agreement.
 */
import { describe, expect, it } from "vitest";
import { ContractFunctionRevertedError, UserRejectedRequestError } from "viem";
import { GUARD_REVERT_REASONS, type RegistryRecord } from "vetted-shared";

import {
  MockGuardProbeSource,
  classifyExecuteError,
  createMockGuardExecutor,
  extractRevertReason,
  previewGuard,
  guardReasonDescription,
  type GuardProbeSource,
} from "../src/lib/guard";
import {
  MOCK_BLOCKLISTED_ADDR,
  MOCK_IMPOSTOR_ADDR,
  MOCK_IMPL_MISMATCH_ADDR,
  MOCK_PAUSED_ADDR,
  MOCK_REVOKED_ADDR,
  MOCK_VERIFIED_ADDR,
} from "../src/lib/mockData";

const BUYER = `0x${"aa".repeat(20)}`;

function stubSource(overrides: Partial<GuardProbeSource> = {}): GuardProbeSource {
  const record: RegistryRecord = {
    status: "VERIFIED",
    riskFlags: "0",
    verifiedAt: 1788048000,
    impl: `0x${"77".repeat(20)}`,
    registrar: `0x${"5e".repeat(20)}`,
    revokedAt: 0,
    reason: `0x${"0".repeat(64)}`,
  };
  return {
    getRecord: async () => record,
    paused: async () => false,
    isBlocked: async () => false,
    resolvedImpl: async () => record.impl,
    ...overrides,
  };
}

describe("previewGuard — the four checks in guard order", () => {
  it("all clear: four ok rows, no revert, impl equality shown as the mockup does", async () => {
    const { rows, firstRevert } = await previewGuard(stubSource(), { token: MOCK_VERIFIED_ADDR, buyer: BUYER });
    expect(firstRevert).toBeNull();
    expect(rows.map((r) => r.ok)).toEqual([true, true, true, true]);
    expect(rows[3]!.detail).toMatch(/ = /);
    expect(rows.map((r) => r.label)).toEqual([
      "Live verification record in Canonical Registry",
      "Token not paused",
      "Buyer not blocklisted",
      "Implementation matches registry record",
    ]);
  });

  it("no record → GUARD_NO_RECORD", async () => {
    const { firstRevert } = await previewGuard(stubSource({ getRecord: async () => null }), {
      token: MOCK_IMPOSTOR_ADDR,
      buyer: BUYER,
    });
    expect(firstRevert).toBe("GUARD_NO_RECORD");
  });

  it("revoked record → GUARD_RECORD_REVOKED with the revoked date", async () => {
    const { firstRevert, rows } = await previewGuard(
      stubSource({
        getRecord: async () => ({
          status: "REVOKED",
          riskFlags: "0",
          verifiedAt: 1788048000,
          impl: `0x${"77".repeat(20)}`,
          registrar: `0x${"5e".repeat(20)}`,
          revokedAt: 1788652800,
          reason: `0x${"0".repeat(64)}`,
        }),
      }),
      { token: MOCK_REVOKED_ADDR, buyer: BUYER },
    );
    expect(firstRevert).toBe("GUARD_RECORD_REVOKED");
    expect(rows[0]!.detail).toContain("2026-09-06");
  });

  it("paused() true → GUARD_PAUSED", async () => {
    const { firstRevert, rows } = await previewGuard(stubSource({ paused: async () => true }), {
      token: MOCK_PAUSED_ADDR,
      buyer: BUYER,
    });
    expect(firstRevert).toBe("GUARD_PAUSED");
    expect(rows[1]!.detail).toBe("paused() → true");
  });

  it("buyer blocklisted → GUARD_BLOCKLISTED", async () => {
    const { firstRevert } = await previewGuard(stubSource({ isBlocked: async () => true }), {
      token: MOCK_BLOCKLISTED_ADDR,
      buyer: BUYER,
    });
    expect(firstRevert).toBe("GUARD_BLOCKLISTED");
  });

  it("impl drift → GUARD_IMPL_MISMATCH with ≠ detail", async () => {
    const { firstRevert, rows } = await previewGuard(stubSource({ resolvedImpl: async () => `0x${"c3".repeat(20)}` }), {
      token: MOCK_IMPL_MISMATCH_ADDR,
      buyer: BUYER,
    });
    expect(firstRevert).toBe("GUARD_IMPL_MISMATCH");
    expect(rows[3]!.detail).toMatch(/ ≠ /);
  });

  it("first deterministic red flag wins when several are red", async () => {
    const { firstRevert } = await previewGuard(
      stubSource({ paused: async () => true, isBlocked: async () => true }),
      { token: MOCK_VERIFIED_ADDR, buyer: BUYER },
    );
    expect(firstRevert).toBe("GUARD_PAUSED");
  });

  it("unavailable probes degrade to advisory, never to a guessed pass/fail", async () => {
    const { rows, firstRevert } = await previewGuard(
      stubSource({ paused: async () => null, isBlocked: async () => null, resolvedImpl: async () => null }),
      { token: MOCK_VERIFIED_ADDR, buyer: BUYER },
    );
    expect(firstRevert).toBeNull();
    expect(rows[1]!.detail).toContain("probe unavailable");
    expect(rows[3]!.ok).toBe(false); // cannot confirm impl match — not ok
    expect(rows[3]!.revertReason).toBeNull(); // but not a deterministic flag either
  });

  it("covers every reason in the shared GUARD_REVERT_REASONS constant", async () => {
    // Guard order guarantee: the union in vetted-shared is exactly what the preview can emit.
    expect(GUARD_REVERT_REASONS).toHaveLength(5);
    for (const reason of GUARD_REVERT_REASONS) {
      expect(guardReasonDescription(reason)).toMatch(/funds never moved/i);
    }
  });
});

describe("mock probe source (fixture-driven)", () => {
  const source = new MockGuardProbeSource();

  it("verified address → VERIFIED fixture record", async () => {
    expect((await source.getRecord(MOCK_VERIFIED_ADDR))?.status).toBe("VERIFIED");
  });
  it("revoked address → REVOKED record", async () => {
    expect((await source.getRecord(MOCK_REVOKED_ADDR))?.status).toBe("REVOKED");
  });
  it("impostor twin → no record (never guessed)", async () => {
    expect(await source.getRecord(MOCK_IMPOSTOR_ADDR)).toBeNull();
  });
  it("paused/blocklist sentinels probe true, everything else false", async () => {
    expect(await source.paused(MOCK_PAUSED_ADDR)).toBe(true);
    expect(await source.paused(MOCK_VERIFIED_ADDR)).toBe(false);
    expect(await source.isBlocked(MOCK_BLOCKLISTED_ADDR, BUYER)).toBe(true);
  });
  it("impl-mismatch token resolves to the impostor implementation", async () => {
    expect(await source.resolvedImpl(MOCK_IMPL_MISMATCH_ADDR)).toMatch(/^0xc30d/);
  });
});

describe("mock executor agrees with the preview", () => {
  const executor = createMockGuardExecutor(new MockGuardProbeSource());

  it("settles the canonical token with a receipt", async () => {
    const result = await executor({ tokenIn: `0x${"d5".repeat(20)}`, tokenOut: MOCK_VERIFIED_ADDR, amountIn: "1,250", buyer: BUYER });
    expect(result).toMatchObject({ kind: "settled", amountOut: "0.98" });
  });

  it("refuses the impostor twin (GUARD_IMPL_MISMATCH per its record drift)", async () => {
    const result = await executor({ tokenIn: `0x${"d5".repeat(20)}`, tokenOut: MOCK_IMPL_MISMATCH_ADDR, amountIn: "1,250", buyer: BUYER });
    expect(result).toMatchObject({ kind: "reverted", reason: "GUARD_IMPL_MISMATCH" });
  });

  it("refuses the revoked canonical — the upgrade beat", async () => {
    const result = await executor({ tokenIn: `0x${"d5".repeat(20)}`, tokenOut: MOCK_REVOKED_ADDR, amountIn: "1,250", buyer: BUYER });
    expect(result).toMatchObject({ kind: "reverted", reason: "GUARD_RECORD_REVOKED" });
  });
});

describe("error classification — nothing-sent vs refused", () => {
  it("wallet rejection → rejected", () => {
    const err = new Error("send failed", { cause: new UserRejectedRequestError(new Error("user rejected")) });
    expect(classifyExecuteError(err)).toEqual({ kind: "rejected" });
  });

  it("gas failure → gas", () => {
    const err = Object.assign(new Error("intrinsic gas too low"), { name: "TransactionExecutionError" });
    expect(classifyExecuteError(err)).toEqual({ kind: "gas" });
  });

  it("guard revert with a known reason → reverted + verbatim reason", () => {
    const base = new ContractFunctionRevertedError({ abi: [], functionName: "execute" });
    const err = new Error("simulated", {
      cause: Object.assign(base, { details: "GUARD_PAUSED: token paused" }),
    });
    expect(extractRevertReason(err)).toBe("GUARD_PAUSED");
    expect(classifyExecuteError(err)).toEqual({ kind: "reverted", reason: "GUARD_PAUSED" });
  });

  it("revert without a known reason stays reverted but untranscribed", () => {
    const err = new Error("simulated", {
      cause: Object.assign(new ContractFunctionRevertedError({ abi: [], functionName: "execute" }), {
        details: "something else",
      }),
    });
    expect(classifyExecuteError(err)).toEqual({ kind: "reverted", reason: null });
  });
});
