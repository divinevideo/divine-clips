import type { ViewCountResult } from './platforms/types';

/**
 * Phyllo fallback client — stub for MVP.
 *
 * In production this would call the Phyllo API to retrieve
 * creator analytics for platforms where direct API access is unavailable.
 */
export async function getPhylloViews(
  _url: string,
  _platform: string,
): Promise<ViewCountResult | null> {
  // TODO: Implement Phyllo API integration in a future milestone
  console.log('[phyllo] Phyllo fallback is a stub in MVP — returning null');
  return null;
}
