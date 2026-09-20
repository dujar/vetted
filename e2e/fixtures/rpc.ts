/**
 * Public-RPC route stub — intercepts the frontend's viem JSON-RPC traffic at
 * the network layer so the LIVE-build specs can force outcomes no live
 * failure can guarantee deterministically (plan Goal: "route-intercepted
 * where a live failure cannot be forced"). Two uses:
 *   1. canned `getRecord`/probe answers so the guard preview passes and the
 *      execute path reaches eth_sendTransaction (rejection / gas unhappy paths);
 *   2. forced call failures for the registry's degraded state (journeys.md:45).
 */
import type { Page } from "@playwright/test";

import { RPC_HOST_SUFFIXES, SELECTORS, chainIdHex } from "./constants";

const word = (hex: string): string => hex.replace(/^0x/, "").padStart(64, "0");

/** ABI-encode the seven-field registry record (wire.md) into a JSON-RPC result. */
export function encodeRegistryRecord(r: {
  status: 0 | 1;
  verifiedAt: number;
  impl: string;
  registrar: string;
  revokedAt?: number;
  reason?: string;
}): string {
  return `0x${[
    word(String(r.status)),
    word("0"),
    word(r.verifiedAt.toString(16)),
    word(r.impl),
    word(r.registrar),
    word((r.revokedAt ?? 0).toString(16)),
    word(r.reason ?? ""),
  ].join("")}`;
}

export interface RpcStubOptions {
  chainId: number;
  /** registry address (lowercase) → encoded getRecord result; absent keys decode to no-record. */
  records?: Record<string, string>;
  /** Force every eth_call to fail (degraded states). */
  failCalls?: boolean;
}

export async function stubPublicRpc(page: Page, opts: RpcStubOptions): Promise<void> {
  await page.route(
    (url) => RPC_HOST_SUFFIXES.some((suffix) => url.hostname === suffix),
    async (route) => {
      const body = route.request().postDataJSON() as { jsonrpc?: string; id?: number; method?: string; params?: unknown[] };
      if (!body?.method) {
        await route.fulfill({ status: 400, contentType: "application/json", body: '{"error":"not json-rpc"}' });
        return;
      }
      const reply = (result: unknown) =>
        route.fulfill({ contentType: "application/json", body: JSON.stringify({ jsonrpc: "2.0", id: body.id, result }) });
      switch (body.method) {
        case "eth_chainId":
          return reply(chainIdHex(opts.chainId));
        case "net_version":
          return reply(String(opts.chainId));
        case "eth_blockNumber":
          return reply("0x1");
        case "eth_gasPrice":
          return reply("0x1");
        case "eth_getBalance":
          return reply("0xde0b6b3a7640000");
        case "eth_getCode":
          return reply("0x");
        case "eth_getTransactionReceipt":
          return reply(null);
        case "eth_call": {
          if (opts.failCalls) {
            await route.fulfill({
              contentType: "application/json",
              body: JSON.stringify({ jsonrpc: "2.0", id: body.id, error: { code: -32000, message: "e2e forced rpc failure" } }),
            });
            return;
          }
          const call = (body.params?.[0] ?? {}) as { to?: string; data?: string };
          const data = (call.data ?? "").toLowerCase();
          if (data.startsWith(SELECTORS.getRecord)) {
            const record = opts.records?.[(call.to ?? "").toLowerCase()];
            return reply(record ?? "0x");
          }
          if (data.startsWith("0x5c975abb") || data.startsWith("0xfbac3951")) {
            // paused() / isBlocked(buyer) probes → definitive false (PROBE_SELECTORS, abi.ts)
            return reply(`0x${"0".repeat(64)}`);
          }
          return reply("0x");
        }
        default:
          return reply("0x");
      }
    },
  );
}
