import { afterEach, describe, expect, it } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import { VerdictBanner } from "../src/components/VerdictBanner";
import { VERDICTS, VERDICT_VARIANT } from "../src/components/verdicts";

describe("VerdictBanner", () => {
  afterEach(cleanup);

  it("renders all four verdicts with their theme variant", () => {
    render(
      <div>
        {VERDICTS.map((v) => (
          <VerdictBanner key={v} verdict={v} />
        ))}
      </div>,
    );
    for (const verdict of VERDICTS) {
      const el = screen.getByText(verdict).closest(".banner");
      expect(el).not.toBeNull();
      expect(el!.className).toBe(`banner ${VERDICT_VARIANT[verdict]}`);
    }
  });

  it("renders REVOKED with the risk (red) variant — formerly verified, revoked since", () => {
    render(<VerdictBanner verdict="REVOKED" />);
    expect(screen.getByText("REVOKED").closest(".banner")!.className).toBe("banner risk");
  });

  it("keeps UNVERIFIED advisory (amber) — the engine never guesses", () => {
    render(<VerdictBanner verdict="UNVERIFIED" />);
    expect(screen.getByText("UNVERIFIED").closest(".banner")!.className).toBe("banner advisory");
  });
});
