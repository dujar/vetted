/**
 * Canonical Registry reads (plan task 4 / J3). The on-chain ABI is
 * point-lookup only (`getRecord`, no enumerate — abi.ts), so the table is
 * driven by the known-token list; live mode runs one getRecord per token plus
 * an erc20 symbol read through viem. Registry address comes from
 * VITE_REGISTRY_ADDRESS — deployments/*.json only exists after steps 3/7, and
 * importing a not-yet-created file would break the build (documented in
 * findings).
 *
 * Step 8: live reads target VETTED_CHAIN (4663 in the product; VITE_CHAIN_ID
 * re-points the bundle at a scratch chain), and VITE_REGISTRY_TOKENS overrides
 * the token list for chains whose known tokens are not the product table —
 * that is the live J3 UI path for the step-3/6 scratch deployments.
 */
import { createPublicClient, http, zeroAddress, type PublicClient } from "viem";
import { erc20Abi } from "viem";
import { REGISTRY_ABI } from "vetted-shared";
import type { RegistryRecord } from "vetted-shared";

import { VETTED_CHAIN } from "./chains";
import { isLiveApi } from "./api";
import { KNOWN_TOKENS, type KnownToken } from "./mockData";

export interface RegistryRow {
  token: string;
  name: string;
  symbol: string;
  record: RegistryRecord | null;
  /** Mock carries the fixture-style hash; live stays null until step 3 emits a Revoked event (verify.md loose end 6). */
  revocationTx: string | null;
}

export interface RegistrySource {
  rows(chainId: number): Promise<RegistryRow[]>;
}

/** Wire rule: status u8 outside {0,1} is no-record, never guessed (wire.md). */
const STATUS_BY_U8: Record<number, RegistryRecord["status"]> = { 0: "VERIFIED", 1: "REVOKED" };

interface RawRecord {
  status: number | bigint;
  riskFlags: bigint;
  verifiedAt: bigint | number;
  impl: string;
  registrar: string;
  revokedAt: bigint | number;
  reason: string;
}

/** Decode a viem getRecord result into the wire shape; null = no record (zero registrar or unknown status). */
export function decodeRegistryRecord(raw: RawRecord | null | undefined): RegistryRecord | null {
  if (!raw) return null;
  const status = STATUS_BY_U8[Number(raw.status)];
  if (status === undefined) return null;
  if (raw.registrar.toLowerCase() === zeroAddress) return null;
  return {
    status,
    riskFlags: raw.riskFlags.toString(),
    verifiedAt: Number(raw.verifiedAt),
    impl: raw.impl,
    registrar: raw.registrar,
    revokedAt: Number(raw.revokedAt),
    reason: raw.reason,
  };
}

export class MockRegistrySource implements RegistrySource {
  async rows(): Promise<RegistryRow[]> {
    await Promise.resolve();
    return KNOWN_TOKENS.map(({ address, name, symbol, record, revocationTx }) => ({
      token: address,
      name,
      symbol,
      record,
      revocationTx,
    }));
  }
}

/**
 * Live-mode known-token list. Product default: the fixture-derived
 * KNOWN_TOKENS table. VITE_REGISTRY_TOKENS (comma-separated 0x… addresses)
 * re-points the table at another chain's tokens — the e2e live pass feeds the
 * scratch deployments this way; names fall back to the live symbol reads.
 */
function knownTokens(): KnownToken[] {
  const raw = import.meta.env.VITE_REGISTRY_TOKENS ?? "";
  const addresses = raw
    .split(",")
    .map((s) => s.trim())
    .filter((a) => /^0x[0-9a-fA-F]{40}$/.test(a));
  if (addresses.length === 0) return KNOWN_TOKENS;
  return addresses.map((address) => ({ address, name: "", symbol: "", record: null, revocationTx: null }));
}

export class ViemRegistrySource implements RegistrySource {
  constructor(
    private readonly client: PublicClient,
    private readonly registryAddress: `0x${string}`,
  ) {}

  async rows(chainId: number): Promise<RegistryRow[]> {
    void chainId; // reads target VETTED_CHAIN — the client is built on it (journeys.md:42)
    return Promise.all(
      knownTokens().map(async ({ address, name, symbol }: KnownToken): Promise<RegistryRow> => {
        const raw = await this.client.readContract({
          address: this.registryAddress,
          abi: REGISTRY_ABI,
          functionName: "getRecord",
          args: [address as `0x${string}`],
        });
        const record = decodeRegistryRecord(raw);
        let symbolLive = symbol;
        try {
          symbolLive = await this.client.readContract({
            address: address as `0x${string}`,
            abi: erc20Abi,
            functionName: "symbol",
          });
        } catch {
          // constant symbol is the fallback — never blocks the table
        }
        return { token: address, name: name || symbolLive, symbol: symbolLive, record, revocationTx: null };
      }),
    );
  }
}

export function getRegistrySource(): RegistrySource {
  if (isLiveApi()) {
    const address = import.meta.env.VITE_REGISTRY_ADDRESS as `0x${string}` | undefined;
    if (!address) {
      throw new Error("VITE_API_MODE=live requires VITE_REGISTRY_ADDRESS (the deployed registry — deployments/<chain>.json)");
    }
    const client = createPublicClient({
      chain: VETTED_CHAIN,
      transport: http(VETTED_CHAIN.rpcUrls.default.http[0]),
    });
    return new ViemRegistrySource(client, address);
  }
  return new MockRegistrySource();
}
