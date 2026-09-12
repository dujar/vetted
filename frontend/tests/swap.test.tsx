/**
 * Swap screen state coverage (J2) — one test per screens/swap.html state:
 * connect (incl. zero-connector graceful prompt), wrong-network before
 * quoting, pre-flight guard checks, confirming, settled, refused (verbatim
 * reasons), nothing-sent (rejected / gas).
 */
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { SwapPage } from "../src/pages/SwapPage";
import type { WalletAdapter } from "../src/lib/wallet";
import {
  createMockGuardExecutor,
  MockGuardProbeSource,
  type GuardExecutionResult,
} from "../src/lib/guard";
import {
  MOCK_IMPL_MISMATCH_ADDR,
  MOCK_REVOKED_ADDR,
  MOCK_VERIFIED_ADDR,
} from "../src/lib/mockData";

const BUYER = `0x${"aa".repeat(20)}`;

function fakeWallet(overrides: Partial<WalletAdapter> = {}): WalletAdapter {
  return {
    status: "connected",
    address: BUYER,
    chainId: 4663,
    connectorNames: ["WalletConnect"],
    connect: async () => {},
    switchToVetted: async () => {},
    // Mock-mode wiring, same as production's env switch (lib/wallet.ts):
    // the executor derives its outcome from the same preview the panel shows.
    execute: createMockGuardExecutor(new MockGuardProbeSource()),
    ...overrides,
  };
}

function setReceiveToken(value: string) {
  fireEvent.change(screen.getByLabelText("receive token address"), { target: { value } });
}

function renderSwap(wallet: WalletAdapter) {
  window.location.hash = "#/swap";
  return render(<SwapPage wallet={wallet} />);
}

afterEach(() => {
  window.location.hash = "";
});

describe("connect states", () => {
  it("not connected: mockup panel + connect button", async () => {
    const connect = vi.fn();
    renderSwap(fakeWallet({ status: "disconnected", address: undefined, chainId: undefined, connect }));
    expect(screen.getByText("Guarded swap")).toBeTruthy();
    expect(screen.getByText(/your funds never move on a revert/)).toBeTruthy();
    fireEvent.click(screen.getByText("Connect wallet"));
    await waitFor(() => expect(connect).toHaveBeenCalled());
  });

  it("zero connectors: prompts for the operator projectId instead of crashing", () => {
    renderSwap(fakeWallet({ status: "disconnected", connectorNames: [] }));
    const alert = screen.getByRole("alert");
    expect(alert.textContent).toContain("VITE_WALLETCONNECT_PROJECT_ID");
    const btn = screen.getByText("Connect wallet") as HTMLButtonElement;
    expect(btn.disabled).toBe(true);
  });
});

describe("wrong network — before quoting (journeys.md:34)", () => {
  it("shows the switch prompt and no guard checks", () => {
    const switchToVetted = vi.fn();
    renderSwap(fakeWallet({ chainId: 421614, switchToVetted }));
    expect(screen.getByText(/switch to Robinhood Chain · 4663 to quote/)).toBeTruthy();
    expect(screen.getByText("Switch to Robinhood Chain · 4663")).toBeTruthy();
    expect(screen.queryByText(/guard checks/)).toBeNull();
    fireEvent.click(screen.getByText("Switch to Robinhood Chain · 4663"));
    expect(switchToVetted).toHaveBeenCalled();
  });
});

describe("pre-flight preview (journeys.md:31-32)", () => {
  it("shows the four checks the contract re-runs at execution", async () => {
    renderSwap(fakeWallet());
    setReceiveToken(MOCK_VERIFIED_ADDR);
    expect(await screen.findByText("verified 2026-08-30")).toBeTruthy();
    expect(screen.getByText("Live verification record in Canonical Registry")).toBeTruthy();
    expect(screen.getByText("Token not paused")).toBeTruthy();
    expect(screen.getByText("Buyer not blocklisted")).toBeTruthy();
    expect(screen.getByText("Implementation matches registry record")).toBeTruthy();
    expect(screen.getByText("Swap guarded")).toBeTruthy();
  });

  it("previews the verbatim refusal for a red-flagged token but leaves the guard in charge", async () => {
    renderSwap(fakeWallet());
    setReceiveToken(MOCK_IMPL_MISMATCH_ADDR);
    expect(await screen.findByRole("alert")).toBeTruthy();
    expect(screen.getByText(/the guard will refuse this swap on-chain: GUARD_IMPL_MISMATCH/)).toBeTruthy();
    const btn = screen.getByText("Swap guarded") as HTMLButtonElement;
    expect(btn.disabled).toBe(false);
  });

  it("hides the checks until a receive address is entered", () => {
    renderSwap(fakeWallet());
    expect(screen.getByText(/Enter a receive token address/)).toBeTruthy();
    expect((screen.getByText("Swap guarded") as HTMLButtonElement).disabled).toBe(true);
  });
});

