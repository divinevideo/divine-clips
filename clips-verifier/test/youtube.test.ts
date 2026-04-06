import { describe, it, expect } from 'vitest';
import { extractYouTubeId } from '../src/platforms/youtube';

describe('extractYouTubeId', () => {
  it('extracts ID from standard watch URL', () => {
    expect(extractYouTubeId('https://www.youtube.com/watch?v=dQw4w9WgXcQ')).toBe('dQw4w9WgXcQ');
  });

  it('extracts ID from youtu.be short URL', () => {
    expect(extractYouTubeId('https://youtu.be/dQw4w9WgXcQ')).toBe('dQw4w9WgXcQ');
  });

  it('extracts ID from embed URL', () => {
    expect(extractYouTubeId('https://www.youtube.com/embed/dQw4w9WgXcQ')).toBe('dQw4w9WgXcQ');
  });

  it('extracts ID from youtu.be short URL with query params', () => {
    expect(extractYouTubeId('https://youtu.be/dQw4w9WgXcQ?t=42')).toBe('dQw4w9WgXcQ');
  });

  it('extracts ID from watch URL with extra params', () => {
    expect(extractYouTubeId('https://www.youtube.com/watch?v=dQw4w9WgXcQ&feature=share')).toBe('dQw4w9WgXcQ');
  });

  it('extracts ID from shorts URL', () => {
    expect(extractYouTubeId('https://www.youtube.com/shorts/dQw4w9WgXcQ')).toBe('dQw4w9WgXcQ');
  });

  it('returns null for non-YouTube URL', () => {
    expect(extractYouTubeId('https://www.tiktok.com/@user/video/12345')).toBeNull();
  });

  it('returns null for malformed URL', () => {
    expect(extractYouTubeId('not-a-url')).toBeNull();
  });

  it('returns null for YouTube URL without video ID', () => {
    expect(extractYouTubeId('https://www.youtube.com/')).toBeNull();
  });

  it('handles youtube.com without www', () => {
    expect(extractYouTubeId('https://youtube.com/watch?v=dQw4w9WgXcQ')).toBe('dQw4w9WgXcQ');
  });
});
