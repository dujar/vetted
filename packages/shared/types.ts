/**
 * Wire-contract types, TS side. Rust mirror: types.rs — both must round-trip
 * the golden fixtures in fixtures/ byte-identically (CI). Shape decisions and
 * on-chain mappings are documented in wire.md; this file mirrors it exactly.
 */
import { GUARD_REVERT_REASONS } from "./abi";

/** The four verdicts — the engine never guesses outside these (journeys.md:14). */
export const VERDICTS = ["VERIFIED", "IMPOSTOR", "UNVERIFIED", "REVOKED"] as const;
export type Verdict = (typeof VERDICTS)[number];

export type GuardRevertReason = (typeof GUARD_REVERT_REASONS)[number];

export interface GuardRevert {
  reason: GuardRevertReason;
  description: string;
}

/** Registry record status — on-chain u8: 0 = VERIFIED, 1 = REVOKED (wire.md). */
export const RECORD_STATUSES = ["VERIFIED", "REVOKED"] as const;
export type RecordStatus = (typeof RECORD_STATUSES)[number];

/**
 * Seven-field registry record (step-3 plan). `riskFlags` is a u256 carried as
 * a decimal string; timestamps are unix seconds; `revokedAt`/`reason` are zero
 * while status is VERIFIED.
 */
export interface RegistryRecord {
  status: RecordStatus;
  riskFlags: string;
  verifiedAt: number;
  impl: string;
  registrar: string;
  revokedAt: number;
  reason: string;
}

/** GET /scan?chainId=&addr= — query params (step-4 plan). */
export interface ScanRequest {
  chainId: number;
  addr: string;
}

/** Severity of a power-report row — drives theme color (green/red duality; amber advisory). */
export type Severity = "risk" | "advisory" | "verified";

export interface PowerReportRow {
  check: string;
  result: string;
  severity: Severity;
  /** Every row carries evidence: tx / slot / bytecode diff / probe result (spec.md:30). */
  evidenceUrl: string;
}

/** Terminal states — no verdict attempted, nothing guessed. */
export const TERMINAL_STATES = ["NOT_CONTRACT", "RPC_RETRYABLE"] as const;
export type TerminalState = (typeof TERMINAL_STATES)[number];

export interface ScanResponse {
  chainId: number;
  addr: string;
  /** null only when terminalState is set. */
  verdict: Verdict | null;
  terminalState: TerminalState | null;
  /** Issuer canonical list unreachable — all verdicts UNVERIFIED (journeys.md:20). */
  degraded: boolean;
  /** Non-4663 chains: stock-token verdicts exist only on 4663 (journeys.md:19). */
  notice: string | null;
  powerReport: PowerReportRow[];
  record: RegistryRecord | null;
  /** Revocation tx for REVOKED verdicts (journeys.md:14). */
  revocationTx: string | null;
}
