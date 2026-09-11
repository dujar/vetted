import type { ReactNode } from "react";

/** Network/status chip — `.chip`; advisory tone for non-default networks. */
export function Chip({
  tone = "default",
  children,
}: {
  tone?: "default" | "advisory";
  children: ReactNode;
}) {
  return (
    <span className="chip mono" style={tone === "advisory" ? { color: "var(--advisory)" } : undefined}>
      {children}
    </span>
  );
}
