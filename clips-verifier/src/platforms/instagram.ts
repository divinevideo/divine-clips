import type { ViewCountResult } from './types';

/**
 * Returns true if the URL is an Instagram URL.
 */
export function isInstagramUrl(url: string): boolean {
  let parsed: URL;
  try {
    parsed = new URL(url);
  } catch {
    return false;
  }
  const host = parsed.hostname.replace(/^www\./, '');
  return host === 'instagram.com';
}

/**
 * Extracts the Instagram post/reel shortcode from a URL.
 * Matches patterns like /p/SHORTCODE/, /reel/SHORTCODE/, /reels/SHORTCODE/
 * Returns null if no shortcode can be extracted.
 */
export function extractInstagramShortcode(url: string): string | null {
  let parsed: URL;
  try {
    parsed = new URL(url);
  } catch {
    return null;
  }

  const host = parsed.hostname.replace(/^www\./, '');
  if (host !== 'instagram.com') return null;

  // Match /p/<shortcode>, /reel/<shortcode>, /reels/<shortcode>
  const match = parsed.pathname.match(/\/(?:p|reel|reels)\/([A-Za-z0-9_-]+)/);
  return match ? match[1] : null;
}

interface OEmbedResponse {
  title?: string;
  author_name?: string;
}

/**
 * Uses Instagram oEmbed to confirm a post/reel exists.
 * Returns 0 views — Instagram Graph API requires a business account and app review,
 * which is not available for the MVP. The oEmbed endpoint only confirms existence.
 * Returns null if the post cannot be confirmed.
 */
export async function getInstagramViews(url: string): Promise<ViewCountResult | null> {
  if (!isInstagramUrl(url)) return null;

  const oEmbedUrl = `https://www.instagram.com/api/v1/oembed/?url=${encodeURIComponent(url)}`;

  let response: Response;
  try {
    response = await fetch(oEmbedUrl);
  } catch (err) {
    console.error('Instagram oEmbed fetch error:', err);
    return null;
  }

  if (!response.ok) {
    console.error(`Instagram oEmbed error: ${response.status} ${response.statusText}`);
    return null;
  }

  const data: OEmbedResponse = await response.json() as OEmbedResponse;
  // oEmbed does not provide view counts; return 0 to confirm existence only
  if (!data.title && !data.author_name) return null;

  return { viewCount: 0, source: 'instagram_oembed' };
}
