/**
 * Registry screen state coverage (J3) — populated table incl. the REVOKED red
 * row with drill-in evidence, loading skeleton, degraded, empty, and the
 * criteria panel (journeys.md:42-45).
 */
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { SELECTORS } from "vetted-shared";

import { RegistryPage } from "../src/pages/RegistryPage";
import { MockRegistrySource, type RegistrySource } from "../src/lib/registry";

const NEVER: RegistrySource = { rows: () => new Promise(() => {}) };
const THROWS: RegistrySource = { rows: async () => { throw new Error("rpc down"); } };
const EMPTY: RegistrySource = { rows: async () => [] };

describe("populated table (mockup rows)", () => {
  it("lists token, symbol, status, implementation, verifiedAt, registrar + scan links", async () => {
    render(<RegistryPage source={new MockRegistrySource()} />);
    expect(await screen.findByText("NVIDIA")).toBeTruthy();
    expect(screen.getByText("AMC Entertainment")).toBeTruthy();
    expect(screen.getByText("SpaceX")).toBeTruthy();
    const badges = screen.getAllByText("VERIFIED");
    expect(badges.length).toBe(3);
    expect(screen.getAllByText("0x5e15…aa11").length).toBe(4);
    expect(screen.getAllByText("scan").length).toBe(4);
    expect((screen.getAllByText("scan")[0] as HTMLAnchorElement).getAttribute("href")).toContain("addr=");
  });

  it("REVOKED row renders red with reason + drill-in revocation-tx evidence", async () => {
    render(<RegistryPage source={new MockRegistrySource()} />);
    expect(await screen.findByText("REVOKED")).toBeTruthy();
    expect(screen.getByText(/beacon impl changed 2026-09-06 — record stale/)).toBeTruthy();
    // drill-in closed by default…
    expect(screen.queryByText(/revocation tx/)).toBeNull();
    // …click evidence to open it
    fireEvent.click(screen.getByText("evidence"));
    expect(screen.getByText(/revocation tx/)).toBeTruthy();
    const tx = screen.getByText("0xf4c0…71b8");
    expect((tx as HTMLAnchorElement).href).toContain("/tx/0xf4c0");
    // second click closes it again
    fireEvent.click(screen.getByText("evidence"));
    expect(screen.queryByText(/revocation tx/)).toBeNull();
  });
});

describe("criteria panel (journeys.md:43)", () => {
  it("shows registrar, criteria.md link, and the on-chain read interface", async () => {
    render(<RegistryPage source={new MockRegistrySource()} />);
    await screen.findByText("NVIDIA");
    expect(screen.getByText(/registrar: 0x5e15…aa11/)).toBeTruthy();
    const criteria = screen.getByText("Verification criteria, published in the repo.") as HTMLAnchorElement;
    expect(criteria.href).toContain("docs/criteria.md");
    expect(screen.getByText(/getRecord\(address token\)/)).toBeTruthy();
    expect(screen.getByText(`selector ${SELECTORS.getRecord}`)).toBeTruthy();
    expect(screen.getByText("watchlists · alerts · partner integrations")).toBeTruthy();
    expect(screen.getByText(/the registry is a/)).toBeTruthy();
  });
});

describe("loading / degraded / empty states (journeys.md:45)", () => {
  it("loading: skeleton + reading caption", () => {
    render(<RegistryPage source={NEVER} />);
    expect(screen.getByText("Reading verification records from the Canonical Registry…")).toBeTruthy();
    expect(screen.getByRole("status", { name: "loading registry records" })).toBeTruthy();
  });

  it("degraded: advisory banner, records untouched", async () => {
    render(<RegistryPage source={THROWS} />);
    expect(
      await screen.findByText(/Issuer canonical list unreachable — new verifications are paused/),
    ).toBeTruthy();
    expect(screen.getByText("Retry")).toBeTruthy();
  });

  it("empty: no records yet + scan pointer", async () => {
    render(<RegistryPage source={EMPTY} />);
    expect(await screen.findByText("No verification records yet on this chain.")).toBeTruthy();
    expect(screen.getByText("Scan an address in the meantime →")).toBeTruthy();
  });

  it("degraded retry recovers when the source comes back", async () => {
    let fail = true;
    const flaky: RegistrySource = {
      rows: async () => {
        if (fail) throw new Error("rpc down");
        return new MockRegistrySource().rows();
      },
    };
    render(<RegistryPage source={flaky} />);
    await screen.findByText("Retry");
    fail = false; // the source recovers before the retry fires
    fireEvent.click(screen.getByText("Retry"));
    await screen.findByText("NVIDIA");
  });
});
