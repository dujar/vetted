/**
 * App shell smoke — real wagmi config (zero connectors in CI: no
 * VITE_WALLETCONNECT_PROJECT_ID, and the test must NOT set it — verify.md),
 * hash routing between the three screens, header nav per the mockups.
 */
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { WagmiProvider } from "wagmi";

import App from "../src/App";
import { getWagmiConfig } from "../src/lib/wagmi";

function renderApp() {
  window.location.hash = "#/";
  const client = new QueryClient();
  return render(
    <QueryClientProvider client={client}>
      <WagmiProvider config={getWagmiConfig()}>
        <App />
      </WagmiProvider>
    </QueryClientProvider>,
  );
}

afterEach(() => {
  window.location.hash = "";
});

describe("shell", () => {
  it("renders the mockup header: wordmark, nav, chain chip", () => {
    renderApp();
    expect(screen.getByText("vetted_")).toBeTruthy();
    // "Scan" appears in the nav and on the hero button
    expect(screen.getAllByText("Scan").length).toBeGreaterThanOrEqual(2);
    expect(screen.getByText("Swap")).toBeTruthy();
    expect(screen.getByText("Registry")).toBeTruthy();
    // network selector is a UI control defaulting to 4663 (journeys.md:12)
    const select = screen.getByLabelText("network selector") as HTMLSelectElement;
    expect(select.value).toBe("4663");
    expect(screen.getByText(/Robinhood Chain Testnet · 46630/)).toBeTruthy();
  });

  it("routes / to the scan hero", () => {
    renderApp();
    expect(screen.getByText("Is this stock token real — and what can it do to you?")).toBeTruthy();
  });

  it("routes #/swap to the swap screen via real wagmi (zero-connector safe)", async () => {
    renderApp();
    window.location.hash = "#/swap"; // jsdom fires hashchange on its own task — findBy* waits for the rerender
    await screen.findByText("Guarded swap");
    // no projectId in the test env → graceful prompt, not a crash (step-1 findings)
    expect(screen.getByText(/VITE_WALLETCONNECT_PROJECT_ID/)).toBeTruthy();
  });

  it("routes #/registry to the registry screen (mock source by default)", async () => {
    renderApp();
    window.location.hash = "#/registry";
    await screen.findByText("Canonical Registry");
  });

  it("network selector drives the scan page chain state", () => {
    renderApp();
    const select = screen.getByLabelText("network selector") as HTMLSelectElement;
    fireEvent.change(select, { target: { value: "421614" } });
    expect(select.value).toBe("421614");
    expect(screen.getByText(/Arbitrum Sepolia · 421614/)).toBeTruthy();
  });
});
