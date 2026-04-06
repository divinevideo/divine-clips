use axum::{
    extract::FromRequestParts,
    http::{request::Parts, HeaderMap},
};

use crate::error::ApiError;

/// Authenticated user extracted from Authorization Bearer header.
/// For MVP, the token is treated as the user's pubkey directly.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub pubkey: String,
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let pubkey = extract_bearer_token(&parts.headers)
            .ok_or(ApiError::Unauthorized)?;

        Ok(AuthenticatedUser { pubkey })
    }
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    let auth_header = headers.get("Authorization")?.to_str().ok()?;
    let token = auth_header.strip_prefix("Bearer ")?;
    if token.is_empty() {
        return None;
    }
    Some(token.to_string())
}

/// Internal service authentication from x-service-token header.
#[derive(Debug, Clone)]
pub struct ServiceAuth;

impl<S> FromRequestParts<S> for ServiceAuth
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let _token = parts
            .headers
            .get("x-service-token")
            .and_then(|v| v.to_str().ok())
            .filter(|t| !t.is_empty())
            .ok_or(ApiError::Unauthorized)?;

        Ok(ServiceAuth)
    }
}
