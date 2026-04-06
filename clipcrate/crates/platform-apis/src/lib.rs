// clipcrate-platform-apis: Social platform API clients
// Provides typed HTTP clients for fetching video metadata and engagement metrics
// from social platforms (YouTube, TikTok, Instagram, etc.) for submission verification.

pub mod youtube;
pub mod tiktok;
pub mod instagram;
pub mod twitter;

#[derive(Debug, Clone)]
pub struct ViewCount {
    pub view_count: u64,
    pub source: String,
}
