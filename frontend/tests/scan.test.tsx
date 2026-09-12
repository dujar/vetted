/**
 * Scan screen state coverage (J1) — one test per journeys.md state matrix
 * entry: empty hero, in-order progress, four verdicts, compare deep link,
 * watchdog, and every unhappy path.
 */
import { fireEvent, render, screen, act } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { ScanPage } from "../src/pages/ScanPage";
import { MockScanClient, MockWatchdogSource, type ScanClient, type WatchdogSource } from "../src/lib/api";
import {
  MOCK_IMPOSTOR_ADDR,
  MOCK_NOT_CONTRACT_ADDR,
  MOCK_REVOKED_ADDR,
  MOCK_RPC_ERROR_ADDR,
  MOCK_VERIFIED_ADDR,
  SCAN_FIXTURES,
} from "../src/lib/mockData";

const watchdog: WatchdogSource = new MockWatchdogSource();

function renderScan(addr: string | string[] | null, client: ScanClient = new MockScanClient(), chainId = 4663) {
  const prevHash = window.location.hash;
  if (addr === null) window.location.hash = "#/";
  else {
    const list = Array.isArray(addr) ? addr : [addr];
    window.location.hash = `#/?${list.map((a) => `addr=${a}`).join("&")}`;
  }
  const onChainChange = vi.fn();
  const utils = render(
    <ScanPage chainId={chainId} onChainChange={onChainChange} client={client} watchdog={watchdog} />,
  );
  return { onChainChange, ...utils, restoreHash: () => (window.location.hash = prevHash) };
}

beforeEach(() => {
  window.location.hash = "#/";
});
afterEach(() => {
  window.location.hash = "";
});

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((r) => (resolve = r));
  return { promise, resolve };
}

describe("empty hero", () => {
  it("renders the mockup hero with input + Scan + try-links", () => {
    renderScan(null);
    expect(screen.getByText("Is this stock token real — and what can it do to you?")).toBeTruthy();
    expect(screen.getByLabelText("token address")).toBeTruthy();
    expect(screen.getByText("Scan")).toBeTruthy();
    expect(screen.getByText("canonical NVIDIA (NVDA)")).toBeTruthy();
    expect(screen.getByText("impostor twin — side-by-side compare")).toBeTruthy();
  });

  it("rejects a malformed address without scanning", () => {
    renderScan(null);
    fireEvent.change(screen.getByLabelText("token address"), { target: { value: "not-an-address" } });
    fireEvent.click(screen.getByText("Scan"));
    expect(screen.getByRole("alert").textContent).toMatch(/40 hex/);
  });
});

describe("scanning progress (journeys.md:13, in order)", () => {
  it("reveals the probe lines in order while the scan is in flight", async () => {
    vi.useFakeTimers();
    const d = deferred<never>();
    const pendingClient: ScanClient = { scan: () => d.promise };
    const { container } = renderScan(MOCK_VERIFIED_ADDR, pendingClient);
    try {
      await act(async () => {
        await vi.advanceTimersByTimeAsync(10);
      });
      const log = () => container.querySelector('[aria-label="scanning"]')!.textContent!;
      expect(log()).not.toMatch(/✓.*fetch bytecode/s);
      await act(async () => {
        await vi.advanceTimersByTimeAsync(460);
      });
      expect(log()).toMatch(/^✓ fetch bytecode/);
      await act(async () => {
        await vi.advanceTimersByTimeAsync(460 * 2);
      });
      const text = log();
      expect(text.indexOf("fetch bytecode")).toBeLessThan(text.indexOf("resolve EIP-1967 implementation slot"));
      expect(text.indexOf("resolve EIP-1967 implementation slot")).toBeLessThan(
        text.indexOf("run probes: paused() · buyer blocklist"),
      );
      expect(text.indexOf("run probes: paused() · buyer blocklist")).toBeLessThan(
        text.indexOf("fetching issuer canonical list"),
      );
    } finally {
      vi.useRealTimers();
      d.resolve(undefined as never);
    }
  });
});

