use axum::{routing::get, Router};
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

async fn health() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize structured tracing
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(fmt::layer().json())
        .init();

    let app = Router::new().route("/health", get(health));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3100").await?;
    info!("clipcrate listening on 0.0.0.0:3100");

    axum::serve(listener, app).await?;

    Ok(())
}
