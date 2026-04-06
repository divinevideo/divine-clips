use anyhow::{anyhow, Result};
use serde::Deserialize;

use crate::ViewCount;

pub struct TikTokClient {
    client: reqwest::Client,
}

impl TikTokClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Check video existence via oEmbed. Returns 0 views — TikTok's oEmbed
    /// does not expose view counts, so Phyllo is used for full metrics.
    pub async fn get_view_count(&self, url: &str) -> Result<ViewCount> {
        let _video_id = extract_video_id(url)
            .ok_or_else(|| anyhow!("Could not extract TikTok video ID from URL: {}", url))?;

        let oembed_url = format!(
            "https://www.tiktok.com/oembed?url={}",
            urlencoding::encode(url)
        );

        let response: TikTokOEmbedResponse = self
            .client
            .get(&oembed_url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        tracing::debug!(
            author = %response.author_name,
            title = %response.title,
            "TikTok oEmbed confirmed video exists"
        );

        // oEmbed confirms existence but does not provide view counts
        Ok(ViewCount {
            view_count: 0,
            source: "tiktok".to_string(),
        })
    }
}

impl Default for TikTokClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract a TikTok video ID from a URL.
///
/// Supports:
/// - `https://www.tiktok.com/@username/video/1234567890`
/// - `https://tiktok.com/@username/video/1234567890`
/// - `https://vm.tiktok.com/SHORTCODE` (short links — only validates, no resolution)
pub fn extract_video_id(url: &str) -> Option<String> {
    let parsed = url::Url::parse(url).ok()?;

    let host = parsed.host_str()?;
    let host = host.strip_prefix("www.").unwrap_or(host);

    match host {
        "tiktok.com" => {
            // Path: /@username/video/VIDEO_ID
            let path = parsed.path();
            let segments: Vec<&str> = path
                .split('/')
                .filter(|s| !s.is_empty())
                .collect();

            // Expected: ["@username", "video", "VIDEO_ID"]
            if segments.len() >= 3 && segments[1] == "video" {
                let id = segments[2];
                if !id.is_empty() {
                    return Some(id.to_string());
                }
            }

            None
        }
        "vm.tiktok.com" | "vt.tiktok.com" => {
            // Short links: use the path segment as the identifier
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
struct TikTokOEmbedResponse {
    author_name: String,
    title: String,
}

// Simple percent-encoding helper to avoid pulling in a full dep
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for byte in s.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9'
                | b'-' | b'_' | b'.' | b'~' => out.push(byte as char),
                _ => {
                    out.push('%');
                    out.push_str(&format!("{:02X}", byte));
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::extract_video_id;

    #[test]
    fn test_standard_tiktok_url() {
        assert_eq!(
            extract_video_id("https://www.tiktok.com/@username/video/1234567890"),
            Some("1234567890".to_string())
        );
    }

    #[test]
    fn test_tiktok_url_without_www() {
        assert_eq!(
            extract_video_id("https://tiktok.com/@username/video/9876543210"),
            Some("9876543210".to_string())
        );
    }

    #[test]
    fn test_vm_short_link() {
        assert_eq!(
            extract_video_id("https://vm.tiktok.com/AbCdEfG/"),
            Some("AbCdEfG".to_string())
        );
    }

    #[test]
    fn test_non_tiktok_url_returns_none() {
        assert_eq!(extract_video_id("https://youtube.com/watch?v=abc"), None);
    }

    #[test]
    fn test_tiktok_url_missing_video_segment_returns_none() {
        assert_eq!(
            extract_video_id("https://www.tiktok.com/@username/"),
            None
        );
    }

    #[test]
    fn test_tiktok_url_wrong_segment_returns_none() {
        assert_eq!(
            extract_video_id("https://www.tiktok.com/@username/photo/1234567890"),
            None
        );
    }

    #[test]
    fn test_invalid_url_returns_none() {
        assert_eq!(extract_video_id("not-a-url"), None);
    }

    #[test]
    fn test_tiktok_url_with_query_params() {
        assert_eq!(
            extract_video_id("https://www.tiktok.com/@username/video/1234567890?lang=en"),
            Some("1234567890".to_string())
        );
    }
}
