use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use clipcrate_cashu::wallet::{get_balance, get_transaction_history, TransactionRecord};

use crate::auth::AuthenticatedUser;
use crate::error::ApiError;
use crate::AppState;

// ── Request / Response types ────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    pub balance_sats: i64,
}

#[derive(Debug, Deserialize)]
pub struct WithdrawRequest {
    pub invoice: String,
    pub amount_sats: i64,
}

#[derive(Debug, Serialize)]
pub struct WithdrawResponse {
    pub status: String,
    pub amount_sats: i64,
    pub preimage: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HistoryResponse {
    pub transactions: Vec<TransactionRecord>,
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// GET /api/wallet/balance
pub async fn get_balance_handler(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<BalanceResponse>, ApiError> {
    let balance_sats = get_balance(&state.db, &user.pubkey)
        .await
        .map_err(ApiError::Internal)?;

    Ok(Json(BalanceResponse { balance_sats }))
}

/// POST /api/wallet/withdraw
/// Actually pays the Lightning invoice via Cashu melt.
pub async fn withdraw(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(req): Json<WithdrawRequest>,
) -> Result<(StatusCode, Json<WithdrawResponse>), ApiError> {
    if req.amount_sats <= 0 {
        return Err(ApiError::BadRequest(
            "amount_sats must be greater than 0".to_string(),
        ));
    }

    if req.invoice.is_empty() {
        return Err(ApiError::BadRequest("invoice must not be empty".to_string()));
    }

    let balance = get_balance(&state.db, &user.pubkey)
        .await
        .map_err(ApiError::Internal)?;

    if req.amount_sats > balance {
        return Err(ApiError::BadRequest(format!(
            "insufficient balance: requested {} sats but balance is {} sats",
            req.amount_sats, balance
        )));
    }

    // Insert withdrawal record as pending
    let withdrawal_id = sqlx::query_scalar::<_, uuid::Uuid>(
        r#"
        INSERT INTO withdrawals (clipper_pubkey, amount_sats, lightning_invoice, status)
        VALUES ($1, $2, $3, 'pending')
        RETURNING id
        "#,
    )
    .bind(&user.pubkey)
    .bind(req.amount_sats)
    .bind(&req.invoice)
    .fetch_one(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.into()))?;

    // Actually pay the Lightning invoice via Cashu
    match state.cashu_wallet.withdraw_to_invoice(&req.invoice).await {
        Ok(preimage) => {
            // Mark as completed with preimage
            sqlx::query(
                "UPDATE withdrawals SET status = 'completed', payment_preimage = $1 WHERE id = $2",
            )
            .bind(&preimage)
            .bind(withdrawal_id)
            .execute(&state.db)
            .await
            .map_err(|e| ApiError::Internal(e.into()))?;

            Ok((
                StatusCode::OK,
                Json(WithdrawResponse {
                    status: "completed".to_string(),
                    amount_sats: req.amount_sats,
                    preimage: Some(preimage),
                }),
            ))
        }
        Err(e) => {
            // Mark as failed
            sqlx::query(
                "UPDATE withdrawals SET status = 'failed' WHERE id = $1",
            )
            .bind(withdrawal_id)
            .execute(&state.db)
            .await
            .map_err(|e2| ApiError::Internal(e2.into()))?;

            Err(ApiError::Internal(e))
        }
    }
}

/// GET /api/wallet/history
pub async fn get_history(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<HistoryResponse>, ApiError> {
    let transactions = get_transaction_history(&state.db, &user.pubkey, 50)
        .await
        .map_err(ApiError::Internal)?;

    Ok(Json(HistoryResponse { transactions }))
}
