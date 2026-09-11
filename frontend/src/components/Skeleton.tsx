/**
 * Skeleton — advisory primitive, NOT a components.html pattern (verify.md
 * loose end 4): loading placeholder built purely from theme tokens. Expected
 * consumer: step 5's registry loading state.
 */
export function Skeleton({ rows = 3, label = "loading" }: { rows?: number; label?: string }) {
  return (
    <div role="status" aria-label={label}>
      {Array.from({ length: rows }, (_, i) => (
        <div
          key={i}
          style={{
            height: 12,
            borderRadius: "var(--radius)",
            background: "var(--bg-inset)",
            border: "1px solid var(--border)",
            marginBottom: "var(--s2)",
            animation: "vetted-pulse 1.4s ease-in-out infinite",
          }}
        />
      ))}
    </div>
  );
}
