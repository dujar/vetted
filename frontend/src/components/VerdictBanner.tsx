import type { ReactNode } from "react";
import { VERDICT_VARIANT, type Verdict } from "./verdicts";

/** Full-width verdict banner — `.banner` with the theme's three variants. */
const BANNER_CLASS = {
  verified: "banner verified",
  risk: "banner risk",
  advisory: "banner advisory",
} as const;

export function VerdictBanner({ verdict, children }: { verdict: Verdict; children?: ReactNode }) {
  return (
    <div className={BANNER_CLASS[VERDICT_VARIANT[verdict]]} role="status">
      <strong className="mono">{verdict}</strong>
      {children ? <div>{children}</div> : null}
    </div>
  );
}
