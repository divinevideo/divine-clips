/**
 * Calculates a fraud score based on view count growth rate.
 *
 * @param currentViews  - The latest verified view count
 * @param previousViews - The view count from the last check (0 for new submissions)
 * @param hoursSinceLastCheck - Hours elapsed since the previous check
 * @returns 0.8 if growth is >10x in <6h, 0.4 if growth is >5x, 0 otherwise
 */
export function calculateFraudScore(
  currentViews: number,
  previousViews: number,
  hoursSinceLastCheck: number,
): number {
  // No previous data or views decreased — no fraud signal
  if (previousViews <= 0 || currentViews <= previousViews) return 0;

  const multiplier = currentViews / previousViews;

  if (multiplier > 10 && hoursSinceLastCheck < 6) {
    return 0.8;
  }

  if (multiplier > 5) {
    return 0.4;
  }

  return 0;
}
