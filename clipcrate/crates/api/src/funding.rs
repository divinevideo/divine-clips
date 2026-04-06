use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::AuthenticatedUser;
use crate::error::ApiError;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct FundingInvoiceResponse {
    pub invoice: String,
    pub quote_id: String,
    pub amount_sats: i64,
}

#[derive(Debug, Deserialize)]
pub struct FundingRequest {
    pub amount_sats: i64,
}

#[derive(Debug, Serialize)]
pub struct FundingStatusResponse {
    pub paid: bool,
    pub amount_credited_sats: Option<i64>,
}

/// POST /api/campaigns/:id/fund
/// Creates a Lightning invoice for funding a campaign via Cashu mint.
pub async fn create_funding_invoice(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(campaign_id): Path<Uuid>,
    Json(req): Json<FundingRequest>,
) -> Result<(StatusCode, Json<FundingInvoiceResponse>), ApiError> {
    if req.amount_sats <= 0 {
        return Err(ApiError::BadRequest(
            "amount_sats must be greater than 0".to_string(),
        ));
    }

    // Verify campaign exists
    let campaign = sqlx::query_as::<_, (String,)>(
        "SELECT status FROM campaigns WHERE id = $1",
    )
    .bind(campaign_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.into()))?;

    if campaign.is_none() {
        return Err(ApiError::NotFound);
    }

    let (invoice, quote_id) = state
        .cashu_wallet
        .create_funding_invoice(req.amount_sats as u64)
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;

    // Store the pending funding record
    sqlx::query(
        r#"
        INSERT INTO campaign_funding (campaign_id, quote_id, amount_sats, status)
        VALUES ($1, $2, $3, 'pending')
        "#,
    )
    .bind(campaign_id)
    .bind(&quote_id)
    .bind(req.amount_sats)
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.into()))?;

    Ok((
        StatusCode::CREATED,
        Json(FundingInvoiceResponse {
            invoice,
            quote_id,
            amount_sats: req.amount_sats,
        }),
    ))
}

/// GET /api/campaigns/:id/fund/:quote_id
/// Check if a funding invoice has been paid and credit the campaign if so.
pub async fn check_funding_status(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path((campaign_id, quote_id)): Path<(Uuid, String)>,
) -> Result<Json<FundingStatusResponse>, ApiError> {
    // Check if already credited
    let existing = sqlx::query_as::<_, (String,)>(
        "SELECT status FROM campaign_funding WHERE campaign_id = $1 AND quote_id = $2",
    )
    .bind(campaign_id)
    .bind(&quote_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.into()))?;

    match existing {
        None => {
            return Err(ApiError::NotFound);
        }
        Some((status,)) if status == "completed" => {
            let (amount,): (i64,) = sqlx::query_as(
                "SELECT amount_sats FROM campaign_funding WHERE campaign_id = $1 AND quote_id = $2",
            )
            .bind(campaign_id)
            .bind(&quote_id)
            .fetch_one(&state.db)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;

            return Ok(Json(FundingStatusResponse {
                paid: true,
                amount_credited_sats: Some(amount),
            }));
        }
        _ => {} // pending — check with mint
    }

    let paid = state
        .cashu_wallet
        .check_funding_paid(&quote_id)
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;

    if !paid {
        return Ok(Json(FundingStatusResponse {
            paid: false,
            amount_credited_sats: None,
        }));
    }

    // Invoice is paid — mint the tokens and credit the campaign
    let _balance = state
        .cashu_wallet
        .complete_funding(&quote_id)
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;

    let (amount_sats,): (i64,) = sqlx::query_as(
        "SELECT amount_sats FROM campaign_funding WHERE campaign_id = $1 AND quote_id = $2",
    )
    .bind(campaign_id)
    .bind(&quote_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.into()))?;

    // Credit the campaign budget and mark funding as completed
    let mut tx = state.db.begin().await.map_err(|e| ApiError::Internal(e.into()))?;

    sqlx::query(
        "UPDATE campaigns SET budget_remaining_sats = budget_remaining_sats + $1, budget_total_sats = budget_total_sats + $1 WHERE id = $2",
    )
    .bind(amount_sats)
    .bind(campaign_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(e.into()))?;

    sqlx::query(
        "UPDATE campaign_funding SET status = 'completed' WHERE campaign_id = $1 AND quote_id = $2",
    )
    .bind(campaign_id)
    .bind(&quote_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| ApiError::Internal(e.into()))?;

    tx.commit().await.map_err(|e| ApiError::Internal(e.into()))?;

    tracing::info!(
        %campaign_id,
        amount_sats,
        "Campaign funded via Cashu"
    );

    Ok(Json(FundingStatusResponse {
        paid: true,
        amount_credited_sats: Some(amount_sats),
    }))
}
