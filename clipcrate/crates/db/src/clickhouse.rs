pub struct ClickHouseClient {
    client: clickhouse::Client,
}

impl ClickHouseClient {
    pub fn new(url: &str) -> Self {
        Self {
            client: clickhouse::Client::default().with_url(url),
        }
    }
}
