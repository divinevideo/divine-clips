import type { ViewCountResult } from './types';

/**
 * Returns true if the URL is an X (formerly Twitter) URL.
 */
export function isXUrl(url: string): boolean {
  let parsed: URL;
  try {
    parsed = new URL(url);
  } catch {
    return false;
  }
  const host = parsed.hostname.replace(/^www\./, '');
  return host === 'x.com' || host === 'twitter.com';
}

/**
 * Extracts the tweet/post ID from an X or Twitter URL.
 * Matches patterns like /<username>/status/<id>
 * Returns null if no ID can be extracted.
 */
export function extractXPostId(url: string): string | null {
  let parsed: URL;
  try {
    parsed = new URL(url);
  } catch {
    return null;
  }

  const host = parsed.hostname.replace(/^www\./, '');
  if (host !== 'x.com' && host !== 'twitter.com') return null;

  // Match /<username>/status/<id>
  const match = parsed.pathname.match(/\/[^/]+\/status\/(\d+)/);
  return match ? match[1] : null;
}

/**
 * Validates that an X/Twitter URL exists by checking URL structure.
 * Returns 0 views — the X API v2 requires a paid Basic tier ($100/month)
 * to access tweet metrics. For the MVP, we only validate the URL format
 * and confirm it matches the expected pattern for a tweet/post.
 * Returns null if the URL does not match expected patterns.
 */
export async function getXViews(url: string): Promise<ViewCountResult | null> {
  if (!isXUrl(url)) return null;

  // Verify the URL has a valid post ID structure
  const postId = extractXPostId(url);
  if (!postId) return null;

  // No API call — X API v2 requires paid tier for metrics.
  // URL structure validation is sufficient for MVP.
  return { viewCount: 0, source: 'x_url_validation' };
}
