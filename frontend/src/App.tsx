import { Chip } from "./components/Chip";
import { Panel } from "./components/Panel";

/** Scaffold shell — NOT a screen. Screens arrive in step 5 from the mockups. */
export default function App() {
  return (
    <div className="wrap">
      <header
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "var(--s4) 0",
          borderBottom: "1px solid var(--border)",
        }}
      >
        <span className="mono" style={{ fontSize: "var(--fs-3)", fontWeight: 700 }}>
          vetted_
        </span>
        <Chip>● Robinhood Chain · 4663</Chip>
      </header>
      <Panel title="scaffold">
        <p className="muted" style={{ margin: 0 }}>
          Monorepo scaffold ready — theme tokens, primitives, chains and wallet
          wiring are in place. Screens arrive in step 5.
        </p>
      </Panel>
    </div>
  );
}
