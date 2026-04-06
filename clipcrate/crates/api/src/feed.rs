use axum::extract::State;
use axum::response::sse::{Event, Sse};
use futures::stream::Stream;
use std::convert::Infallible;

use crate::AppState;

pub async fn live_feed(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = async_stream::stream! {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
        loop {
            interval.tick().await;
            if let Ok(campaigns) = clipcrate_db::postgres::list_recent_campaigns(&state.db, 5).await {
                for campaign in campaigns {
                    let data = serde_json::json!({
                        "type": "campaign",
                        "id": campaign.id,
                        "title": campaign.title,
                        "cpm_sats": campaign.cpm_sats,
                        "budget_remaining_sats": campaign.budget_remaining_sats,
                    });
                    yield Ok(Event::default().event("campaign").data(data.to_string()));
                }
            }
        }
    };
    Sse::new(stream)
}
