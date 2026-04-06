import { describe, it, expect } from 'vitest';
import { isXUrl, extractXPostId, getXViews } from '../src/platforms/x';

describe('isXUrl', () => {
  it('returns true for x.com URL', () => {
    expect(isXUrl('https://x.com/user/status/1234567890')).toBe(true);
  });

  it('returns true for twitter.com URL', () => {
    expect(isXUrl('https://twitter.com/user/status/1234567890')).toBe(true);
  });

  it('returns true for www.x.com URL', () => {
    expect(isXUrl('https://www.x.com/user/status/1234567890')).toBe(true);
  });

  it('returns false for YouTube URL', () => {
    expect(isXUrl('https://www.youtube.com/watch?v=dQw4w9WgXcQ')).toBe(false);
  });

  it('returns false for non-URL string', () => {
    expect(isXUrl('not-a-url')).toBe(false);
  });
});

describe('extractXPostId', () => {
  it('extracts post ID from x.com URL', () => {
    expect(extractXPostId('https://x.com/elonmusk/status/1234567890123456789')).toBe('1234567890123456789');
  });

  it('extracts post ID from twitter.com URL', () => {
    expect(extractXPostId('https://twitter.com/jack/status/9876543210')).toBe('9876543210');
  });

  it('returns null for profile URL without status', () => {
    expect(extractXPostId('https://x.com/elonmusk')).toBeNull();
  });

  it('returns null for non-X URL', () => {
    expect(extractXPostId('https://www.youtube.com/watch?v=dQw4w9WgXcQ')).toBeNull();
  });

  it('returns null for invalid URL', () => {
    expect(extractXPostId('not-a-url')).toBeNull();
  });
});

describe('getXViews', () => {
  it('returns view count result for valid post URL', async () => {
    const result = await getXViews('https://x.com/user/status/1234567890');
    expect(result).toEqual({ viewCount: 0, source: 'x_url_validation' });
  });

  it('returns null for URL without post ID', async () => {
    const result = await getXViews('https://x.com/user');
    expect(result).toBeNull();
  });

  it('returns null for non-X URL', async () => {
    const result = await getXViews('https://www.youtube.com/watch?v=abc');
    expect(result).toBeNull();
  });
});
