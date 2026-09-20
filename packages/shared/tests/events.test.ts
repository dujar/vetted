/**
 * Event-pin tripwire (step 3, verify.md loose end 3) — mirrors abi.ts's
 * selector test: recomputes the topic0 hashes with viem and compares to the
 * pinned literals in events.ts. Byte drift breaks here, not on-chain.
 */
import { describe, expect, it } from "vitest";
import { keccak256, toEventSelector, toHex } from "viem";
import { REGISTRY_EVENT_SIGNATURES, REGISTRY_EVENT_TOPICS } from "../events";

/** viem wants the bare canonical signature (no `event` keyword, no names). */
const CANONICAL: Record<(typeof REGISTRY_EVENT_SIGNATURES)[number], string> = {
  "event Verify(address indexed token, uint256 riskFlags, address indexed impl)":
    "Verify(address,uint256,address)",
  "event Revoke(address indexed token, string reason)": "Revoke(address,string)",
};

describe("registry event pins", () => {
  it("topic0 hashes recompute from the canonical signatures", () => {
    for (const sig of REGISTRY_EVENT_SIGNATURES) {
      const name = CANONICAL[sig].slice(0, CANONICAL[sig].indexOf("(")) as "Verify" | "Revoke";
      expect(REGISTRY_EVENT_TOPICS[name]).toBe(toEventSelector(CANONICAL[sig]));
    }
  });

  it("Verify topic0 is the pinned literal", () => {
    expect(toEventSelector("Verify(address,uint256,address)")).toBe(
      "0x3e797825af25f16433592602e044447c6902a26f7c31d1918940c56b824c7db3",
    );
  });

  it("Revoke topic0 is the pinned literal", () => {
    expect(toEventSelector("Revoke(address,string)")).toBe(
      "0x2fa80445a7995a05a1a47457227da064b86a578212322a7cd41a235d469749a1",
    );
  });
});
