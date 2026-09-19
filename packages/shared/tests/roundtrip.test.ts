/**
 * TS round-trip: every golden fixture parses into the wire types and
 * re-serializes byte-identically (canonical form: 2-space JSON + trailing
 * newline, keys in interface order). The Rust side (tests/roundtrip.rs)
 * enforces the same — the two implementations cannot drift apart.
 */
import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";
import {
  GUARD_ABI_SIGNATURES,
  GUARD_REVERT_REASONS,
  PROBE_SELECTORS,
  REGISTRY_ABI_SIGNATURES,
  SELECTORS,
} from "../abi";
import type { GuardRevert, ScanResponse } from "../types";
import { TERMINAL_STATES, VERDICTS } from "../types";
import type { WatchdogStats } from "../watchdog";
import { toFunctionSelector } from "viem";

const ROOT = path.resolve(import.meta.dirname, "..");
const canonical = (file: string) => readFileSync(file, "utf8");
const roundtrip = (file: string) => {
  const raw = canonical(file);
  const parsed = JSON.parse(raw);
  return { raw, reSerialized: JSON.stringify(parsed, null, 2) + "\n", parsed };
};

describe("verdict fixtures", () => {
  const dir = path.join(ROOT, "fixtures", "verdict");
  const files = readdirSync(dir).sort();

  it("has exactly one fixture per verdict, named after it", () => {
    expect(files).toEqual([...VERDICTS].sort().map((v) => `${v}.json`));
  });

  for (const file of files) {
    it(`${file} round-trips byte-identically`, () => {
      const { raw, reSerialized, parsed } = roundtrip(path.join(dir, file));
      expect(reSerialized).toBe(raw);
      const scan = parsed as ScanResponse;
      expect(scan.verdict).toBe(file.replace(".json", ""));
      expect(scan.terminalState === null || VERDICTS.includes(scan.verdict as never)).toBe(true);
      if (scan.verdict === null) {
        expect(scan.terminalState).not.toBeNull();
        expect(TERMINAL_STATES).toContain(scan.terminalState);
      }
      for (const row of scan.powerReport) {
        expect(row.evidenceUrl).toMatch(/^https?:\/\//);
      }
      if (scan.verdict === "REVOKED") {
        expect(scan.record?.status).toBe("REVOKED");
        expect(scan.revocationTx).toMatch(/^0x[0-9a-f]{64}$/);
      }
      if (scan.verdict === "VERIFIED") {
        expect(scan.record?.status).toBe("VERIFIED");
        expect(scan.record?.revokedAt).toBe(0);
      }
    });
  }
});

describe("guard fixtures", () => {
  const dir = path.join(ROOT, "fixtures", "guard");
  const files = readdirSync(dir).sort();

  it("has exactly one fixture per guard revert reason, named after it", () => {
    expect(files).toEqual(GUARD_REVERT_REASONS.slice().sort().map((r) => `${r}.json`));
  });

  for (const file of files) {
    it(`${file} round-trips byte-identically`, () => {
      const { raw, reSerialized, parsed } = roundtrip(path.join(dir, file));
      expect(reSerialized).toBe(raw);
      expect((parsed as GuardRevert).reason).toBe(file.replace(".json", ""));
      expect((parsed as GuardRevert).description.length).toBeGreaterThan(0);
    });
  }
});

describe("watchdog fixtures", () => {
  const dir = path.join(ROOT, "fixtures", "watchdog");
  const files = readdirSync(dir).sort();

  it("has exactly the OK and DEGRADED shapes", () => {
    expect(files).toEqual(["WATCHDOG_DEGRADED.json", "WATCHDOG_OK.json"]);
  });

  for (const file of files) {
    it(`${file} round-trips byte-identically`, () => {
      const { raw, reSerialized, parsed } = roundtrip(path.join(dir, file));
      expect(reSerialized).toBe(raw);
      const stats = parsed as WatchdogStats;
      expect(stats.chainId).toBe(4663);
      expect(stats.baselinePerDay).toBeGreaterThan(0);
      // The degrade sentinel: runs 0 carries a provenance link and is never
      // presented as a measured zero (wire.md, Watchdog API).
      if (stats.runs === 0) {
        expect(file).toBe("WATCHDOG_DEGRADED.json");
        expect(stats.provenanceUrl).toMatch(/^https?:\/\//);
      } else {
        expect(stats.runs).toBeGreaterThan(0);
        expect(stats.provenanceUrl).toMatch(/^https?:\/\//);
      }
    });
  }
});

describe("abi", () => {
  it("pins exactly the five registry/guard functions", () => {
    expect(REGISTRY_ABI_SIGNATURES).toHaveLength(3);
    expect(GUARD_ABI_SIGNATURES).toHaveLength(2);
  });

  it("selector constants match the human-readable signatures", () => {
    const pairs = [
      ["getRecord", "getRecord(address)"],
      ["verify", "verify(address,uint256,address)"],
      ["revoke", "revoke(address,string)"],
      ["commit", "commit(address,address,uint256,uint256)"],
      ["execute", "execute()"],
    ] as const;
    for (const [name, sig] of pairs) {
      expect(SELECTORS[name], sig).toBe(toFunctionSelector(sig));
    }
  });

  it("exposes the five guard reasons verbatim", () => {
    expect([...GUARD_REVERT_REASONS]).toEqual([
      "GUARD_NO_RECORD",
      "GUARD_RECORD_REVOKED",
      "GUARD_PAUSED",
      "GUARD_BLOCKLISTED",
      "GUARD_IMPL_MISMATCH",
    ]);
  });

  it("ships PROBE_SELECTORS as marked placeholders until step 2 calibrates", () => {
    for (const bytes of Object.values(PROBE_SELECTORS)) {
      expect(bytes).toMatch(/^0x[0-9a-f]{8}$/);
    }
  });
});
