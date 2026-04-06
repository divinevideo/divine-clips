// clipcrate-api: Campaign and submission REST API handlers
// Provides Axum router definitions, request/response types, and handler functions
// for managing video campaigns and content creator submissions.

pub mod auth;
pub mod campaigns;
pub mod dashboard;
pub mod error;
pub mod feed;
pub mod funding;
pub mod internal;
pub mod leaderboard;
pub mod submissions;
pub mod wallet;

use axum::{
    http::{HeaderValue, Method},
    routing::{get, patch, post},
    Router,
};
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub clickhouse: clipcrate_db::clickhouse::ClickHouseClient,
    pub cashu_wallet: clipcrate_cashu::cashu_wallet::CashuWallet,
}

pub fn router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin([
            "https://clips.divine.video".parse::<HeaderValue>().unwrap(),
            "https://divine.video".parse::<HeaderValue>().unwrap(),
            "http://localhost:5173".parse::<HeaderValue>().unwrap(),
        ])
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::OPTIONS])
        .allow_headers(Any);

    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/api/campaigns", post(campaigns::create_campaign))
        .route("/api/campaigns", get(campaigns::list_campaigns))
        .route("/api/campaigns/{id}", get(campaigns::get_campaign))
        .route("/api/campaigns/{id}", patch(campaigns::update_campaign))
        .route("/api/campaigns/{id}/fund", post(funding::create_funding_invoice))
        .route("/api/campaigns/{id}/fund/{quote_id}", get(funding::check_funding_status))
        .route("/api/submissions", post(submissions::create_submission))
        .route("/api/submissions", get(submissions::list_submissions))
        .route("/api/submissions/{id}", get(submissions::get_submission))
        .route("/api/feed/live", get(feed::live_feed))
        .route("/api/internal/submissions", get(internal::list_pending_submissions))
        .route("/api/internal/verifications", post(internal::post_verifications))
        .route("/api/internal/payouts", post(internal::trigger_payout))
        .route("/api/wallet/balance", get(wallet::get_balance_handler))
        .route("/api/wallet/withdraw", post(wallet::withdraw))
        .route("/api/wallet/history", get(wallet::get_history))
        .route("/api/dashboard", get(dashboard::get_dashboard))
        .route("/api/leaderboard", get(leaderboard::get_leaderboard))
        .route("/api/stats/social-proof", get(leaderboard::get_social_proof))
        .layer(cors)
        .with_state(state)
}
