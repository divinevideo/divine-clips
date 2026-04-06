use axum::{
    extract::State,
    Json,
};
use serde::Serialize;

use clipcrate_cashu::wallet::get_balance;
use clipcrate_db::postgres;
use clipcrate_trust::max_weekly_views;

use crate::auth::AuthenticatedUser;
use crate::error::ApiError;
use crate::AppState;

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct DashboardResponse {
    pub trust_level: i32,
    pub total_verified_views: i64,
    pub total_earned_sats: i64,
    pub balance_sats: i64,
    pub active_submissions: i64,
    pub weekly_views_used: i64,
    pub weekly_views_limit: i64,
}

// ── Handler ───────────────────────────────────────────────────────────────────

/// GET /api/dashboard
pub async fn get_dashboard(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<DashboardResponse>, ApiError> {
    let clipper = postgres::get_or_create_clipper(&state.db, &user.pubkey)
        .await
        .map_err(ApiError::Internal)?;

    let active_submissions = postgres::count_active_submissions(&state.db, &user.pubkey)
        .await
        .map_err(ApiError::Internal)?;

    let balance_sats = get_balance(&state.db, &user.pubkey)
        .await
        .map_err(ApiError::Internal)?;

    let weekly_views_limit = max_weekly_views(clipper.trust_level);

    Ok(Json(DashboardResponse {
        trust_level: clipper.trust_level,
        total_verified_views: clipper.total_verified_views,
        total_earned_sats: clipper.total_earned_sats,
        balance_sats,
        active_submissions,
        weekly_views_used: clipper.weekly_views_used,
        weekly_views_limit,
    }))
}
