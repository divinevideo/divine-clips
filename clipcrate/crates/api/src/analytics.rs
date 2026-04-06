use axum::{
    extract::{Path, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::auth::AuthenticatedUser;
use crate::error::ApiError;
use crate::AppState;

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ViewSnapshot {
    pub timestamp: i64,
    pub view_count: i64,
}

#[derive(Debug, Serialize)]
pub struct PayoutPoint {
    pub timestamp: DateTime<Utc>,
    pub amount_sats: i64,
    pub cumulative_sats: i64,
}

#[derive(Debug, Serialize)]
pub struct SubmissionAnalyticsResponse {
    pub snapshots: Vec<ViewSnapshot>,
    pub payouts: Vec<PayoutPoint>,
}

#[derive(Debug, Serialize)]
pub struct DailyViews {
    pub date: String,
    pub views: i64,
}

#[derive(Debug, Serialize)]
pub struct PlatformStats {
    pub platform: String,
    pub views: i64,
    pub earned_sats: i64,
}

#[derive(Debug, Serialize)]
pub struct OverviewResponse {
    pub daily_views: Vec<DailyViews>,
    pub by_platform: Vec<PlatformStats>,
}

// ── Internal row types for sqlx::query_as ────────────────────────────────────

#[derive(sqlx::FromRow)]
struct SubmissionOwner {
    clipper_pubkey: String,
}

#[derive(sqlx::FromRow)]
struct PayoutRow {
    created_at: DateTime<Utc>,
    amount_sats: i64,
    cumulative_sats: Option<i64>,
}

#[derive(sqlx::FromRow)]
struct PlatformRow {
    platform: String,
    views: Option<i64>,
    earned_sats: Option<i64>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/analytics/submission/{id}
pub async fn get_submission_analytics(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<Json<SubmissionAnalyticsResponse>, ApiError> {
    // Verify the submission belongs to the authenticated user
    let submission = sqlx::query_as::<_, SubmissionOwner>(
        "SELECT clipper_pubkey FROM submissions WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?
    .ok_or(ApiError::NotFound)?;

    if submission.clipper_pubkey != user.pubkey {
        return Err(ApiError::Forbidden);
    }

    // Fetch ClickHouse snapshots (gracefully returns empty on error)
    let ch_snapshots = state
        .clickhouse
        .get_submission_snapshots(&id.to_string())
        .await;

    let snapshots: Vec<ViewSnapshot> = ch_snapshots
        .into_iter()
        .map(|s| ViewSnapshot {
            timestamp: s.timestamp as i64,
            view_count: s.view_count,
        })
        .collect();

    // Fetch payouts from Postgres with running cumulative sum
    let payout_rows = sqlx::query_as::<_, PayoutRow>(
        r#"
        SELECT created_at, amount_sats,
               SUM(amount_sats) OVER (ORDER BY created_at) AS cumulative_sats
        FROM payouts
        WHERE submission_id = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?;

    let payouts: Vec<PayoutPoint> = payout_rows
        .into_iter()
        .map(|r| PayoutPoint {
            timestamp: r.created_at,
            amount_sats: r.amount_sats,
            cumulative_sats: r.cumulative_sats.unwrap_or(0),
        })
        .collect();

    Ok(Json(SubmissionAnalyticsResponse { snapshots, payouts }))
}

/// GET /api/analytics/overview
pub async fn get_overview(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<OverviewResponse>, ApiError> {
    // Fetch ClickHouse daily views (gracefully returns empty on error)
    let ch_daily = state
        .clickhouse
        .get_clipper_daily_views(&user.pubkey)
        .await;

    let daily_views: Vec<DailyViews> = ch_daily
        .into_iter()
        .map(|d| DailyViews {
            date: d.date,
            views: d.views,
        })
        .collect();

    // Fetch per-platform stats from Postgres (always available)
    let platform_rows = sqlx::query_as::<_, PlatformRow>(
        r#"
        SELECT
            s.platform,
            COALESCE(SUM(s.total_verified_views), 0)::bigint AS views,
            COALESCE(SUM(p.amount_sats), 0)::bigint AS earned_sats
        FROM submissions s
        LEFT JOIN payouts p ON p.submission_id = s.id
        WHERE s.clipper_pubkey = $1
        GROUP BY s.platform
        ORDER BY views DESC
        "#,
    )
    .bind(&user.pubkey)
    .fetch_all(&state.db)
    .await
    .map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?;

    let by_platform: Vec<PlatformStats> = platform_rows
        .into_iter()
        .map(|r| PlatformStats {
            platform: r.platform,
            views: r.views.unwrap_or(0),
            earned_sats: r.earned_sats.unwrap_or(0),
        })
        .collect();

    Ok(Json(OverviewResponse {
        daily_views,
        by_platform,
    }))
}
