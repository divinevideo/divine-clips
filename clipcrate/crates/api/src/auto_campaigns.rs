use anyhow::{anyhow, ensure, Result};
use nostr_sdk::prelude::*;
use sqlx::PgPool;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use ::url::{Host, Url};

const BASE_CPM_SATS: i32 = 1000;
const PER_VIDEO_BUDGET: i64 = 10000;
const DEFAULT_RELAY_URL: &str = "wss://relay.divine.video";
const DEFAULT_INITIAL_LIMIT: usize = 100;
const MAX_INITIAL_LIMIT: usize = 1000;
const DEFAULT_LOOKBACK_SECS: u64 = 0;
const MAX_LOOKBACK_SECS: u64 = 86_400;
const VIDEO_KIND: u16 = 34236;

#[cfg(test)]
mod postgres_tests;

/// Validated runtime configuration for the auto-campaign Nostr relay listener.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutoCampaignConfig {
    relay_url: String,
    initial_limit: usize,
    lookback_secs: u64,
}

impl AutoCampaignConfig {
    pub fn from_env() -> Result<Self> {
        fn value(name: &str) -> Result<Option<String>> {
            match std::env::var(name) {
                Ok(value) => Ok(Some(value)),
                Err(std::env::VarError::NotPresent) => Ok(None),
                Err(_) => Err(anyhow!("{name} must be valid UTF-8")),
            }
        }

        Self::parse(
            value("RELAY_URL")?,
            value("AUTO_CAMPAIGN_INITIAL_LIMIT")?,
            value("AUTO_CAMPAIGN_LOOKBACK_SECS")?,
        )
    }

    fn parse(
        relay_url: Option<String>,
        initial_limit: Option<String>,
        lookback_secs: Option<String>,
    ) -> Result<Self> {
        let relay_url = relay_url.unwrap_or_else(|| DEFAULT_RELAY_URL.into());
        ensure!(
            !relay_url.is_empty()
                && !relay_url
                    .chars()
                    .any(|c| c.is_whitespace() || c.is_control()),
            "Invalid RELAY_URL"
        );
        let parsed = Url::parse(&relay_url).map_err(|_| anyhow!("Invalid RELAY_URL"))?;
        let authority_has_at = relay_url
            .split_once("://")
            .map(|(_, rest)| {
                let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
                authority.contains('@')
            })
            .unwrap_or(false);
        ensure!(
            matches!(parsed.scheme(), "ws" | "wss")
                && parsed.host().is_some()
                && !authority_has_at
                && parsed.username().is_empty()
                && parsed.password().is_none()
                && parsed.query().is_none()
                && parsed.fragment().is_none(),
            "RELAY_URL must be a ws:// or wss:// endpoint without credentials, query, or fragment"
        );
        if parsed.scheme() == "ws" {
            let host = parsed.host().ok_or_else(|| anyhow!("Invalid RELAY_URL"))?;
            ensure!(
                is_plaintext_ws_allowed_host(&host),
                "RELAY_URL requires wss:// for non-local hosts (ws:// is only permitted for loopback and Kubernetes *.svc / *.svc.cluster.local endpoints)"
            );
        }

        let initial_limit = match initial_limit {
            Some(raw) => {
                let parsed: usize = raw
                    .parse()
                    .map_err(|_| anyhow!("Invalid AUTO_CAMPAIGN_INITIAL_LIMIT"))?;
                ensure!(
                    parsed <= MAX_INITIAL_LIMIT,
                    "AUTO_CAMPAIGN_INITIAL_LIMIT must be between 0 and {MAX_INITIAL_LIMIT}"
                );
                parsed
            }
            None => DEFAULT_INITIAL_LIMIT,
        };

        let lookback_secs = match lookback_secs {
            Some(raw) => {
                let parsed: u64 = raw
                    .parse()
                    .map_err(|_| anyhow!("Invalid AUTO_CAMPAIGN_LOOKBACK_SECS"))?;
                ensure!(
                    parsed <= MAX_LOOKBACK_SECS,
                    "AUTO_CAMPAIGN_LOOKBACK_SECS must be between 0 and {MAX_LOOKBACK_SECS}"
                );
                parsed
            }
            None => DEFAULT_LOOKBACK_SECS,
        };

        Ok(Self {
            relay_url,
            initial_limit,
            lookback_secs,
        })
    }

