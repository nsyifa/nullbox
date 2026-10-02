export type AgingLevel = "fresh" | "yellowed" | "cobwebs" | "fossil";

const HOUR = 3600 * 1000;

/** How old an open ticket looks. Thresholds are presentation-only; tweak freely. */
export function agingLevel(createdAt: string, now = Date.now()): AgingLevel {
  const age = now - Date.parse(createdAt);
  if (age < 24 * HOUR) return "fresh";
  if (age < 3 * 24 * HOUR) return "yellowed";
  if (age < 7 * 24 * HOUR) return "cobwebs";
  return "fossil";
}
