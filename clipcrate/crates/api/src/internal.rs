use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use clipcrate_db::models::{Payout, Submission};
use clipcrate_db::postgres;

use crate::auth::ServiceAuth;
use crate::error::ApiError;
use crate::AppState;

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct VerificationResult {
    pub submission_id: Uuid,
    pub view_count: i64,
    pub source: String,
    pub fraud_score: f32,
}

#[derive(Debug, Deserialize)]
pub struct VerificationBatch {
    pub results: Vec<VerificationResult>,
}

#[derive(Debug, Serialize)]
pub struct VerificationOutcome {
    pub submission_id: Uuid,
    pub views_recorded: i64,
    pub payout_sats: i64,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct VerificationBatchResponse {
    pub outcomes: Vec<VerificationOutcome>,
}

#[derive(Debug, Deserialize)]
pub struct TriggerPayoutRequest {
    pub submission_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct PendingSubmissionsResponse {
    pub submissions: Vec<Submission>,
}

#[derive(Debug, Serialize)]
pub struct PayoutResponse {
    pub payout: Payout,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/internal/submissions
pub async fn list_pending_submissions(
    State(state): State<AppState>,
    _auth: ServiceAuth,
) -> Result<Json<PendingSubmissionsResponse>, ApiError> {
    let submissions = postgres::list_pending_submissions(&state.db)
        .await
        .map_err(ApiError::Internal)?;

    Ok(Json(PendingSubmissionsResponse { submissions }))
}

/// POST /api/internal/verifications
pub async fn post_verifications(
    State(state): State<AppState>,
    _auth: ServiceAuth,
    Json(batch): Json<VerificationBatch>,
) -> Result<Json<VerificationBatchResponse>, ApiError> {
    let mut outcomes = Vec::with_capacity(batch.results.len());

    for result in batch.results {
        let outcome = process_verification(&state, result).await;
        outcomes.push(outcome);
    }

    Ok(Json(VerificationBatchResponse { outcomes }))
}

async fn process_verification(
    state: &AppState,
    result: VerificationResult,
) -> VerificationOutcome {
    match try_process_verification(state, result.submission_id, result.view_count).await {
        Ok((views, payout_sats)) => VerificationOutcome {
            submission_id: result.submission_id,
            views_recorded: views,
            payout_sats,
            status: "ok".to_string(),
        },
        Err(err) => {
            tracing::error!(
                submission_id = %result.submission_id,
                "verification processing failed: {:?}", err
            );
            VerificationOutcome {
                submission_id: result.submission_id,
                views_recorded: 0,
                payout_sats: 0,
                status: format!("error: {}", err),
            }
        }
    }
}

async fn try_process_verification(
    state: &AppState,
    submission_id: Uuid,
    view_count: i64,
) -> anyhow::Result<(i64, i64)> {
    // Step 1: Update submission views (outside transaction; idempotent view update)
    let submission = postgres::update_submission_views(&state.db, submission_id, view_count)
        .await?
        .ok_or_else(|| anyhow::anyhow!("submission not found: {}", submission_id))?;

    // Step 2: Load the associated campaign
    let campaign = postgres::get_campaign(&state.db, submission.campaign_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("campaign not found: {}", submission.campaign_id))?;

    // Step 3: Calculate how much is owed
    let earned_sats = (view_count as i64 * campaign.cpm_sats as i64) / 1000;
    let already_paid = submission.total_paid_sats;
    let unpaid_sats = (earned_sats - already_paid).max(0);

    if unpaid_sats == 0 || campaign.budget_remaining_sats == 0 || campaign.status == "exhausted" {
        return Ok((view_count, 0));
    }

    // Step 4: Begin transaction — deduct budget, create payout, update submission paid_sats
    let mut tx = state.db.begin().await?;

    // Lock campaign row
    let locked_campaign = sqlx::query_as::<_, clipcrate_db::models::Campaign>(
        "SELECT * FROM campaigns WHERE id = $1 FOR UPDATE",
    )
    .bind(campaign.id)
    .fetch_optional(&mut *tx)
    .await?;

    let locked_campaign = match locked_campaign {
        Some(c) => c,
        None => {
            tx.rollback().await?;
            return Ok((view_count, 0));
        }
    };

    if locked_campaign.budget_remaining_sats == 0 || locked_campaign.status == "exhausted" {
        tx.rollback().await?;
        return Ok((view_count, 0));
    }

    // Pay up to available budget (partial payment if budget is insufficient)
    let actual_payout = unpaid_sats.min(locked_campaign.budget_remaining_sats);
    let new_budget = locked_campaign.budget_remaining_sats - actual_payout;
    let new_status = if new_budget == 0 { "exhausted" } else { locked_campaign.status.as_str() };

    // Deduct from campaign
    sqlx::query(
        r#"
        UPDATE campaigns
        SET budget_remaining_sats = $2,
            status = $3,
            updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(campaign.id)
    .bind(new_budget)
    .bind(new_status)
    .execute(&mut *tx)
    .await?;

    // Create payout record
    let payout = sqlx::query_as::<_, clipcrate_db::models::Payout>(
        r#"
        INSERT INTO payouts (submission_id, campaign_id, clipper_pubkey, amount_sats, views_at_payout)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(submission.id)
    .bind(campaign.id)
    .bind(&submission.clipper_pubkey)
    .bind(actual_payout)
    .bind(view_count)
    .fetch_one(&mut *tx)
    .await?;

    // Update submission total_paid_sats
    sqlx::query(
        r#"
        UPDATE submissions
        SET total_paid_sats = total_paid_sats + $2
        WHERE id = $1
        "#,
    )
    .bind(submission.id)
    .bind(actual_payout)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    tracing::info!(
        submission_id = %submission.id,
        payout_id = %payout.id,
        amount_sats = actual_payout,
        "payout created"
    );

    Ok((view_count, actual_payout))
}

/// POST /api/internal/payouts — trigger manual payout for a submission
pub async fn trigger_payout(
    State(state): State<AppState>,
    _auth: ServiceAuth,
    Json(req): Json<TriggerPayoutRequest>,
) -> Result<(StatusCode, Json<PayoutResponse>), ApiError> {
    let submission = postgres::get_submission(&state.db, req.submission_id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;

    let campaign = postgres::get_campaign(&state.db, submission.campaign_id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or(ApiError::NotFound)?;

    let earned_sats = (submission.total_verified_views * campaign.cpm_sats as i64) / 1000;
    let unpaid_sats = (earned_sats - submission.total_paid_sats).max(0);

    if unpaid_sats == 0 {
        return Err(ApiError::BadRequest(
            "no unpaid amount for this submission".to_string(),
        ));
    }

    if campaign.budget_remaining_sats == 0 || campaign.status == "exhausted" {
        return Err(ApiError::BadRequest(
            "campaign budget is exhausted".to_string(),
        ));
    }

    let mut tx = state.db.begin().await.map_err(|e| ApiError::Internal(e.into()))?;

    // Lock campaign row
    let locked_campaign = sqlx::query_as::<_, clipcrate_db::models::Campaign>(
        "SELECT * FROM campaigns WHERE id = $1 FOR UPDATE",
    )
    .bind(campaign.id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(e.into()))?
    .ok_or(ApiError::NotFound)?;

    let actual_payout = unpaid_sats.min(locked_campaign.budget_remaining_sats);
    let new_budget = locked_campaign.budget_remaining_sats - actual_payout;
    let new_status = if new_budget == 0 { "exhausted" } else { locked_campaign.status.as_str() };

    sqlx::query(
        r#"
        UPDATE campaigns
        SET budget_remaining_sats = $2,
            status = $3,
            updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(campaign.id)
    .bind(new_budget)
    .bind(new_status)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(e.into()))?;

    let payout = sqlx::query_as::<_, clipcrate_db::models::Payout>(
        r#"
        INSERT INTO payouts (submission_id, campaign_id, clipper_pubkey, amount_sats, views_at_payout)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(submission.id)
    .bind(campaign.id)
    .bind(&submission.clipper_pubkey)
    .bind(actual_payout)
    .bind(submission.total_verified_views)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(e.into()))?;

    sqlx::query(
        r#"
        UPDATE submissions
        SET total_paid_sats = total_paid_sats + $2
        WHERE id = $1
        "#,
    )
    .bind(submission.id)
    .bind(actual_payout)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(e.into()))?;

    tx.commit().await.map_err(|e| ApiError::Internal(e.into()))?;

    Ok((StatusCode::CREATED, Json(PayoutResponse { payout })))
}