    pub fn relay_url(&self) -> &str {
        &self.relay_url
    }

    pub fn initial_limit(&self) -> usize {
        self.initial_limit
    }

    pub fn lookback_secs(&self) -> u64 {
        self.lookback_secs
    }
}

fn is_plaintext_ws_allowed_host(host: &Host<&str>) -> bool {
    match host {
        Host::Ipv4(ip) => ip.is_loopback(),
        Host::Ipv6(ip) => ip.is_loopback(),
        Host::Domain(domain) => {
            let without_root_dot = domain.strip_suffix('.').unwrap_or(domain);
            let normalized = without_root_dot.to_ascii_lowercase();
            if normalized == "localhost" {
                return true;
            }
            let prefix = normalized
                .strip_suffix(".svc.cluster.local")
                .or_else(|| normalized.strip_suffix(".svc"));
            matches!(
                prefix,
                Some(p)
                    if p.split('.').count() >= 2
                        && p.split('.').all(|label| !label.is_empty())
            )
        }
    }
}

fn initial_since(now: Timestamp, lookback_secs: u64) -> Timestamp {
    Timestamp::from(now.as_u64().saturating_sub(lookback_secs))
}

fn next_since_cursor(
    current_since: Timestamp,
    event_created_at: Timestamp,
    now: Timestamp,
) -> Timestamp {
    let clamped = Timestamp::from(event_created_at.as_u64().min(now.as_u64()));
    Timestamp::from(current_since.as_u64().max(clamped.as_u64()))
}

fn build_subscription_filter(since: Timestamp, initial_limit: usize) -> Filter {
    Filter::new()
        .kind(Kind::from(VIDEO_KIND))
        .since(since)
        .limit(initial_limit)
}

