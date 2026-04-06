// campaign.rs — build kind 30402 (NIP-15 classified listing) campaign events

use nostr::prelude::*;

/// Build a kind 30402 campaign event (NIP-15 classified listing).
///
/// # Arguments
/// - `title` — display name of the campaign
/// - `summary` — short description
/// - `content_refs` — `"<kind>:<pubkey>:<d-tag>"` addresses of target content
/// - `relay_url` — relay hint to include in `a` tags
/// - `cpm_sats` — cost per 1000 views in sats
/// - `budget_sats` — total campaign budget in sats
/// - `target_platforms` — list of platform identifiers (e.g. `["youtube", "tiktok"]`)
/// - `expires_at` — optional UNIX timestamp for campaign expiry
/// - `campaign_id` — unique `d`-tag identifier for this campaign
/// - `guidelines` — detailed campaign guidelines placed in the event content
pub fn build_campaign_event(
    title: impl Into<String>,
    summary: impl Into<String>,
    content_refs: &[impl AsRef<str>],
    relay_url: impl AsRef<str>,
    cpm_sats: u64,
    budget_sats: u64,
    target_platforms: &[impl AsRef<str>],
    expires_at: Option<u64>,
    campaign_id: impl Into<String>,
    guidelines: impl Into<String>,
) -> EventBuilder {
    let title: String = title.into();
    let summary: String = summary.into();
    let campaign_id: String = campaign_id.into();
    let guidelines: String = guidelines.into();
    let relay_hint: String = relay_url.as_ref().to_string();

    let mut tags: Vec<Tag> = Vec::new();

    // d tag — addressable event identifier
    tags.push(Tag::identifier(campaign_id));

    // title tag
    tags.push(Tag::title(title.clone()));

    // summary tag
    tags.push(Tag::parse(["summary", &summary]).expect("summary tag"));

    // a tags for each content reference with relay hint
    for content_ref in content_refs {
        tags.push(
            Tag::parse(["a", content_ref.as_ref(), &relay_hint]).expect("a tag"),
        );
    }

    // price tag: ["price", cpm_sats, "sats", "per_1000_views"]
    tags.push(
        Tag::parse(["price", &cpm_sats.to_string(), "sats", "per_1000_views"])
            .expect("price tag"),
    );

    // budget tag
    tags.push(Tag::parse(["budget", &budget_sats.to_string()]).expect("budget tag"));

    // status tag
    tags.push(Tag::parse(["status", "active"]).expect("status tag"));

    // label namespace tags (NIP-32)
    tags.push(Tag::parse(["L", "divine-clips"]).expect("L tag"));
    tags.push(Tag::parse(["l", "campaign", "divine-clips"]).expect("l tag"));

    // platform hashtags
    for platform in target_platforms {
        tags.push(Tag::hashtag(platform.as_ref().to_string()));
    }

    // optional expiration tag
    if let Some(ts) = expires_at {
        tags.push(Tag::expiration(Timestamp::from(ts)));
    }

    EventBuilder::new(Kind::from_u16(30402), guidelines).tags(tags)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_builder() -> EventBuilder {
        build_campaign_event(
            "Test Campaign",
            "A short summary",
            &["30402:deadbeef0000000000000000000000000000000000000000000000000000dead:camp-1"],
            "wss://relay.example.com",
            100u64,
            50000u64,
            &["youtube", "tiktok"],
            Some(9999999999),
            "camp-uuid-1",
            "Create engaging clips from the source video.",
        )
    }

    /// Helper: sign the builder and return the resulting event for inspection.
    fn sign(builder: EventBuilder) -> Event {
        let keys = Keys::generate();
        builder.sign_with_keys(&keys).expect("sign")
    }

    #[test]
    fn correct_kind_number() {
        let event = sign(make_builder());
        assert_eq!(event.kind.as_u16(), 30402);
    }

    #[test]
    fn has_d_tag() {
        let event = sign(make_builder());
        let has_d = event
            .tags
            .iter()
            .any(|t| t.as_slice().first().map(|s| s == "d").unwrap_or(false)
                && t.as_slice().get(1).map(|s| s == "camp-uuid-1").unwrap_or(false));
        assert!(has_d, "expected ['d', 'camp-uuid-1'] tag");
    }

    #[test]
    fn has_title_tag() {
        let event = sign(make_builder());
        let has_title = event
            .tags
            .iter()
            .any(|t| t.as_slice().first().map(|s| s == "title").unwrap_or(false)
                && t.as_slice().get(1).map(|s| s == "Test Campaign").unwrap_or(false));
        assert!(has_title, "expected ['title', 'Test Campaign'] tag");
    }

    #[test]
    fn has_summary_tag() {
        let event = sign(make_builder());
        let has = event
            .tags
            .iter()
            .any(|t| t.as_slice().first().map(|s| s == "summary").unwrap_or(false));
        assert!(has, "expected 'summary' tag");
    }

    #[test]
    fn a_tags_include_relay_hint() {
        let event = sign(make_builder());
        let a_tags: Vec<_> = event
            .tags
            .iter()
            .filter(|t| t.as_slice().first().map(|s| s == "a").unwrap_or(false))
            .collect();
        assert!(!a_tags.is_empty(), "expected at least one 'a' tag");
        for tag in &a_tags {
            let relay = tag.as_slice().get(2).map(|s| s.as_str()).unwrap_or("");
            assert_eq!(relay, "wss://relay.example.com", "relay hint missing in a tag");
        }
    }

    #[test]
    fn has_price_tag() {
        let event = sign(make_builder());
        let has = event.tags.iter().any(|t| {
            let s = t.as_slice();
            s.first().map(|v| v == "price").unwrap_or(false)
                && s.get(1).map(|v| v == "100").unwrap_or(false)
                && s.get(2).map(|v| v == "sats").unwrap_or(false)
                && s.get(3).map(|v| v == "per_1000_views").unwrap_or(false)
        });
        assert!(has, "expected ['price', '100', 'sats', 'per_1000_views'] tag");
    }

    #[test]
    fn has_budget_tag() {
        let event = sign(make_builder());
        let has = event.tags.iter().any(|t| {
            let s = t.as_slice();
            s.first().map(|v| v == "budget").unwrap_or(false)
                && s.get(1).map(|v| v == "50000").unwrap_or(false)
        });
        assert!(has, "expected ['budget', '50000'] tag");
    }

    #[test]
    fn has_label_namespace_tags() {
        let event = sign(make_builder());
        let has_l_upper = event.tags.iter().any(|t| {
            let s = t.as_slice();
            s.first().map(|v| v == "L").unwrap_or(false)
                && s.get(1).map(|v| v == "divine-clips").unwrap_or(false)
        });
        let has_l_lower = event.tags.iter().any(|t| {
            let s = t.as_slice();
            s.first().map(|v| v == "l").unwrap_or(false)
                && s.get(1).map(|v| v == "campaign").unwrap_or(false)
                && s.get(2).map(|v| v == "divine-clips").unwrap_or(false)
        });
        assert!(has_l_upper, "expected ['L', 'divine-clips'] tag");
        assert!(has_l_lower, "expected ['l', 'campaign', 'divine-clips'] tag");
    }

    #[test]
    fn has_status_active_tag() {
        let event = sign(make_builder());
        let has = event.tags.iter().any(|t| {
            let s = t.as_slice();
            s.first().map(|v| v == "status").unwrap_or(false)
                && s.get(1).map(|v| v == "active").unwrap_or(false)
        });
        assert!(has, "expected ['status', 'active'] tag");
    }

    #[test]
    fn has_platform_hashtags() {
        let event = sign(make_builder());
        let platforms: Vec<&str> = event
            .tags
            .iter()
            .filter(|t| t.as_slice().first().map(|v| v == "t").unwrap_or(false))
            .filter_map(|t| t.as_slice().get(1).map(|s| s.as_str()))
            .collect();
        assert!(platforms.contains(&"youtube"), "expected 'youtube' hashtag");
        assert!(platforms.contains(&"tiktok"), "expected 'tiktok' hashtag");
    }

    #[test]
    fn has_expiration_tag() {
        let event = sign(make_builder());
        let has = event
            .tags
            .iter()
            .any(|t| t.as_slice().first().map(|v| v == "expiration").unwrap_or(false));
        assert!(has, "expected 'expiration' tag");
    }

    #[test]
    fn no_expiration_tag_when_none() {
        let builder = build_campaign_event(
            "No Expiry",
            "summary",
            &[] as &[&str],
            "wss://relay.example.com",
            50u64,
            1000u64,
            &[] as &[&str],
            None,
            "camp-no-exp",
            "guidelines",
        );
        let event = sign(builder);
        let has = event
            .tags
            .iter()
            .any(|t| t.as_slice().first().map(|v| v == "expiration").unwrap_or(false));
        assert!(!has, "unexpected 'expiration' tag when expires_at is None");
    }

    #[test]
    fn minimum_tag_count() {
        // d, title, summary, price, budget, status, L, l = 8 minimum
        let event = sign(make_builder());
        assert!(
            event.tags.len() >= 8,
            "expected at least 8 tags, got {}",
            event.tags.len()
        );
    }
}