describe("verdict states (fixture-driven)", () => {
  it("VERIFIED: badge, name, power report with evidence links, verified-≠-safe line", async () => {
    renderScan(MOCK_VERIFIED_ADDR);
    expect(await screen.findByText("VERIFIED")).toBeTruthy();
    expect(screen.getByText("NVIDIA")).toBeTruthy();
    expect(screen.getByText("Buyer blocklist in modifier")).toBeTruthy();
    const evidence = screen.getAllByText("evidence");
    expect(evidence.length).toBeGreaterThanOrEqual(3);
    expect((evidence[0] as HTMLAnchorElement).href).toContain("blockscout.com");
    expect(screen.getByText(/Verified ≠ safe/)).toBeTruthy();
    expect(screen.getByText("Swap guarded →")).toBeTruthy();
  });

  it("VERIFIED: watchdog widget shows the spec numbers beside the verdict", async () => {
    renderScan(MOCK_VERIFIED_ADDR);
    await screen.findByText("VERIFIED");
    expect(await screen.findByText("6,092")).toBeTruthy();
    expect(screen.getByText("~150/day")).toBeTruthy();
    expect(screen.getByText("ground truth")).toBeTruthy();
  });

  it("IMPOSTOR: risk card with positive-evidence rows and guard link", async () => {
    renderScan(MOCK_IMPOSTOR_ADDR);
    expect(await screen.findByText("IMPOSTOR")).toBeTruthy();
    expect(screen.getByText("This is not a Robinhood Stock Token.")).toBeTruthy();
    expect(screen.getByText("Metadata mimics canonical NVIDIA (NVDA)")).toBeTruthy();
    expect(screen.getByText("Watch the guard refuse it on-chain →")).toBeTruthy();
  });

  it("UNVERIFIED: depth-boundary notice visible (spec.md:29)", async () => {
    renderScan(SCAN_FIXTURES.UNVERIFIED.addr);
    expect(await screen.findByText("UNVERIFIED")).toBeTruthy();
    expect(screen.getByText(/Depth boundary: full verdicts cover the Robinhood stock-token pattern/)).toBeTruthy();
  });

  it("UNVERIFIED + degraded flag shows the degraded banner (journeys.md:20)", async () => {
    renderScan(SCAN_FIXTURES.UNVERIFIED.addr);
    await screen.findByText("UNVERIFIED");
    expect(
      screen.getByText("Issuer canonical list unreachable. All verdicts drop to UNVERIFIED until it responds."),
    ).toBeTruthy();
  });

  it("REVOKED: revoked banner with revocation tx link (journeys.md:14)", async () => {
    renderScan(MOCK_REVOKED_ADDR);
    expect(await screen.findByText("REVOKED")).toBeTruthy();
    expect(screen.getByText("Verification revoked — this token changed since it was verified.")).toBeTruthy();
    const tx = screen.getByText("0xab12…ab12");
    expect((tx as HTMLAnchorElement).href).toContain("/tx/0xab12");
    expect(screen.getByText(/beacon implementation changed 2026-09-06/)).toBeTruthy();
    expect(screen.getByText(/The registry record is stale by definition/)).toBeTruthy();
  });
});

describe("compare deep link (?addr=&addr=)", () => {
  it("renders two verdict cards side by side — the demo's impostor-vs-canonical beat", async () => {
    renderScan([MOCK_VERIFIED_ADDR, MOCK_IMPOSTOR_ADDR]);
    expect(await screen.findByText("VERIFIED")).toBeTruthy();
    expect(screen.getByText("IMPOSTOR")).toBeTruthy();
    expect(screen.getByText(MOCK_VERIFIED_ADDR)).toBeTruthy();
    expect(screen.getByText(MOCK_IMPOSTOR_ADDR)).toBeTruthy();
  });

  it("also works as the journeys.md:9 bare query (?addr=0xA…&addr=0xB…, no hash)", async () => {
    window.history.replaceState(null, "", `/?addr=${MOCK_VERIFIED_ADDR}&addr=${MOCK_IMPOSTOR_ADDR}`);
    try {
      render(<ScanPage chainId={4663} client={new MockScanClient()} watchdog={watchdog} />);
      expect(await screen.findByText("VERIFIED")).toBeTruthy();
      expect(screen.getByText("IMPOSTOR")).toBeTruthy();
    } finally {
      window.history.replaceState(null, "", "/");
    }
  });
});

