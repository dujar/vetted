import type { ReactNode } from "react";

/**
 * Card primitive = the theme's `.panel` — components.html has no separate
 * card pattern; the panel is the card (verify.md loose end 4).
 */
export function Panel({
  title,
  children,
  className = "",
}: {
  title?: string;
  children: ReactNode;
  className?: string;
}) {
  return (
    <section className={`panel ${className}`.trim()}>
      {title ? (
        <div className="label" style={{ marginBottom: "var(--s2)" }}>
          {title}
        </div>
      ) : null}
      {children}
    </section>
  );
}
