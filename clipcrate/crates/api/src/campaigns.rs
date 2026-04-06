use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use clipcrate_db::models::Campaign;
use clipcrate_db::postgres;

use crate::auth::AuthenticatedUser;
use crate::error::ApiError;
use crate::AppState;

const VALID_PLATFORMS: &[&str] = &["youtube", "tiktok", "instagram", "twitter", "x"];

// ── Request / Response types ────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateCampaignRequest {
    pub title: String,
    pub budget_sats: i64,
    pub cpm_sats: i32,
    pub target_platforms: Vec<String>,
    pub content_refs: Vec<String>,
    pub guidelines: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCampaignRequest {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

#[derive(Debug, Serialize)]
pub struct CampaignResponse {
    pub campaign: Campaign,
}

#[derive(Debug, Serialize)]
pub struct CampaignsResponse {
    pub campaigns: Vec<Campaign>,
    pub limit: i64,
    pub offset: i64,
}

// ── Validation helpers ───────────────────────────────────────────────────────

fn validate_create_request(req: &CreateCampaignRequest) -> Result<(), ApiError> {
    if req.budget_sats < 1000 {
        return Err(ApiError::BadRequest(
            "budget_sats must be at least 1000".to_string(),
        ));
    }
    if req.cpm_sats <= 0 {
        return Err(ApiError::BadRequest(
            "cpm_sats must be positive".to_string(),
        ));
    }
    if req.target_platforms.is_empty() {
        return Err(ApiError::BadRequest(
            "target_platforms must not be empty".to_string(),
        ));
    }
    for platform in &req.target_platforms {
        if !VALID_PLATFORMS.contains(&platform.to_lowercase().as_str()) {
            return Err(ApiError::BadRequest(format!(
                "invalid platform '{}'; allowed: {}",
                platform,
                VALID_PLATFORMS.join(", ")
            )));
        }
    }
    Ok(())
}

const VALID_STATUSES: &[&str] = &["active", "paused", "cancelled"];

fn validate_status(status: &str) -> Result<(), ApiError> {
    if !VALID_STATUSES.contains(&status) {
        return Err(ApiError::BadRequest(format!(
            "invalid status '{}'; allowed: {}",
            status,
            VALID_STATUSES.join(", ")
        )));
    }
    Ok(())
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// POST /campaigns
pub async fn create_campaign(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(req): Json<CreateCampaignRequest>,
) -> Result<(StatusCode, Json<CampaignResponse>), ApiError> {
    validate_create_request(&req)?;

    let campaign = postgres::create_campaign(
        &state.db,
        &user.pubkey,
        &req.title,
        req.budget_sats,
        req.cpm_sats,
        req.target_platforms,
        req.content_refs,
        req.guidelines.as_deref(),
        req.expires_at,
    )
    .await
    .map_err(ApiError::Internal)?;

    Ok((StatusCode::CREATED, Json(CampaignResponse { campaign })))
}

/// GET /campaigns
pub async fn list_campaigns(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<CampaignsResponse>, ApiError> {
    let limit = params.limit.clamp(1, 100);
    let offset = params.offset.max(0);

    let campaigns = postgres::list_active_campaigns(&state.db, limit, offset)
        .await
        .map_err(ApiError::Internal)?;

    Ok(Json(CampaignsResponse {
        campaigns,
        limit,
        offset,
    }))
}

/// GET /campaigns/:id
pub async fn get_campaign(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<CampaignResponse>, ApiError> {
    let campaign = postgres::get_campaign(&state.db, id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(CampaignResponse { campaign }))
}

/// PATCH /campaigns/:id
pub async fn update_campaign(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCampaignRequest>,
) -> Result<Json<CampaignResponse>, ApiError> {
    validate_status(&req.status)?;

    let campaign = postgres::update_campaign_status(&state.db, id, &user.pubkey, &req.status)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(CampaignResponse { campaign }))
}
