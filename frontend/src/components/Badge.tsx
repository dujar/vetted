import { VERDICT_VARIANT, type Verdict } from "./verdicts";

/** Verdict badge — `.badge` with the theme's three variants; REVOKED renders red (risk). */
const BADGE_CLASS = {
  verified: "badge verified",
  risk: "badge impostor",
  advisory: "badge unverified",
} as const;

export function Badge({ verdict }: { verdict: Verdict }) {
  return <span className={BADGE_CLASS[VERDICT_VARIANT[verdict]]}>{verdict}</span>;
}
