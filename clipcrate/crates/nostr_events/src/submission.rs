// submission.rs — build kind 30403 submission events

use nostr::prelude::*;

/// Build a kind 30403 submission event.
///
/// # Arguments
/// - `submission_id` — unique `d`-tag identifier for this submission
/// - `campaign_ref` — `"<kind>:<pubkey>:<d-tag>"` coordinate of the campaign being submitted to
/// - `relay_url` — relay hint included in the `a` tag
/// - `external_url` — URL of the submitted clip (e.g. YouTube link)
/// - `platform` — platform name (e.g. `"youtube"`)
pub fn build_submission_event(
    submission_id: impl Into<String>,
    campaign_ref: impl AsRef<str>,
    relay_url: impl AsRef<str>,
    external_url: impl Into<String>,
    platform: impl AsRef<str>,
) -> EventBuilder {
    let submission_id: String = submission_id.into();
    let relay_hint: String = relay_url.as_ref().to_string();
    let external_url: String = external_url.into();

    let mut tags: Vec<Tag> = Vec::new();

    // d tag — addressable event identifier
    tags.push(Tag::identifier(submission_id));

    // a tag — reference to campaign, with relay hint
    tags.push(
        Tag::parse(["a", campaign_ref.as_ref(), &relay_hint]).expect("a tag"),
    );

    // r tag — external URL of the submitted clip
    tags.push(Tag::reference(external_url));

    // platform hashtag
    tags.push(Tag::hashtag(platform.as_ref().to_string()));

    // submission status
    tags.push(Tag::parse(["status", "pending"]).expect("status tag"));

    // label namespace tags (NIP-32)
    tags.push(Tag::parse(["L", "divine-clips"]).expect("L tag"));
    tags.push(Tag::parse(["l", "submission", "divine-clips"]).expect("l tag"));

    EventBuilder::new(Kind::from_u16(30403), "")
        .tags(tags)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_builder() -> EventBuilder {
        build_submission_event(
            "sub-uuid-1",
            "30402:deadbeef0000000000000000000000000000000000000000000000000000dead:camp-1",
            "wss://relay.example.com",
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
            "youtube",
        )
    }

    fn sign(builder: EventBuilder) -> Event {
        let keys = Keys::generate();
        builder.sign_with_keys(&keys).expect("sign")
    }

    #[test]
    fn correct_kind_number() {
        let event = sign(make_builder());
        assert_eq!(event.kind.as_u16(), 30403);
    }

    #[test]
    fn has_d_tag() {
        let event = sign(make_builder());
        let has = event
            .tags
            .iter()
            .any(|t| t.as_slice().first().map(|v| v == "d").unwrap_or(false)
                && t.as_slice().get(1).map(|v| v == "sub-uuid-1").unwrap_or(false));
        assert!(has, "expected ['d', 'sub-uuid-1'] tag");
    }

    #[test]
    fn a_tag_has_relay_hint() {
        let event = sign(make_builder());
        let a_tags: Vec<_> = event
            .tags
            .iter()
            .filter(|t| t.as_slice().first().map(|v| v == "a").unwrap_or(false))
            .collect();
        assert!(!a_tags.is_empty(), "expected at least one 'a' tag");
        for tag in &a_tags {
            let relay = tag.as_slice().get(2).map(|s| s.as_str()).unwrap_or("");
            assert_eq!(relay, "wss://relay.example.com", "relay hint missing");
        }
    }

    #[test]
    fn has_r_tag() {
        let event = sign(make_builder());
        let has = event.tags.iter().any(|t| {
            let s = t.as_slice();
            s.first().map(|v| v == "r").unwrap_or(false)
                && s.get(1)
                    .map(|v| v.contains("youtube.com"))
                    .unwrap_or(false)
        });
        assert!(has, "expected 'r' tag with YouTube URL");
    }

    #[test]
    fn has_platform_tag() {
        let event = sign(make_builder());
        let has = event.tags.iter().any(|t| {
            let s = t.as_slice();
            s.first().map(|v| v == "t").unwrap_or(false)
                && s.get(1).map(|v| v == "youtube").unwrap_or(false)
        });
        assert!(has, "expected ['t', 'youtube'] tag");
    }

    #[test]
    fn has_status_tag() {
        let event = sign(make_builder());
        let has = event
            .tags
            .iter()
            .any(|t| t.as_slice().first().map(|v| v == "status").unwrap_or(false));
        assert!(has, "expected 'status' tag");
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
                && s.get(1).map(|v| v == "submission").unwrap_or(false)
                && s.get(2).map(|v| v == "divine-clips").unwrap_or(false)
        });
        assert!(has_l_upper, "expected ['L', 'divine-clips'] tag");
        assert!(has_l_lower, "expected ['l', 'submission', 'divine-clips'] tag");
    }
}
