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

    sqlx::query(
        r#"
        INSERT INTO withdrawals (clipper_pubkey, amount_sats, lightning_invoice, status)
        VALUES ($1, $2, $3, 'pending')
        "#,
    )
    .bind(&user.pubkey)
    .bind(req.amount_sats)
    .bind(&req.invoice)
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.into()))?;

    Ok((
        StatusCode::CREATED,
        Json(WithdrawResponse {
            status: "pending".to_string(),
            amount_sats: req.amount_sats,
        }),
    ))
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
