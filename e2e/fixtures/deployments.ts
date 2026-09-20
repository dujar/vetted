/**
 * Fixture loader for `deployments/*.json` (plan task 1) — the scratch chains
 * this suite runs against. Tolerant by design (verify loose end 10): the
 * writer is step 3's / step 6's FUNDED deploy.sh run, which has not happened
 * (operator key 0 wei on all three chains — step-3 findings), so missing
 * files and PENDING placeholders are skipped, never crashed on. Specs gated
 * on scratch state call `test.skip(!SCRATCH, …)` and the runbook in
 * e2e/README.md says how to activate them.
 */
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";

const ADDR_RE = /^0x[0-9a-fA-F]{40}$/;

export interface ScratchDeployment {
  chainId: number;
  file: string;
  registry: string;
  guard: string;
  tokens: {
    replica?: string;
    twin1?: string;
    twin2?: string;
    tokenA?: string;
    tokenB?: string;
    tokenPlain?: string;
  };
}

const addr = (v: unknown): string | undefined => (typeof v === "string" && ADDR_RE.test(v) ? v.toLowerCase() : undefined);

/**
 * First scratch deployment that is actually usable, preferring 46630 (the
 * step-6 rehearsal chain; step 3 also writes it). `status: "deployed"` plus a
 * well-formed registry+guard pair is the bar — anything less (PENDING
 * addresses, partial JSON) degrades to null so scratch-gated specs skip.
 */
export function readScratchDeployment(root: string): ScratchDeployment | null {
  for (const chain of ["46630", "421614"]) {
    const file = path.join(root, "deployments", `${chain}.json`);
    if (!existsSync(file)) continue;
    let json: Record<string, unknown>;
    try {
      json = JSON.parse(readFileSync(file, "utf8"));
    } catch {
      continue; // half-written / placeholder file — treat as absent
    }
    if (json.status !== "deployed") continue;
    const registry = addr(json.registry);
    const guard = addr(json.guard);
    if (!registry || !guard) continue;
    const r = (json.replicas ?? {}) as Record<string, unknown>;
    const m = (json.mockTokens ?? {}) as Record<string, unknown>;
    return {
      chainId: Number(json.chainId ?? chain),
      file,
      registry,
      guard,
      tokens: {
        replica: addr(r.proxy),
        twin1: addr(r.twin1 ?? r.twinPlain),
        twin2: addr(r.twin2 ?? r.twinSelfProxy),
        tokenA: addr(m.tokenA),
        tokenB: addr(m.tokenB),
        tokenPlain: addr(m.tokenPlain),
      },
    };
  }
  return null;
}
