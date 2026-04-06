// clipcrate-db: Postgres and ClickHouse database clients
// Provides connection pool setup, query helpers, and typed repository interfaces
// for persisting campaign data in Postgres and analytics in ClickHouse.

pub mod models;
pub mod postgres;
pub mod clickhouse;
