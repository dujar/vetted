/** Wire/transport seam tests — mock mode serves the golden fixtures; live mode builds wire URLs. */
import { describe, expect, it, vi, afterEach } from "vitest";

import {
  FetchScanClient,
  FetchWatchdogSource,
  MOCK_WATCHDOG_PROVENANCE_URL,
  MockScanClient,
  MockWatchdogSource,
  getScanClient,
  getWatchdogSource,
  isLiveApi,
} from "../src/lib/api";
import {
  MOCK_IMPOSTOR_ADDR,
  MOCK_NOT_CONTRACT_ADDR,
  MOCK_REVOKED_ADDR,
  MOCK_RPC_ERROR_ADDR,
  MOCK_VERIFIED_ADDR,
  SCAN_FIXTURES,
} from "../src/lib/mockData";

afterEach(() => {
  vi.unstubAllEnvs();
});

describe("MockScanClient", () => {
  const client = new MockScanClient();

  it("serves the VERIFIED fixture verbatim at the fixture address", async () => {
    expect(await client.scan(4663, MOCK_VERIFIED_ADDR)).toEqual(SCAN_FIXTURES.VERIFIED);
  });

  it("serves the IMPOSTOR fixture at the impostor address", async () => {
    expect(await client.scan(4663, MOCK_IMPOSTOR_ADDR)).toEqual(SCAN_FIXTURES.IMPOSTOR);
  });

  it("serves the REVOKED fixture (with the requested addr) at the mock revoked address", async () => {
    const res = await client.scan(4663, MOCK_REVOKED_ADDR);
    expect(res.verdict).toBe("REVOKED");
    expect(res.revocationTx).toBe(SCAN_FIXTURES.REVOKED.revocationTx);
    expect(res.record?.status).toBe("REVOKED");
    expect(res.addr).toBe(MOCK_REVOKED_ADDR);
  });

  it("serves the UNVERIFIED fixture (degraded) at the fixture address", async () => {
    const res = await client.scan(4663, SCAN_FIXTURES.UNVERIFIED.addr);
    expect(res.verdict).toBe("UNVERIFIED");
    expect(res.degraded).toBe(true);
  });

  it("returns NOT_CONTRACT (no verdict attempted) for the EOA sentinel", async () => {
    const res = await client.scan(4663, MOCK_NOT_CONTRACT_ADDR);
    expect(res.verdict).toBeNull();
    expect(res.terminalState).toBe("NOT_CONTRACT");
  });

  it("returns RPC_RETRYABLE for the rpc sentinel", async () => {
    const res = await client.scan(4663, MOCK_RPC_ERROR_ADDR);
    expect(res.verdict).toBeNull();
    expect(res.terminalState).toBe("RPC_RETRYABLE");
  });

  it("defaults unknown contracts to UNVERIFIED off-pattern (depth boundary, never guessed)", async () => {
    const res = await client.scan(4663, `0x${"1".repeat(40)}`);
    expect(res.verdict).toBe("UNVERIFIED");
    expect(res.terminalState).toBeNull();
    expect(res.powerReport[0]?.severity).toBe("advisory");
  });

  it("marks non-4663 scans read-only with a notice (journeys.md:19)", async () => {
    const res = await client.scan(421614, MOCK_VERIFIED_ADDR);
    expect(res.notice).toContain("4663");
    expect(res.verdict).toBe("UNVERIFIED");
  });
});

describe("MockWatchdogSource", () => {
  it("serves the spec numbers inline (6,092 runs / ~150 per day) with the published-baseline provenance", async () => {
    expect(await new MockWatchdogSource().stats(4663)).toEqual({
      chainId: 4663,
      runs: 6092,
      baselinePerDay: 150,
      provenanceUrl: MOCK_WATCHDOG_PROVENANCE_URL,
    });
  });
});

describe("live clients", () => {
  it("GETs /scan with chainId + addr query params", async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(JSON.stringify(SCAN_FIXTURES.VERIFIED), { status: 200 }));
    vi.stubGlobal("fetch", fetchMock);
    const res = await new FetchScanClient("https://worker.example/").scan(4663, MOCK_VERIFIED_ADDR);
    expect(res.verdict).toBe("VERIFIED");
    expect(fetchMock).toHaveBeenCalledWith(
      `https://worker.example/scan?chainId=4663&addr=${encodeURIComponent(MOCK_VERIFIED_ADDR)}`,
    );
  });

  it("throws on scan-backend failure (UI owns the retryable state)", async () => {
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(new Response("boom", { status: 500 })));
    await expect(new FetchScanClient("https://worker.example").scan(4663, MOCK_VERIFIED_ADDR)).rejects.toThrow("500");
  });

  it("GETs /watchdog with the chainId and carries provenanceUrl (the shared wire type)", async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response('{"chainId":4663,"runs":6092,"baselinePerDay":150,"provenanceUrl":"https://docs.robinhood.com/chain/contracts"}', { status: 200 }));
    vi.stubGlobal("fetch", fetchMock);
    expect(await new FetchWatchdogSource("https://worker.example").stats(4663)).toEqual({
      chainId: 4663,
      runs: 6092,
      baselinePerDay: 150,
      provenanceUrl: "https://docs.robinhood.com/chain/contracts",
    });
    expect(fetchMock).toHaveBeenCalledWith("https://worker.example/watchdog?chainId=4663");
  });
});

describe("env switch", () => {
  it("defaults to mock (app works with no backend)", () => {
    vi.stubEnv("VITE_API_MODE", "");
    expect(isLiveApi()).toBe(false);
    expect(getScanClient()).toBeInstanceOf(MockScanClient);
    expect(getWatchdogSource()).toBeInstanceOf(MockWatchdogSource);
  });

  it("switches to live clients on VITE_API_MODE=live + VITE_API_URL", () => {
    vi.stubEnv("VITE_API_MODE", "live");
    vi.stubEnv("VITE_API_URL", "https://worker.example");
    expect(isLiveApi()).toBe(true);
    expect(getScanClient()).toBeInstanceOf(FetchScanClient);
    expect(getWatchdogSource()).toBeInstanceOf(FetchWatchdogSource);
  });
});
