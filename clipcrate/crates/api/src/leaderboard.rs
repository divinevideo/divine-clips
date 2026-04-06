use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

use clipcrate_db::postgres;

use crate::error::ApiError;
use crate::AppState;

// ── Request types ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LeaderboardParams {
    #[serde(default = "default_metric")]
    pub metric: String,
    #[serde(default = "default_period")]
    pub period: String,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_metric() -> String {
    "earnings".to_string()
}

fn default_period() -> String {
    "week".to_string()
}

fn default_limit() -> i64 {
    50
}

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct LeaderboardEntryResponse {
    pub pubkey: String,
    pub trust_level: i32,
    pub value: i64,
}

#[derive(Debug, Serialize)]
pub struct SocialProofResponse {
    pub clippers_this_week: i64,
    pub sats_earned_this_week: i64,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/leaderboard?metric=earnings&period=week&limit=50
pub async fn get_leaderboard(
    State(state): State<AppState>,
    Query(params): Query<LeaderboardParams>,
) -> Result<Json<Vec<LeaderboardEntryResponse>>, ApiError> {
    let limit = params.limit.min(100).max(1);
    let entries = postgres::get_leaderboard(&state.db, &params.metric, &params.period, limit)
        .await
        .map_err(ApiError::Internal)?;

    let response: Vec<LeaderboardEntryResponse> = entries
        .into_iter()
        .map(|e| LeaderboardEntryResponse {
            pubkey: e.pubkey,
            trust_level: e.trust_level,
            value: e.value,
        })
        .collect();

    Ok(Json(response))
}

/// GET /api/stats/social-proof
pub async fn get_social_proof(
    State(state): State<AppState>,
) -> Result<Json<SocialProofResponse>, ApiError> {
    let (clippers_this_week, sats_earned_this_week) =
        postgres::get_social_proof_stats(&state.db)
            .await
            .map_err(ApiError::Internal)?;

    Ok(Json(SocialProofResponse {
        clippers_this_week,
        sats_earned_this_week,
    }))
}
