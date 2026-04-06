use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use clipcrate_db::postgres::{delete_push_subscription, save_push_subscription};

use crate::auth::AuthenticatedUser;
use crate::error::ApiError;
use crate::AppState;

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SubscribeKeys {
    pub p256dh: String,
    pub auth: String,
}

#[derive(Debug, Deserialize)]
pub struct SubscribeRequest {
    pub endpoint: String,
    pub keys: SubscribeKeys,
}

#[derive(Debug, Deserialize)]
pub struct UnsubscribeRequest {
    pub endpoint: String,
}

#[derive(Debug, Serialize)]
pub struct VapidKeyResponse {
    pub vapid_public_key: String,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// POST /api/notifications/subscribe
/// Saves a Web Push subscription for the authenticated user.
pub async fn subscribe(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(req): Json<SubscribeRequest>,
) -> Result<StatusCode, ApiError> {
    save_push_subscription(
        &state.db,
        &user.pubkey,
        &req.endpoint,
        &req.keys.p256dh,
        &req.keys.auth,
    )
    .await
    .map_err(ApiError::Internal)?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/notifications/unsubscribe
/// Removes a Web Push subscription by endpoint.
pub async fn unsubscribe(
    State(state): State<AppState>,
    Json(req): Json<UnsubscribeRequest>,
) -> Result<StatusCode, ApiError> {
    delete_push_subscription(&state.db, &req.endpoint)
        .await
        .map_err(ApiError::Internal)?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/notifications/vapid-key
/// Returns the public VAPID key (no auth required).
pub async fn vapid_key() -> Json<VapidKeyResponse> {
    let key = std::env::var("VAPID_PUBLIC_KEY")
        .unwrap_or_else(|_| "placeholder_vapid_public_key".to_string());

    Json(VapidKeyResponse {
        vapid_public_key: key,
    })
}
