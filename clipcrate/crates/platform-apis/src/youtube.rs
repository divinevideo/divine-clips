use anyhow::{anyhow, Result};
use serde::Deserialize;

use crate::ViewCount;

pub struct YouTubeClient {
    pub api_key: String,
    client: reqwest::Client,
}

impl YouTubeClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_view_count(&self, url: &str) -> Result<ViewCount> {
        let video_id = extract_video_id(url)
            .ok_or_else(|| anyhow!("Could not extract YouTube video ID from URL: {}", url))?;

        let api_url = format!(
            "https://www.googleapis.com/youtube/v3/videos?part=statistics&id={}&key={}",
            video_id, self.api_key
        );

        let response: YouTubeApiResponse = self
            .client
            .get(&api_url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let item = response
            .items
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("No video found with ID: {}", video_id))?;

        let view_count = item
            .statistics
            .view_count
            .parse::<u64>()
            .unwrap_or(0);

        Ok(ViewCount {
            view_count,
            source: "youtube".to_string(),
        })
    }
}

/// Extract a YouTube video ID from a URL.
///
/// Supports:
/// - `https://www.youtube.com/watch?v=VIDEO_ID`
/// - `https://youtu.be/VIDEO_ID`
/// - `https://youtube.com/shorts/VIDEO_ID`
pub fn extract_video_id(url: &str) -> Option<String> {
    let parsed = url::Url::parse(url).ok()?;

    let host = parsed.host_str()?;

    // Normalize host: strip "www." prefix
    let host = host.strip_prefix("www.").unwrap_or(host);

    match host {
        "youtube.com" => {
            let path = parsed.path();

            if path.starts_with("/shorts/") {
                // /shorts/VIDEO_ID
                let id = path.strip_prefix("/shorts/")?.trim_end_matches('/');
                if id.is_empty() {
                    return None;
                }
                return Some(id.to_string());
            }

            if path == "/watch" || path.starts_with("/watch?") || path.starts_with("/watch/") {
                // ?v=VIDEO_ID query param
                for (key, value) in parsed.query_pairs() {
                    if key == "v" {
                        let id = value.into_owned();
                        if id.is_empty() {
                            return None;
                        }
                        return Some(id);
                    }
                }
            }

            None
        }
        "youtu.be" => {
            // Path is /VIDEO_ID
            let id = parsed.path().trim_start_matches('/').trim_end_matches('/');
            if id.is_empty() {
                None
            } else {
                Some(id.to_string())
            }
        }
        _ => None,
    }
}

#[derive(Debug, Deserialize)]
struct YouTubeApiResponse {
    items: Vec<YouTubeVideoItem>,
}

#[derive(Debug, Deserialize)]
struct YouTubeVideoItem {
    statistics: YouTubeStatistics,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YouTubeStatistics {
    view_count: String,
}

#[cfg(test)]
mod tests {
    use super::extract_video_id;

    #[test]
    fn test_watch_url() {
        assert_eq!(
            extract_video_id("https://www.youtube.com/watch?v=abc123"),
            Some("abc123".to_string())
        );
    }

    #[test]
    fn test_short_url() {
        assert_eq!(
            extract_video_id("https://youtu.be/abc123"),
            Some("abc123".to_string())
        );
    }

    #[test]
    fn test_shorts_url() {
        assert_eq!(
            extract_video_id("https://youtube.com/shorts/abc123"),
            Some("abc123".to_string())
        );
    }

    #[test]
    fn test_shorts_url_with_www() {
        assert_eq!(
            extract_video_id("https://www.youtube.com/shorts/abc123"),
            Some("abc123".to_string())
        );
    }

    #[test]
    fn test_non_youtube_url_returns_none() {
        assert_eq!(extract_video_id("https://tiktok.com/something"), None);
    }

    #[test]
    fn test_watch_url_without_www() {
        assert_eq!(
            extract_video_id("https://youtube.com/watch?v=dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn test_watch_url_with_extra_params() {
        assert_eq!(
            extract_video_id("https://www.youtube.com/watch?v=abc123&t=42s&list=PLfoo"),
            Some("abc123".to_string())
        );
    }

    #[test]
    fn test_youtu_be_with_query_params() {
        assert_eq!(
            extract_video_id("https://youtu.be/abc123?t=42"),
            Some("abc123".to_string())
        );
    }

    #[test]
    fn test_empty_video_id_in_watch_returns_none() {
        assert_eq!(
            extract_video_id("https://www.youtube.com/watch?v="),
            None
        );
    }

    #[test]
    fn test_youtu_be_no_path_returns_none() {
        assert_eq!(extract_video_id("https://youtu.be/"), None);
    }

    #[test]
    fn test_instagram_url_returns_none() {
        assert_eq!(extract_video_id("https://www.instagram.com/reel/abc123/"), None);
    }

    #[test]
    fn test_invalid_url_returns_none() {
        assert_eq!(extract_video_id("not-a-url"), None);
    }

    #[test]
    fn test_shorts_empty_id_returns_none() {
        assert_eq!(extract_video_id("https://www.youtube.com/shorts/"), None);
    }
}
