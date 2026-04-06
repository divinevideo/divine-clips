use nostr_sdk::prelude::*;
use sqlx::PgPool;

const BASE_CPM_SATS: i32 = 1000;
const PER_VIDEO_BUDGET: i64 = 10000;

pub async fn run_auto_campaign_listener(db: PgPool) {
    loop {
        if let Err(e) = run_listener(&db).await {
            tracing::error!("auto-campaign listener error: {e:#}, restarting in 30s");
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        }
    }
}

async fn run_listener(db: &PgPool) -> anyhow::Result<()> {
    let keys = Keys::generate();
    let client = Client::new(keys);
    client.add_relay("wss://relay.divine.video").await?;
    client.connect().await;

    let filter = Filter::new().kind(Kind::from(34236));
    client.subscribe(vec![filter], None).await?;

    client.handle_notifications(|notification| async {
        if let RelayPoolNotification::Event { event, .. } = notification {
            if let Err(e) = handle_video(db, &event).await {
                tracing::error!("auto-campaign insert error: {e:#}");
            }
        }
        Ok(false)
    }).await?;

    Ok(())
}

async fn handle_video(db: &PgPool, event: &Event) -> anyhow::Result<()> {
    let event_id = event.id.to_hex();
    let creator = event.pubkey.to_hex();

    let title = event.tags.iter()
        .find_map(|t| {
            let s = t.as_slice();
            if s.first().map(|v| v.as_str()) == Some("title") { s.get(1).map(|v| v.to_string()) }
            else { None }
        })
        .unwrap_or_else(|| "DiVine Loop".into());

    let d_tag = event.tags.iter()
        .find_map(|t| {
            let s = t.as_slice();
            if s.first().map(|v| v.as_str()) == Some("d") { s.get(1).map(|v| v.to_string()) }
            else { None }
        })
        .unwrap_or_default();

    sqlx::query(
        "INSERT INTO campaigns (creator_pubkey, title, budget_total_sats, budget_remaining_sats, cpm_sats, status, source, divine_video_event_id, content_refs, target_platforms)
         VALUES ($1, $2, $3, $3, $4, 'active', 'auto', $5, ARRAY[$6], ARRAY['tiktok','instagram','youtube','x'])
         ON CONFLICT (divine_video_event_id) DO NOTHING"
    )
    .bind(&creator)
    .bind(format!("Clip: {title}"))
    .bind(PER_VIDEO_BUDGET)
    .bind(BASE_CPM_SATS)
    .bind(&event_id)
    .bind(&d_tag)
    .execute(db).await?;

    tracing::info!(event_id, creator, title, "auto-campaign created");
    Ok(())
}
