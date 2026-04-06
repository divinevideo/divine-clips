-- Run manually against ClickHouse: clickhouse-client < migrations/002_clickhouse_schema.sql

CREATE TABLE IF NOT EXISTS verification_snapshots (
    submission_id UUID,
    campaign_id UUID,
    clipper_pubkey String,
    timestamp DateTime,
    view_count UInt64,
    source String,
    fraud_score Float32 DEFAULT 0.0
) ENGINE = MergeTree()
ORDER BY (submission_id, timestamp);

CREATE MATERIALIZED VIEW IF NOT EXISTS campaign_analytics
ENGINE = AggregatingMergeTree()
ORDER BY (campaign_id, date)
AS SELECT
    campaign_id,
    toDate(timestamp) AS date,
    maxState(view_count) AS total_views,
    uniqState(submission_id) AS active_submissions
FROM verification_snapshots
GROUP BY campaign_id, date;
