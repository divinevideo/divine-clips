import { describe, it, expect } from 'vitest';
import { isInstagramUrl, extractInstagramShortcode } from '../src/platforms/instagram';

describe('isInstagramUrl', () => {
  it('returns true for standard Instagram URL', () => {
    expect(isInstagramUrl('https://www.instagram.com/p/ABC123/')).toBe(true);
  });

  it('returns true for Instagram URL without www', () => {
    expect(isInstagramUrl('https://instagram.com/reel/ABC123/')).toBe(true);
  });

  it('returns false for YouTube URL', () => {
    expect(isInstagramUrl('https://www.youtube.com/watch?v=dQw4w9WgXcQ')).toBe(false);
  });

  it('returns false for non-URL string', () => {
    expect(isInstagramUrl('not-a-url')).toBe(false);
  });
});

describe('extractInstagramShortcode', () => {
  it('extracts shortcode from /p/ URL', () => {
    expect(extractInstagramShortcode('https://www.instagram.com/p/CxYz1234/')).toBe('CxYz1234');
  });

  it('extracts shortcode from /reel/ URL', () => {
    expect(extractInstagramShortcode('https://www.instagram.com/reel/CxYz1234/')).toBe('CxYz1234');
  });

  it('extracts shortcode from /reels/ URL', () => {
    expect(extractInstagramShortcode('https://instagram.com/reels/CxYz1234/')).toBe('CxYz1234');
  });

  it('returns null for profile URL without post', () => {
    expect(extractInstagramShortcode('https://www.instagram.com/username/')).toBeNull();
  });

  it('returns null for non-Instagram URL', () => {
    expect(extractInstagramShortcode('https://www.tiktok.com/@user/video/123')).toBeNull();
  });

  it('returns null for invalid URL', () => {
    expect(extractInstagramShortcode('not-a-url')).toBeNull();
  });
});
