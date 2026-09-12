/**
 * Verdict vocabulary + theme mapping. The union now comes from
 * `vetted-shared` (step-1 findings: the local mirror existed only until step 5
 * wired the API client — that switch point is here; the mirror is deleted).
 * `VERDICT_VARIANT` stays local: it maps the shared union onto this repo's
 * theme tokens (green/red duality; REVOKED renders red).
 */
import type { Verdict } from "vetted-shared";

export { VERDICTS } from "vetted-shared";
export type { Verdict };

/** Theme variant per verdict — the green/red duality; REVOKED is red. */
export type VerdictVariant = "verified" | "risk" | "advisory";

export const VERDICT_VARIANT: Record<Verdict, VerdictVariant> = {
  VERIFIED: "verified",
  IMPOSTOR: "risk",
  UNVERIFIED: "advisory",
  REVOKED: "risk",
};
