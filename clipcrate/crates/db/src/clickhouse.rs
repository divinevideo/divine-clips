use std::sync::Arc;

#[derive(Clone)]
pub struct ClickHouseClient {
    client: Arc<clickhouse::Client>,
}

impl std::fmt::Debug for ClickHouseClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClickHouseClient").finish()
    }
}

impl ClickHouseClient {
    pub fn new(url: &str) -> Self {
        Self {
            client: Arc::new(clickhouse::Client::default().with_url(url)),
        }
    }
}
