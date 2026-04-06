import type { ViewCountResult } from './types';

/**
 * Extracts the YouTube video ID from various YouTube URL formats.
 * Returns null if the URL is not a valid YouTube URL or has no video ID.
 */
export function extractYouTubeId(url: string): string | null {
  let parsed: URL;
  try {
    parsed = new URL(url);
  } catch {
    return null;
  }

  const { hostname, pathname, searchParams } = parsed;

  // Normalize hostname
  const host = hostname.replace(/^www\./, '');

  if (host === 'youtu.be') {
    // https://youtu.be/<id>
    const id = pathname.slice(1).split('?')[0];
    return id.length > 0 ? id : null;
  }

  if (host === 'youtube.com') {
    // https://youtube.com/watch?v=<id>
    const v = searchParams.get('v');
    if (v && v.length > 0) return v;

    // https://youtube.com/embed/<id>
    // https://youtube.com/shorts/<id>
    const embedMatch = pathname.match(/^\/(embed|shorts|v)\/([^/?#]+)/);
    if (embedMatch && embedMatch[2].length > 0) return embedMatch[2];

    return null;
  }

  return null;
}

interface YouTubeApiResponse {
  items?: Array<{
    statistics?: {
      viewCount?: string;
    };
  }>;
}

/**
 * Fetches view count for a YouTube video using the YouTube Data API v3.
 */
export async function getYouTubeViews(url: string, apiKey: string): Promise<ViewCountResult | null> {
  const videoId = extractYouTubeId(url);
  if (!videoId) return null;

  const apiUrl = `https://www.googleapis.com/youtube/v3/videos?part=statistics&id=${encodeURIComponent(videoId)}&key=${encodeURIComponent(apiKey)}`;

  const response = await fetch(apiUrl);
  if (!response.ok) {
    console.error(`YouTube API error: ${response.status} ${response.statusText}`);
    return null;
  }

  const data: YouTubeApiResponse = await response.json() as YouTubeApiResponse;
  const item = data.items?.[0];
  if (!item) return null;

  const viewCount = parseInt(item.statistics?.viewCount ?? '0', 10);
  return { viewCount, source: 'youtube_api_v3' };
}
