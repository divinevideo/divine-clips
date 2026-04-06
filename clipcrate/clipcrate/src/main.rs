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
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/clipcrate".to_string());

    let pool = clipcrate_db::postgres::create_pool(&database_url).await?;
    info!("connected to database");

    let state = AppState { db: pool };
    let app = clipcrate_api::router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3100").await?;
    info!("clipcrate listening on 0.0.0.0:3100");

    axum::serve(listener, app).await?;

    Ok(())
}
