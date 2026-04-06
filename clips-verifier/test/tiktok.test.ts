import { describe, it, expect } from 'vitest';
import { extractTikTokVideoId, isTikTokUrl } from '../src/platforms/tiktok';

describe('isTikTokUrl', () => {
  it('returns true for standard TikTok URL', () => {
    expect(isTikTokUrl('https://www.tiktok.com/@username/video/1234567890123456789')).toBe(true);
  });

  it('returns true for TikTok URL without www', () => {
    expect(isTikTokUrl('https://tiktok.com/@username/video/1234567890123456789')).toBe(true);
  });

  it('returns true for vm.tiktok.com short URL', () => {
    expect(isTikTokUrl('https://vm.tiktok.com/ZMeABCDEF/')).toBe(true);
  });

  it('returns false for YouTube URL', () => {
    expect(isTikTokUrl('https://www.youtube.com/watch?v=dQw4w9WgXcQ')).toBe(false);
  });

  it('returns false for non-URL string', () => {
    expect(isTikTokUrl('not-a-url')).toBe(false);
  });
});

describe('extractTikTokVideoId', () => {
  it('extracts video ID from standard TikTok URL', () => {
    expect(extractTikTokVideoId('https://www.tiktok.com/@username/video/1234567890123456789')).toBe('1234567890123456789');
  });

  it('extracts video ID from TikTok URL without www', () => {
    expect(extractTikTokVideoId('https://tiktok.com/@username/video/9876543210987654321')).toBe('9876543210987654321');
  });

  it('returns null for vm.tiktok.com short URL (no extractable ID)', () => {
    expect(extractTikTokVideoId('https://vm.tiktok.com/ZMeABCDEF/')).toBeNull();
  });

  it('returns null for non-TikTok URL', () => {
    expect(extractTikTokVideoId('https://www.youtube.com/watch?v=dQw4w9WgXcQ')).toBeNull();
  });

  it('returns null for TikTok URL without video segment', () => {
    expect(extractTikTokVideoId('https://www.tiktok.com/@username')).toBeNull();
  });
});
