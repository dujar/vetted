/**
 * Verdict vocabulary + theme mapping. Local mirror of packages/shared
 * (`VERDICTS`); the shared package is the source of truth — step 5 switches
 * the import to vetted-shared when it wires the API client. Kept local so the
 * step-1 scaffold has no cross-package wiring.
 */
export const VERDICTS = ["VERIFIED", "IMPOSTOR", "UNVERIFIED", "REVOKED"] as const;
export type Verdict = (typeof VERDICTS)[number];

/** Theme variant per verdict — the green/red duality; REVOKED is red. */
export type VerdictVariant = "verified" | "risk" | "advisory";

export const VERDICT_VARIANT: Record<Verdict, VerdictVariant> = {
  VERIFIED: "verified",
  IMPOSTOR: "risk",
  UNVERIFIED: "advisory",
  REVOKED: "risk",
};
