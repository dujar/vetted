/** Hash router — including the journeys.md:9 bare-query entry format (review round 1 blocking fix). */
import { describe, expect, it } from "vitest";

import { parseHash } from "../src/lib/router";

describe("parseHash", () => {
  it("parses the hash query when present (#/?addr=…)", () => {
    const route = parseHash("#/?addr=0xabc", "");
    expect(route.path).toBe("/");
    expect(route.query.getAll("addr")).toEqual(["0xabc"]);
  });

  it("falls back to location.search for the bare journeys.md:9 format (?addr=…, empty hash)", () => {
    const route = parseHash("", "?addr=0xabc&addr=0xdef");
    expect(route.path).toBe("/");
    expect(route.query.getAll("addr")).toEqual(["0xabc", "0xdef"]);
  });

  it("hash query wins over a stale bare search after in-app navigation", () => {
    const route = parseHash("#/?addr=0x111", "?addr=0x222");
    expect(route.query.getAll("addr")).toEqual(["0x111"]);
  });

  it("parses screen paths with and without a query", () => {
    expect(parseHash("#/swap", "").path).toBe("/swap");
    expect(parseHash("#/registry", "?addr=0x1").path).toBe("/registry");
    expect(parseHash("#/", "").path).toBe("/");
    expect(parseHash("", "").path).toBe("/");
  });
});