describe("execution outcomes", () => {
  it("confirming: broadcast panel while the receipt is pending", async () => {
    renderSwap(
      fakeWallet({
        execute: () => new Promise<GuardExecutionResult>(() => {}),
      }),
    );
    setReceiveToken(MOCK_VERIFIED_ADDR);
    await screen.findByText("verified 2026-08-30");
    fireEvent.click(screen.getByText("Swap guarded"));
    expect(screen.getByText("Confirming…")).toBeTruthy();
    expect(screen.getByText(/waiting for receipt/)).toBeTruthy();
    expect(screen.getByText(/a red flag reverts it atomically/)).toBeTruthy();
  });

  it("settled: green receipt with guard summary", async () => {
    renderSwap(fakeWallet());
    setReceiveToken(MOCK_VERIFIED_ADDR);
    await screen.findByText("verified 2026-08-30");
    fireEvent.click(screen.getByText("Swap guarded"));
    expect(await screen.findByText("Settled")).toBeTruthy();
    expect(screen.getByText("All guard checks passed at execution")).toBeTruthy();
    expect(screen.getByText(/0\.98 NVDA/)).toBeTruthy();
    expect(screen.queryByText(/never left your wallet/i)).toBeNull();
  });

  it("refused GUARD_IMPL_MISMATCH: verbatim banner + funds-never-moved", async () => {
    renderSwap(fakeWallet());
    setReceiveToken(MOCK_IMPL_MISMATCH_ADDR);
    await screen.findByRole("alert");
    fireEvent.click(screen.getByText("Swap guarded"));
    expect(await screen.findByText("Swap refused at execution")).toBeTruthy();
    expect(screen.getByText(/GUARD_IMPL_MISMATCH: .*funds never moved/)).toBeTruthy();
    expect(screen.getByText(/never left your wallet/)).toBeTruthy();
  });

  it("refused GUARD_RECORD_REVOKED: the upgrade beat (verified before, refused now)", async () => {
    renderSwap(fakeWallet());
    setReceiveToken(MOCK_REVOKED_ADDR);
    await screen.findByRole("alert");
    fireEvent.click(screen.getByText("Swap guarded"));
    expect(await screen.findByText("Swap refused — verification revoked")).toBeTruthy();
    expect(screen.getByText(/GUARD_RECORD_REVOKED:/)).toBeTruthy();
    expect(screen.getByText(/A static docs page cannot do this/)).toBeTruthy();
  });

  it("rejected in wallet: nothing was sent", async () => {
    renderSwap(
      fakeWallet({
        execute: async () => {
          throw Object.assign(new Error("denied"), { name: "WalletRequestRejectedError" });
        },
      }),
    );
    setReceiveToken(MOCK_VERIFIED_ADDR);
    await screen.findByText("verified 2026-08-30");
    fireEvent.click(screen.getByText("Swap guarded"));
    expect(await screen.findByText("Nothing was sent")).toBeTruthy();
    expect(screen.getByText("Rejected in wallet")).toBeTruthy();
    expect(screen.getByText(/never left your wallet/)).toBeTruthy();
    expect(screen.getByText("Retry when ready — the guard re-runs every check at execution.")).toBeTruthy();
  });

  it("gas failure: nothing was sent, retry offered", async () => {
    renderSwap(
      fakeWallet({
        execute: async () => {
          throw Object.assign(new Error("out of gas"), { name: "TransactionExecutionError" });
        },
      }),
    );
    setReceiveToken(MOCK_VERIFIED_ADDR);
    await screen.findByText("verified 2026-08-30");
    fireEvent.click(screen.getByText("Swap guarded"));
    expect(await screen.findByText("Nothing was sent")).toBeTruthy();
    expect(screen.getByText("Gas too low")).toBeTruthy();
    fireEvent.click(screen.getByText("Swap guarded"));
    expect(screen.getByText(/you receive — any token address accepted/)).toBeTruthy();
  });
});
