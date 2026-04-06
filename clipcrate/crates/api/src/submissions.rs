use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use clipcrate_db::models::Submission;
use clipcrate_db::postgres;
use clipcrate_trust::max_active_submissions;

use crate::auth::AuthenticatedUser;
use crate::error::ApiError;
use crate::AppState;

const VALID_PLATFORMS: &[&str] = &["tiktok", "instagram", "youtube", "x"];

/// Platform host suffixes used for URL domain validation.
fn platform_host_matches(platform: &str, host: &str) -> bool {
    match platform {
        "tiktok" => host.ends_with("tiktok.com"),
        "instagram" => host.ends_with("instagram.com"),
        "youtube" => host.ends_with("youtube.com") || host.ends_with("youtu.be"),
        "x" => host.ends_with("x.com") || host.ends_with("twitter.com"),
        _ => false,
    }
}

// ── Request / Response types ────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateSubmissionRequest {
    pub campaign_id: Uuid,
    pub external_url: String,
    pub platform: String,
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
pub struct SubmissionResponse {
    pub submission: Submission,
}

#[derive(Debug, Serialize)]
pub struct SubmissionsResponse {
    pub submissions: Vec<Submission>,
    pub limit: i64,
    pub offset: i64,
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// POST /api/submissions
pub async fn create_submission(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(req): Json<CreateSubmissionRequest>,
) -> Result<(StatusCode, Json<SubmissionResponse>), ApiError> {
    // Validate platform
    let platform = req.platform.to_lowercase();
    if !VALID_PLATFORMS.contains(&platform.as_str()) {
        return Err(ApiError::BadRequest(format!(
            "invalid platform '{}'; allowed: {}",
            req.platform,
            VALID_PLATFORMS.join(", ")
        )));
    }

    // Validate URL is well-formed and host matches platform
    let parsed = url::Url::parse(&req.external_url)
        .map_err(|_| ApiError::BadRequest("invalid URL".to_string()))?;

    let host = parsed
        .host_str()
        .ok_or_else(|| ApiError::BadRequest("URL has no host".to_string()))?;

    if !platform_host_matches(&platform, host) {
        return Err(ApiError::BadRequest(format!(
            "URL host '{}' does not match platform '{}'",
            host, platform
        )));
    }

    // Check campaign exists and is active
    let campaign = postgres::get_campaign(&state.db, req.campaign_id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;

    if campaign.status != "active" {
        return Err(ApiError::BadRequest(
            "campaign is not active".to_string(),
        ));
    }

    // Get or create clipper, then check trust-based submission limit
    let clipper = postgres::get_or_create_clipper(&state.db, &user.pubkey)
        .await
        .map_err(ApiError::Internal)?;

    let active_count = postgres::count_active_submissions(&state.db, &user.pubkey)
        .await
        .map_err(ApiError::Internal)?;

    let limit = max_active_submissions(clipper.trust_level) as i64;
    if active_count >= limit {
        return Err(ApiError::BadRequest(format!(
            "trust level {} allows at most {} active submissions",
            clipper.trust_level, limit
        )));
    }

    // Insert submission; catch unique violation (campaign+url duplicate) → 409
    let submission = postgres::create_submission(
        &state.db,
        req.campaign_id,
        &user.pubkey,
        &req.external_url,
        &platform,
    )
    .await
    .map_err(|err| {
        // sqlx wraps the PgDatabaseError; check the code string
        if let Some(db_err) = err.downcast_ref::<sqlx::Error>() {
            if let sqlx::Error::Database(ref dbe) = db_err {
                if dbe.code().as_deref() == Some("23505") {
                    return ApiError::Conflict(
                        "submission for this URL and campaign already exists".to_string(),
                    );
                }
            }
        }
        ApiError::Internal(err)
    })?;

    Ok((StatusCode::CREATED, Json(SubmissionResponse { submission })))
}

/// GET /api/submissions
pub async fn list_submissions(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(params): Query<PaginationParams>,
) -> Result<Json<SubmissionsResponse>, ApiError> {
    let limit = params.limit.clamp(1, 100);
    let offset = params.offset.max(0);

    let submissions =
        postgres::list_clipper_submissions(&state.db, &user.pubkey, limit, offset)
            .await
            .map_err(ApiError::Internal)?;

    Ok(Json(SubmissionsResponse {
        submissions,
        limit,
        offset,
    }))
}

/// GET /api/submissions/:id
pub async fn get_submission(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<SubmissionResponse>, ApiError> {
    let submission = postgres::get_submission(&state.db, id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(SubmissionResponse { submission }))
}
