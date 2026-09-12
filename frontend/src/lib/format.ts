/** Small display helpers shared by the three screens. */

/** `0x9f8c24b7…d2e4` — first 4 + last 4 hex chars (mockup style `0x77be…41af`). */
export function shortAddr(addr: string): string {
  return addr.length > 12 ? `${addr.slice(0, 6)}…${addr.slice(-4)}` : addr;
}

/** Unix seconds → `2026-08-30`. */
export function formatDate(unixSeconds: number): string {
  return new Date(unixSeconds * 1000).toISOString().slice(0, 10);
}

/** `6092` → `6,092` (mockup / spec wording). */
export function formatInt(n: number): string {
  return n.toLocaleString("en-US");
}

/**
 * bytes32 → display string. The registry's `reason` carries ASCII text
 * ("beacon impl changed") right-aligned in the word; nulls are trimmed.
 * Non-decodable bytes fall back to the hex itself — never guessed.
 */
export function decodeBytes32(bytes32: string): string {
  if (!/^0x[0-9a-fA-F]{64}$/.test(bytes32)) return bytes32;
  const hex = bytes32.slice(2);
  let out = "";
  for (let i = 0; i < 64; i += 2) {
    const code = parseInt(hex.slice(i, i + 2), 16);
    if (code === 0) continue;
    out += code >= 0x20 && code < 0x7f ? String.fromCharCode(code) : ".";
  }
  return out.trim() === "" ? bytes32 : out;
}
