// payout.rs — build kind 9734 payout receipt events

use nostr::prelude::*;

/// Build a kind 9734 payout receipt event (based on NIP-57 zap-request shape).
///
/// # Arguments
/// - `clipper_pubkey` — hex-encoded public key of the clipper receiving the payout
/// - `submission_ref` — `"<kind>:<pubkey>:<d-tag>"` coordinate of the submission
/// - `campaign_ref` — `"<kind>:<pubkey>:<d-tag>"` coordinate of the campaign
/// - `relay_url` — relay hint for the `a` tags
/// - `amount_msats` — payout amount in millisatoshis
/// - `description` — human-readable payout description / memo
pub fn build_payout_event(
    clipper_pubkey: impl AsRef<str>,
    submission_ref: impl AsRef<str>,
    campaign_ref: impl AsRef<str>,
    relay_url: impl AsRef<str>,
    amount_msats: u64,
    description: impl Into<String>,
) -> EventBuilder {
    let relay_hint: String = relay_url.as_ref().to_string();
    let description: String = description.into();

    let mut tags: Vec<Tag> = Vec::new();

    // p tag — recipient (clipper) public key
    if let Ok(pubkey) = PublicKey::from_hex(clipper_pubkey.as_ref()) {
        tags.push(Tag::public_key(pubkey));
    } else {
        // Fallback: raw tag if pubkey parse fails (shouldn't happen in production)
        tags.push(
            Tag::parse(["p", clipper_pubkey.as_ref()]).expect("p tag"),
        );
    }

    // a tag — submission reference with relay hint
    tags.push(
        Tag::parse(["a", submission_ref.as_ref(), &relay_hint]).expect("submission a tag"),
    );

    // a tag — campaign reference with relay hint
    tags.push(
        Tag::parse(["a", campaign_ref.as_ref(), &relay_hint]).expect("campaign a tag"),
    );

    // amount tag — in millisatoshis
    tags.push(Tag::parse(["amount", &amount_msats.to_string()]).expect("amount tag"));

    // relays tag — relay hint list
    tags.push(Tag::parse(["relays", &relay_hint]).expect("relays tag"));

    EventBuilder::new(Kind::ZapRequest, description).tags(tags)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A valid 64-char hex pubkey (all zeros except last nibble).
    const TEST_PUBKEY: &str =
        "0000000000000000000000000000000000000000000000000000000000000001";

    fn make_builder() -> EventBuilder {
        build_payout_event(
            TEST_PUBKEY,
            "30403:deadbeef0000000000000000000000000000000000000000000000000000dead:sub-1",
            "30402:deadbeef0000000000000000000000000000000000000000000000000000dead:camp-1",
            "wss://relay.example.com",
            21_000u64,
            "Payout for clip views",
        )
    }

    fn sign(builder: EventBuilder) -> Event {
        let keys = Keys::generate();
        builder.sign_with_keys(&keys).expect("sign")
    }

    #[test]
    fn correct_kind_number() {
        let event = sign(make_builder());
        assert_eq!(event.kind.as_u16(), 9734);
    }

    #[test]
    fn has_p_tag() {
        let event = sign(make_builder());
        let has = event
            .tags
            .iter()
            .any(|t| t.as_slice().first().map(|v| v == "p").unwrap_or(false));
        assert!(has, "expected 'p' tag");
    }

    #[test]
    fn a_tags_include_relay_hint() {
        let event = sign(make_builder());
        let a_tags: Vec<_> = event
            .tags
            .iter()
            .filter(|t| t.as_slice().first().map(|v| v == "a").unwrap_or(false))
            .collect();
        assert_eq!(a_tags.len(), 2, "expected exactly 2 'a' tags (submission + campaign)");
        for tag in &a_tags {
            let relay = tag.as_slice().get(2).map(|s| s.as_str()).unwrap_or("");
            assert_eq!(relay, "wss://relay.example.com", "relay hint missing in a tag");
        }
    }

    #[test]
    fn has_amount_tag() {
        let event = sign(make_builder());
        let has = event.tags.iter().any(|t| {
            let s = t.as_slice();
            s.first().map(|v| v == "amount").unwrap_or(false)
                && s.get(1).map(|v| v == "21000").unwrap_or(false)
        });
        assert!(has, "expected ['amount', '21000'] tag");
    }

    #[test]
    fn has_relays_tag() {
        let event = sign(make_builder());
        let has = event.tags.iter().any(|t| {
            let s = t.as_slice();
            s.first().map(|v| v == "relays").unwrap_or(false)
                && s.get(1)
                    .map(|v| v == "wss://relay.example.com")
                    .unwrap_or(false)
        });
        assert!(has, "expected 'relays' tag with relay URL");
    }

    #[test]
    fn submission_a_tag_present() {
        let event = sign(make_builder());
        let has = event.tags.iter().any(|t| {
            let s = t.as_slice();
            s.first().map(|v| v == "a").unwrap_or(false)
                && s.get(1).map(|v| v.starts_with("30403:")).unwrap_or(false)
        });
        assert!(has, "expected 'a' tag referencing kind 30403 submission");
    }

    #[test]
    fn campaign_a_tag_present() {
        let event = sign(make_builder());
        let has = event.tags.iter().any(|t| {
            let s = t.as_slice();
            s.first().map(|v| v == "a").unwrap_or(false)
                && s.get(1).map(|v| v.starts_with("30402:")).unwrap_or(false)
        });
        assert!(has, "expected 'a' tag referencing kind 30402 campaign");
    }
}