describe("bare-query entry format (journeys.md:9 — blocking review fix)", () => {
  it("scans ?addr=0x… with no hash instead of rendering the hero", async () => {
    window.history.replaceState(null, "", `/?addr=${MOCK_VERIFIED_ADDR}`);
    try {
      render(<ScanPage chainId={4663} client={new MockScanClient()} watchdog={watchdog} />);
      expect(await screen.findByText("VERIFIED")).toBeTruthy();
      expect(screen.queryByText("Is this stock token real — and what can it do to you?")).toBeNull();
    } finally {
      window.history.replaceState(null, "", "/");
    }
  });
});

describe("unhappy paths", () => {
  it("not a contract: no verdict attempted", async () => {
    renderScan(MOCK_NOT_CONTRACT_ADDR);
    expect(await screen.findByText("not a contract")).toBeTruthy();
    expect(screen.getByText(/has no code — a wallet address, not a token/)).toBeTruthy();
    expect(screen.queryByText("VERIFIED")).toBeNull();
  });

  it("RPC_RETRYABLE: retryable error state, nothing guessed", async () => {
    renderScan(MOCK_RPC_ERROR_ADDR);
    expect(await screen.findByText("rpc unreachable")).toBeTruthy();
    expect(screen.getByText(/No partial verdicts are shown/)).toBeTruthy();
    expect(screen.getByText("Retry scan")).toBeTruthy();
  });

  it("transport throw renders the retryable error card; retry re-runs the scan", async () => {
    let calls = 0;
    const flaky: ScanClient = {
      scan: async () => {
        calls += 1;
        if (calls === 1) throw new Error("timeout");
        return SCAN_FIXTURES.VERIFIED;
      },
    };
    renderScan(MOCK_VERIFIED_ADDR, flaky);
    expect(await screen.findByText("rpc unreachable")).toBeTruthy();
    fireEvent.click(screen.getByText("Retry scan"));
    expect(await screen.findByText("VERIFIED")).toBeTruthy();
  });

  it("non-4663 selector: read-only notice + switch back (journeys.md:19)", async () => {
    const { onChainChange } = renderScan(MOCK_VERIFIED_ADDR, new MockScanClient(), 421614);
    expect(await screen.findByText("wrong network")).toBeTruthy();
    expect(screen.getByText(/scanned read-only/)).toBeTruthy();
    fireEvent.click(screen.getByText("Switch to 4663"));
    expect(onChainChange).toHaveBeenCalledWith(4663);
  });

  it("non-4663 scan does not claim the token is off-pattern (no depth-boundary banner)", async () => {
    renderScan(MOCK_VERIFIED_ADDR, new MockScanClient(), 421614);
    await screen.findByText("wrong network");
    expect(screen.queryByText(/Depth boundary: full verdicts cover/)).toBeNull();
  });

  it("switching the network selector rescans instead of leaving stale cards", async () => {
    const { rerender } = render(
      <ScanPage chainId={4663} client={new MockScanClient()} watchdog={watchdog} />,
    );
    window.location.hash = `#/?addr=${MOCK_VERIFIED_ADDR}`;
    expect(await screen.findByText("VERIFIED")).toBeTruthy();
    rerender(<ScanPage chainId={421614} client={new MockScanClient()} watchdog={watchdog} />);
    expect(await screen.findByText("wrong network")).toBeTruthy();
    expect(screen.getByText(/scanned read-only/)).toBeTruthy();
  });
});
