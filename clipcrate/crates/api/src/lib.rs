// clipcrate-api: Campaign and submission REST API handlers
// Provides Axum router definitions, request/response types, and handler functions
// for managing video campaigns and content creator submissions.

pub mod auth;
pub mod campaigns;
pub mod dashboard;
pub mod error;
pub mod feed;
pub mod internal;
pub mod submissions;
pub mod wallet;

use axum::{
    routing::{get, patch, post},
    Router,
};
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: PgPool,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/campaigns", post(campaigns::create_campaign))
        .route("/campaigns", get(campaigns::list_campaigns))
        .route("/campaigns/{id}", get(campaigns::get_campaign))
        .route("/campaigns/{id}", patch(campaigns::update_campaign))
        .route("/api/submissions", post(submissions::create_submission))
        .route("/api/submissions", get(submissions::list_submissions))
        .route("/api/submissions/{id}", get(submissions::get_submission))
        .with_state(state)
}
