import type { ViewCountResult } from './types';

/**
 * Returns true if the URL is a TikTok URL.
 */
export function isTikTokUrl(url: string): boolean {
  let parsed: URL;
  try {
    parsed = new URL(url);
  } catch {
    return false;
  }
  const host = parsed.hostname.replace(/^www\./, '');
  return host === 'tiktok.com' || host === 'vm.tiktok.com';
}

/**
 * Extracts the TikTok video ID from a standard TikTok URL.
 * Returns null for short URLs (vm.tiktok.com) or URLs without a video segment.
 */
export function extractTikTokVideoId(url: string): string | null {
  let parsed: URL;
  try {
    parsed = new URL(url);
  } catch {
    return null;
  }

  const host = parsed.hostname.replace(/^www\./, '');
  if (host !== 'tiktok.com') return null;

  // Match /@username/video/<id>
  const match = parsed.pathname.match(/\/@[^/]+\/video\/(\d+)/);
  return match ? match[1] : null;
}

interface OEmbedResponse {
  title?: string;
  author_name?: string;
}

/**
 * Uses TikTok oEmbed to confirm a video exists.
 * Returns 0 views (oEmbed does not expose view counts) and confirms existence.
 * Returns null if the video cannot be confirmed.
 */
export async function getTikTokViews(url: string): Promise<ViewCountResult | null> {
  if (!isTikTokUrl(url)) return null;

  const oEmbedUrl = `https://www.tiktok.com/oembed?url=${encodeURIComponent(url)}`;

  let response: Response;
  try {
    response = await fetch(oEmbedUrl);
  } catch (err) {
    console.error('TikTok oEmbed fetch error:', err);
    return null;
  }

  if (!response.ok) {
    console.error(`TikTok oEmbed error: ${response.status} ${response.statusText}`);
    return null;
  }

  const data: OEmbedResponse = await response.json() as OEmbedResponse;
  // oEmbed does not provide view counts; return 0 to confirm existence only
  if (!data.title && !data.author_name) return null;

  return { viewCount: 0, source: 'tiktok_oembed' };
}
