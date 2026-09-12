/**
 * Mock-mode data — the fixture-backed transport seam (plan task 1). All scan
 * verdicts come from the step-1 golden fixtures in packages/shared (imported,
 * never retranscribed); watchdog numbers are inline constants per verify.md
 * loose end 4 (no watchdog fixture exists — packages/shared round-trip tests
 * own the fixtures/ directory). Mock-only sentinels (revoked address, probe
 * outcomes) live here so the fixtures' bytes stay untouched.
 */
import type { RegistryRecord, ScanResponse } from "vetted-shared";

import VERIFIED_FIXTURE from "../../../packages/shared/fixtures/verdict/VERIFIED.json";
import IMPOSTOR_FIXTURE from "../../../packages/shared/fixtures/verdict/IMPOSTOR.json";
import UNVERIFIED_FIXTURE from "../../../packages/shared/fixtures/verdict/UNVERIFIED.json";
import REVOKED_FIXTURE from "../../../packages/shared/fixtures/verdict/REVOKED.json";
import GUARD_NO_RECORD from "../../../packages/shared/fixtures/guard/GUARD_NO_RECORD.json";
import GUARD_RECORD_REVOKED from "../../../packages/shared/fixtures/guard/GUARD_RECORD_REVOKED.json";
import GUARD_PAUSED from "../../../packages/shared/fixtures/guard/GUARD_PAUSED.json";
import GUARD_BLOCKLISTED from "../../../packages/shared/fixtures/guard/GUARD_BLOCKLISTED.json";
import GUARD_IMPL_MISMATCH from "../../../packages/shared/fixtures/guard/GUARD_IMPL_MISMATCH.json";

export const SCAN_FIXTURES = {
  VERIFIED: VERIFIED_FIXTURE as ScanResponse,
  IMPOSTOR: IMPOSTOR_FIXTURE as ScanResponse,
  UNVERIFIED: UNVERIFIED_FIXTURE as ScanResponse,
  REVOKED: REVOKED_FIXTURE as ScanResponse,
};

/** Guard revert fixtures, keyed by the verbatim revert reason (journeys.md:32). */
export const GUARD_FIXTURES = {
  GUARD_NO_RECORD: GUARD_NO_RECORD,
  GUARD_RECORD_REVOKED: GUARD_RECORD_REVOKED,
  GUARD_PAUSED: GUARD_PAUSED,
  GUARD_BLOCKLISTED: GUARD_BLOCKLISTED,
  GUARD_IMPL_MISMATCH: GUARD_IMPL_MISMATCH,
} as const;

/** `pad("d5da")` → `0x000000000000000000000000000000000000d5da` — readable sentinels. */
const pad = (hex: string): `0x${string}` => `0x${hex.padStart(40, "0")}`;
/** Mockup-style `first4…last4` addresses: 0x77be + zeros + 41af displays as `0x77be…41af`. */
const padEnds = (first4: string, last4: string): `0x${string}` =>
  `0x${first4}${"0".repeat(32)}${last4}`;
/** Mockup-style `first4…last4` tx hashes (shortAddr shows first4+last4). */
const padHash = (first4: string, last4: string): `0x${string}` =>
  `0x${first4}${"0".repeat(56)}${last4}`;

/** Canonical NVIDIA — the VERIFIED fixture's own address. */
export const MOCK_VERIFIED_ADDR = SCAN_FIXTURES.VERIFIED.addr;
/** Impostor twin — the IMPOSTOR fixture's address. */
export const MOCK_IMPOSTOR_ADDR = SCAN_FIXTURES.IMPOSTOR.addr;
/**
 * The canonical after its beacon upgrade — REVOKED fixture content served at a
 * mock-only address (the fixture itself reuses the impostor address; mock mode
 * needs one address per state). Demo beat: "verified yesterday, revoked today".
 */
export const MOCK_REVOKED_ADDR = padEnds("9f8c", "d2e5");
/** EOA / empty address → NOT_CONTRACT terminal state. */
export const MOCK_NOT_CONTRACT_ADDR = pad("dead");
/** Mock sentinel → RPC_RETRYABLE terminal state. */
export const MOCK_RPC_ERROR_ADDR = pad("bad1");

/** Mock swap-probe outcomes (the guard's deterministic red flags, one token each). */
export const MOCK_PAUSED_ADDR = pad("fa5e");
export const MOCK_BLOCKLISTED_ADDR = pad("b10c");
/** Token whose record still points at the canonical impl while its actual implementation differs → GUARD_IMPL_MISMATCH. */
export const MOCK_IMPL_MISMATCH_ADDR = pad("d1f5");
/** Mock USDG-style pay token. */
export const MOCK_USDG_ADDR = pad("d5da");

/** Mock watchdog stats — spec.md:8/:37 (6,092 runs in 6 weeks, ~150/day). */
export const MOCK_WATCHDOG = { runs: 6092, baselinePerDay: 150 } as const;

/** Mock guard receipt / tx hashes — display as the mockup's `0x2b97…44de` style. */
export const MOCK_TX = {
  settle: padHash("2b97", "44de"),
  revert: padHash("8d21", "9f3a"),
  broadcast: padHash("6e30", "c5d1"),
  revocation: padHash("f4c0", "71b8"),
} as const;

