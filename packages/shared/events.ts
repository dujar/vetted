/**
 * Canonical Registry event signatures — pinned here with the contracts
 * (step 3, verify.md loose end 3), same discipline as abi.ts: step 3's
 * deployed contracts must match these byte-for-byte. Consumers: step 4
 * extracts the revocation tx from Revoke logs; step 5 renders REVOKED
 * reason + evidence from the same event (journeys.md:42).
 */
import { parseAbi } from "viem";

/** Human-readable signatures — the Rust `sol!` declarations match these
 * (field names are cosmetic; the canonical signature string excludes them). */
export const REGISTRY_EVENT_SIGNATURES = [
  "event Verify(address indexed token, uint256 riskFlags, address indexed impl)",
  "event Revoke(address indexed token, string reason)",
] as const;

/** Parsed, ready for viem log decoding (steps 4/5/8). */
export const REGISTRY_EVENTS_ABI = parseAbi([...REGISTRY_EVENT_SIGNATURES]);

/**
 * topic0 = keccak256(canonical signature) — the hashes the deployed
 * contracts emit. Recomputed against REGISTRY_EVENT_SIGNATURES by
 * tests/events.test.ts as the tripwire.
 */
export const REGISTRY_EVENT_TOPICS = {
  Verify: "0x3e797825af25f16433592602e044447c6902a26f7c31d1918940c56b824c7db3",
  Revoke: "0x2fa80445a7995a05a1a47457227da064b86a578212322a7cd41a235d469749a1",
} as const;

/**
 * revoke(token, reason) conversion rule (verify.md loose end 3): the
 * record's bytes32 `reason` field stores keccak256(utf8(reason)); the full
 * human-readable text rides the Revoke event, which is what J3 renders.
 * While status is VERIFIED, record.reason stays zero (wire.md).
 */
export const REVOKE_REASON_RULE = "record.reason = keccak256(utf8(reason)); full text in the Revoke event" as const;
