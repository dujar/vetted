/**
 * App shell (verify.md loose end 2 fix) — the mockups' shared header.bar
 * (wordmark + Scan/Swap/Registry nav + chain chip) and the three routes.
 * Routing is a dependency-free hash router (lib/router.ts) so deep links
 * survive static hosting. Deliberate step-5 edit of the step-1 shell.
 */
import { useState } from "react";

import { Chip } from "./components/Chip";
import { useWalletAdapter } from "./lib/wallet";
import { ALL_CHAINS, DEFAULT_CHAIN } from "./lib/chains";
import { useHashRoute } from "./lib/router";
import { PRODUCT_WORDMARK } from "./lib/brand";
import { ScanPage } from "./pages/ScanPage";
import { SwapPage } from "./pages/SwapPage";
import { RegistryPage } from "./pages/RegistryPage";

export default function App() {
  const route = useHashRoute();
  const wallet = useWalletAdapter();
  const [chainId, setChainId] = useState<number>(DEFAULT_CHAIN.id);

  const nav = (
    <nav style={{ display: "flex", gap: "var(--s4)", fontSize: "var(--fs-1)" }}>
      <a href="#/">Scan</a>
      <a href="#/swap">Swap</a>
      <a href="#/registry">Registry</a>
    </nav>
  );

  return (
    <div className="wrap">
      <header
        className="bar"
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "var(--s4) 0",
          borderBottom: "1px solid var(--border)",
          marginBottom: "var(--s6)",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: "var(--s3)" }}>
          <span className="mono" style={{ fontSize: "var(--fs-3)", fontWeight: 700 }}>
            {PRODUCT_WORDMARK}
          </span>
          {nav}
        </div>
        {route.path === "/" ? (
          <select
            className="chip mono"
            value={chainId}
            onChange={(e) => setChainId(Number(e.target.value))}
            aria-label="network selector"
          >
            {ALL_CHAINS.map((chain) => (
              <option key={chain.id} value={chain.id}>
                ● {chain.name} · {chain.id}
              </option>
            ))}
          </select>
        ) : (
          <Chip>● Robinhood Chain · 4663</Chip>
        )}
      </header>

      {route.path === "/swap" ? (
        <SwapPage wallet={wallet} />
      ) : route.path === "/registry" ? (
        <RegistryPage />
      ) : (
        <ScanPage chainId={chainId} onChainChange={setChainId} />
      )}
    </div>
  );
}