const isoDay = (iso: string): number => Math.floor(Date.parse(`${iso}T00:00:00Z`) / 1000);

/** The four registry.html tokens — known-token list, mock + live identical (verify.md loose end 5). */
export interface KnownToken {
  address: string;
  name: string;
  symbol: string;
  /** Fixture-derived record; null = no record on-chain (renders in the empty path). */
  record: RegistryRecord | null;
  /** Revocation tx — mock carries the fixture-style hash; live stays null until step 3 emits Revoked (verify.md loose end 6). */
  revocationTx: string | null;
}

const registrar = SCAN_FIXTURES.VERIFIED.record!.registrar;

export const KNOWN_TOKENS: KnownToken[] = [
  {
    address: MOCK_VERIFIED_ADDR,
    name: "NVIDIA",
    symbol: "NVDA",
    record: SCAN_FIXTURES.VERIFIED.record,
    revocationTx: null,
  },
  {
    address: padEnds("44e0", "b9c2"),
    name: "AMC Entertainment",
    symbol: "AMC",
    record: {
      status: "VERIFIED",
      riskFlags: "0",
      verifiedAt: isoDay("2026-08-30"),
      impl: padEnds("44e0", "b9c2"),
      registrar,
      revokedAt: 0,
      reason: `0x${"0".repeat(64)}`,
    },
    revocationTx: null,
  },
  {
    address: padEnds("1d4f", "88b3"),
    name: "OpenAI",
    symbol: "OPENAI",
    record: {
      status: "REVOKED",
      riskFlags: "0",
      verifiedAt: isoDay("2026-08-12"),
      impl: padEnds("1d4f", "88b3"),
      registrar,
      revokedAt: isoDay("2026-09-06"),
      reason: SCAN_FIXTURES.REVOKED.record!.reason,
    },
    revocationTx: MOCK_TX.revocation,
  },
  {
    address: padEnds("9c71", "02fa"),
    name: "SpaceX",
    symbol: "SPACEX",
    record: {
      status: "VERIFIED",
      riskFlags: "0",
      verifiedAt: isoDay("2026-09-02"),
      impl: padEnds("9c71", "02fa"),
      registrar,
      revokedAt: 0,
      reason: `0x${"0".repeat(64)}`,
    },
    revocationTx: null,
  },
];

/** Registry record served for the mock swap/scan "revoked canonical" address. */
export const MOCK_REVOKED_RECORD: RegistryRecord = {
  ...SCAN_FIXTURES.REVOKED.record!,
};

/**
 * Mock scan lookup — one ScanResponse per state, journeys.md state matrix:
 * 4 verdicts (fixtures verbatim), NOT_CONTRACT, RPC_RETRYABLE, and the
 * off-pattern default (depth boundary: unknown contracts are UNVERIFIED +
 * structural heuristics, never guessed).
 */
export function mockScanResponse(chainId: number, addr: string): ScanResponse {
  const key = addr.toLowerCase();

  if (chainId !== 4663) {
    // journeys.md:19 — non-4663 scans run read-only; stock-token verdicts exist only on 4663.
    return {
      chainId,
      addr,
      verdict: "UNVERIFIED",
      terminalState: null,
      degraded: false,
      notice: `Robinhood Stock Tokens live on Robinhood Chain · 4663 — ${chainId} is scanned read-only; switch the selector back for verdicts.`,
      powerReport: [],
      record: null,
      revocationTx: null,
    };
  }
  if (key === MOCK_NOT_CONTRACT_ADDR.toLowerCase()) {
    return {
      chainId,
      addr,
      verdict: null,
      terminalState: "NOT_CONTRACT",
      degraded: false,
      notice: null,
      powerReport: [],
      record: null,
      revocationTx: null,
    };
  }
  if (key === MOCK_RPC_ERROR_ADDR.toLowerCase()) {
    return {
      chainId,
      addr,
      verdict: null,
      terminalState: "RPC_RETRYABLE",
      degraded: false,
      notice: null,
      powerReport: [],
      record: null,
      revocationTx: null,
    };
  }
  if (key === MOCK_VERIFIED_ADDR.toLowerCase()) return SCAN_FIXTURES.VERIFIED;
  if (key === MOCK_IMPOSTOR_ADDR.toLowerCase()) return SCAN_FIXTURES.IMPOSTOR;
  if (key === SCAN_FIXTURES.UNVERIFIED.addr.toLowerCase()) return SCAN_FIXTURES.UNVERIFIED;
  if (key === MOCK_REVOKED_ADDR.toLowerCase()) {
    return { ...SCAN_FIXTURES.REVOKED, addr };
  }
  // Off-pattern default — depth boundary (spec.md:29): UNVERIFIED + advisory heuristics.
  return {
    chainId,
    addr,
    verdict: "UNVERIFIED",
    terminalState: null,
    degraded: false,
    notice: null,
    powerReport: [
      {
        check: "Structural heuristics (advisory — no signature match)",
        result: "OFF_PATTERN",
        severity: "advisory",
        evidenceUrl: `https://robinhoodchain.blockscout.com/address/${addr}?tab=code`,
      },
    ],
    record: null,
    revocationTx: null,
  };
}