pub async fn run_auto_campaign_listener(db: PgPool, config: AutoCampaignConfig) {
    let cursor = Arc::new(AtomicU64::new(
        initial_since(Timestamp::now(), config.lookback_secs()).as_u64(),
    ));

    loop {
        let since = Timestamp::from(cursor.load(Ordering::Relaxed));
        match run_listener(&db, &config, since, &cursor).await {
            Ok(()) => {
                tracing::warn!("auto-campaign listener stopped, restarting in 30s");
            }
            Err(e) => {
                tracing::error!("auto-campaign listener error: {e:#}, restarting in 30s");
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    }
}

async fn run_listener(
    db: &PgPool,
    config: &AutoCampaignConfig,
    since: Timestamp,
    cursor: &Arc<AtomicU64>,
) -> Result<()> {
    let keys = Keys::generate();
    let client = Client::new(keys);
    let session_result = run_listener_session(&client, db, config, since, cursor).await;
    let _ = client.shutdown().await;
    session_result
}

async fn run_listener_session(
    client: &Client,
    db: &PgPool,
    config: &AutoCampaignConfig,
    since: Timestamp,
    cursor: &Arc<AtomicU64>,
) -> Result<()> {
    client.add_relay(config.relay_url()).await?;
    // Attach the notification receiver before connecting and subscribing so
    // early events or an immediate CLOSED response cannot be dropped.
    let mut notifications = client.notifications();
    client.connect().await;

    let filter = build_subscription_filter(since, config.initial_limit());
    tracing::info!(
        relay_url = config.relay_url(),
        since = since.as_u64(),
        initial_limit = config.initial_limit(),
        "subscribing auto-campaign listener to relay"
    );
    let output = client.subscribe(vec![filter], None).await?;
    ensure!(
        !output.success.is_empty(),
        "auto-campaign relay subscription did not succeed on any relay (failed_relays={})",
        output.failed.len()
    );
    let sub_id = output.val;

    loop {
        match notifications.recv().await {
            Ok(RelayPoolNotification::Event {
                subscription_id,
                event,
                ..
            }) if subscription_id == sub_id && event.kind == Kind::from(VIDEO_KIND) => {
                match handle_video(db, &event).await {
                    Ok(()) => {
                        let current = Timestamp::from(cursor.load(Ordering::Relaxed));
                        let updated =
                            next_since_cursor(current, event.created_at, Timestamp::now());
                        cursor.fetch_max(updated.as_u64(), Ordering::Relaxed);
                    }
                    Err(e) => {
                        tracing::error!("auto-campaign insert error: {e:#}");
                    }
                }
            }
            Ok(RelayPoolNotification::Message {
                message:
                    RelayMessage::Closed {
                        subscription_id,
                        message,
                    },
                ..
            }) if subscription_id == sub_id => {
                tracing::warn!(reason = %message, "auto-campaign relay closed subscription");
                return Ok(());
            }
            Ok(RelayPoolNotification::Shutdown) => {
                tracing::warn!("auto-campaign relay pool shut down");
                return Ok(());
            }
            Ok(_) => {}
            Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                tracing::warn!(skipped, "auto-campaign notification receiver lagged");
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                tracing::warn!("auto-campaign notification channel closed");
                return Ok(());
            }
        }
    }
}

async fn handle_video(db: &PgPool, event: &Event) -> Result<()> {
    if event.kind != Kind::from(VIDEO_KIND) {
        return Ok(());
    }

    let event_id = event.id.to_hex();
    let creator = event.pubkey.to_hex();

    let title = event
        .tags
        .iter()
        .find_map(|t| {
            let s = t.as_slice();
            if s.first().map(|v| v.as_str()) == Some("title") {
                s.get(1).map(|v| v.to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "DiVine Loop".into());

    let d_tag = event
        .tags
        .iter()
        .find_map(|t| {
            let s = t.as_slice();
            if s.first().map(|v| v.as_str()) == Some("d") {
                s.get(1).map(|v| v.to_string())
            } else {
                None
            }
        })
        .unwrap_or_default();

    let result = sqlx::query(
        "INSERT INTO campaigns (creator_pubkey, title, budget_total_sats, budget_remaining_sats, cpm_sats, status, source, divine_video_event_id, content_refs, target_platforms)
         VALUES ($1, $2, $3, $3, $4, 'active', 'auto', $5, ARRAY[$6], ARRAY['tiktok','instagram','youtube','x'])
         ON CONFLICT (divine_video_event_id) WHERE divine_video_event_id IS NOT NULL DO NOTHING"
    )
    .bind(&creator)
    .bind(format!("Clip: {title}"))
    .bind(PER_VIDEO_BUDGET)
    .bind(BASE_CPM_SATS)
    .bind(&event_id)
    .bind(&d_tag)
    .execute(db)
    .await?;

    if result.rows_affected() > 0 {
        tracing::info!(event_id, creator, title, "auto-campaign created");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_use_public_wss_relay_and_bounded_live_subscription() {
        let cfg = AutoCampaignConfig::parse(None, None, None).unwrap();
        assert_eq!(cfg.relay_url(), "wss://relay.divine.video");
        assert_eq!(cfg.initial_limit(), 100);
        assert_eq!(cfg.lookback_secs(), 0);
    }

    #[test]
    fn allows_internal_cluster_and_loopback_ws_endpoints() {
        for url in [
            "ws://funnelcake-relay.funnelcake.svc.cluster.local:7777",
            "ws://funnelcake-relay.funnelcake.svc:7777",
            "ws://FUNNELCAKE-RELAY.FUNNELCAKE.SVC.CLUSTER.LOCAL.:7777",
            "ws://localhost:7777",
            "ws://127.0.0.1:7777",
            "ws://[::1]:7777",
            "wss://relay.divine.video",
            "wss://relay.internal.example:8443/ws",
        ] {
            let cfg = AutoCampaignConfig::parse(Some(url.into()), None, None)
                .unwrap_or_else(|e| panic!("expected {url} to be accepted: {e}"));
            assert_eq!(cfg.relay_url(), url);
        }
    }

    #[test]
    fn rejects_plaintext_ws_to_external_hosts_and_malformed_urls() {
        for url in [
            "",
            "https://relay.divine.video",
            "http://funnelcake-relay.funnelcake.svc.cluster.local:7777",
            "ws://relay.divine.video",
            "ws://example.com:7777",
            "ws://10.0.0.5:7777",
            "ws://0.0.0.0:7777",
            "ws://[::]:7777",
            "ws://[::ffff:127.0.0.1]:7777",
            "ws://[::ffff:10.0.0.1]:7777",
            "ws://foo.svc:7777",
            "ws://.svc.cluster.local:7777",
            "ws://svc.cluster.local:7777",
            "ws://a..b.svc.cluster.local:7777",
            "ws://localhost..:7777",
            "ws://funnelcake-relay.funnelcake.svc..:7777",
            "ws://localhost.evil.com:7777",
            "ws://a.b.svc.cluster.local.evil.com:7777",
            "wss://relay.divine.video\n",
            "ws://loc\talhost:7777",
            " wss://relay.divine.video ",
            "wss://@relay.divine.video",
            "wss://user@relay.divine.video",
            "wss://:pass@relay.divine.video",
            "wss://user:pass@relay.divine.video",
            "wss://relay.divine.video?token=secret",
            "wss://relay.divine.video#frag",
        ] {
            let err = AutoCampaignConfig::parse(Some(url.into()), None, None)
                .expect_err(&format!("expected {url:?} to be rejected"));
            let msg = err.to_string();
            assert!(!msg.contains("secret"), "error leaked query value: {msg}");
            assert!(!msg.contains("pass"), "error leaked password value: {msg}");
        }
    }

    #[test]
    fn validates_initial_limit_and_lookback_bounds() {
        let zero = AutoCampaignConfig::parse(None, Some("0".into()), Some("0".into())).unwrap();
        assert_eq!(zero.initial_limit(), 0);
        assert_eq!(zero.lookback_secs(), 0);

        let max =
            AutoCampaignConfig::parse(None, Some("1000".into()), Some("86400".into())).unwrap();
        assert_eq!(max.initial_limit(), 1000);
        assert_eq!(max.lookback_secs(), 86_400);

        assert!(AutoCampaignConfig::parse(None, Some("1001".into()), None).is_err());
        assert!(AutoCampaignConfig::parse(None, Some("-1".into()), None).is_err());
        assert!(AutoCampaignConfig::parse(None, None, Some("86401".into())).is_err());
        assert!(AutoCampaignConfig::parse(None, None, Some("abc".into())).is_err());
    }

    #[test]
    fn subscription_filter_bounds_history_with_since_and_limit() {
        let since = Timestamp::from(1_795_400_000);
        let filter = build_subscription_filter(since, 100);
        let json = serde_json::to_value(&filter).unwrap();
        assert_eq!(json["kinds"], serde_json::json!([34236]));
        assert_eq!(json["since"], serde_json::json!(1_795_400_000));
        assert_eq!(json["limit"], serde_json::json!(100));

        let zero_limit = build_subscription_filter(since, 0);
        let zero_json = serde_json::to_value(&zero_limit).unwrap();
        assert_eq!(zero_json["since"], serde_json::json!(1_795_400_000));
        assert_eq!(zero_json["limit"], serde_json::json!(0));
    }

    #[test]
    fn since_cursor_advances_monotonically_and_clamps_future_timestamps() {
        let now = Timestamp::from(2_000);
        assert_eq!(initial_since(now, 300), Timestamp::from(1_700));
        assert_eq!(initial_since(Timestamp::from(100), 300), Timestamp::from(0));

        // Advances on newer in-range event
        assert_eq!(
            next_since_cursor(Timestamp::from(1_700), Timestamp::from(1_850), now),
            Timestamp::from(1_850)
        );
        // Never moves backward on older out-of-order event
        assert_eq!(
            next_since_cursor(Timestamp::from(1_850), Timestamp::from(1_800), now),
            Timestamp::from(1_850)
        );
        // Clamps future-dated event timestamp to now so it cannot starve live events
        assert_eq!(
            next_since_cursor(Timestamp::from(1_850), Timestamp::from(9_999_999), now),
            now
        );
    }
}
