use std::sync::Arc;

use clickhouse::Row;
use serde::Deserialize;

#[derive(Clone)]
pub struct ClickHouseClient {
    client: Arc<clickhouse::Client>,
}

impl std::fmt::Debug for ClickHouseClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClickHouseClient").finish()
    }
}

#[derive(Debug, Clone, Row, Deserialize)]
pub struct ViewSnapshot {
    pub timestamp: u32,
    pub view_count: i64,
}

#[derive(Debug, Clone, Row, Deserialize)]
pub struct DailyViews {
    pub date: String,
    pub views: i64,
}

impl ClickHouseClient {
    pub fn new(url: &str) -> Self {
        Self {
            client: Arc::new(clickhouse::Client::default().with_url(url)),
        }
    }

    /// Returns time-series view snapshots for a single submission.
    /// Returns empty vec on connection/query errors to handle unpopulated ClickHouse gracefully.
    pub async fn get_submission_snapshots(&self, submission_id: &str) -> Vec<ViewSnapshot> {
        let query = format!(
            "SELECT toUnixTimestamp(timestamp) AS timestamp, view_count \
             FROM submission_view_snapshots \
             WHERE submission_id = '{}' \
             ORDER BY timestamp ASC \
             LIMIT 500",
            submission_id.replace('\'', "")
        );
        match self.client.query(&query).fetch_all::<ViewSnapshot>().await {
            Ok(rows) => rows,
            Err(e) => {
                tracing::warn!("ClickHouse get_submission_snapshots error: {}", e);
                vec![]
            }
        }
    }

    /// Returns daily aggregate view counts for a clipper.
    /// Returns empty vec on connection/query errors to handle unpopulated ClickHouse gracefully.
    pub async fn get_clipper_daily_views(&self, clipper_pubkey: &str) -> Vec<DailyViews> {
        let query = format!(
            "SELECT formatDateTime(toDate(timestamp), '%Y-%m-%d') AS date, \
                    sum(view_count) AS views \
             FROM submission_view_snapshots \
             WHERE clipper_pubkey = '{}' \
               AND timestamp >= now() - INTERVAL 30 DAY \
             GROUP BY date \
             ORDER BY date ASC",
            clipper_pubkey.replace('\'', "")
        );
        match self.client.query(&query).fetch_all::<DailyViews>().await {
            Ok(rows) => rows,
            Err(e) => {
                tracing::warn!("ClickHouse get_clipper_daily_views error: {}", e);
                vec![]
            }
        }
    }
}
