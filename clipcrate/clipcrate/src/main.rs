use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use clipcrate_api::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize structured tracing
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(fmt::layer().json())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost:5432/clipcrate".into());
    let clickhouse_url = std::env::var("CLICKHOUSE_URL")
        .unwrap_or_else(|_| "http://localhost:8123".into());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3100".into());

    let db = clipcrate_db::postgres::create_pool(&database_url).await?;
    info!("connected to database");

    let clickhouse = clipcrate_db::clickhouse::ClickHouseClient::new(&clickhouse_url);

    let cashu_wallet = clipcrate_cashu::cashu_wallet::from_env().await?;
    info!("cashu wallet initialized");

    let state = AppState {
        db,
        clickhouse,
        cashu_wallet,
    };
    let app = clipcrate_api::router(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("clipcrate listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
