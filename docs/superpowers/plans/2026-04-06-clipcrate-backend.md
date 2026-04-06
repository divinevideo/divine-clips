# Clipcrate Backend Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the clipcrate Rust API service and clips-verifier CF Worker — the complete backend for DiVine Clips.

**Architecture:** Cargo workspace with Axum REST API, Postgres (sqlx) for mutable state, ClickHouse for analytics, Moksha for Cashu mint, nostr crate for event publishing to funnelcake. CF Worker (TypeScript) for periodic view count verification via platform APIs + Phyllo.

**Tech Stack:** Rust (Axum, sqlx, clickhouse-rs, nostr v0.37, moksha-core), TypeScript (Cloudflare Workers, wrangler), Postgres, ClickHouse, Cashu/Moksha.

**Spec:** `docs/superpowers/specs/2026-04-06-divine-clips-design.md`

**Patterns to follow:** Mirror divine-funnelcake workspace structure (crates/, shared workspace deps, tracing + Prometheus observability). Auth via Keycast UCAN Bearer tokens (see keycast/api/src/ucan_auth/).

---

## File Structure

```
clipcrate/
├── Cargo.toml                          # Workspace root
├── clipcrate/
│   ├── Cargo.toml                      # Main binary crate
│   └── src/
│       └── main.rs                     # Entry point: config, DB pools, start Axum
├── crates/
│   ├── api/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                  # Router builder
│   │       ├── auth.rs                 # Keycast UCAN token validation middleware
│   │       ├── campaigns.rs            # Campaign CRUD handlers
│   │       ├── submissions.rs          # Submission handlers
│   │       ├── wallet.rs               # Cashu balance, withdraw, history
│   │       ├── dashboard.rs            # Clipper stats endpoint
│   │       ├── internal.rs             # Service-to-service endpoints (verifier)
│   │       ├── feed.rs                 # SSE live activity feed
│   │       └── error.rs               # API error types
│   ├── db/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                  # Re-exports
│   │       ├── postgres.rs             # Postgres pool + query helpers
│   │       ├── clickhouse.rs           # ClickHouse client + verification writes
│   │       └── models.rs              # Campaign, Submission, Clipper, Payout structs
│   ├── nostr_events/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                  # Re-exports
│   │       ├── campaign.rs             # Build kind 30402 events
│   │       ├── submission.rs           # Build kind 30403 events
│   │       ├── payout.rs              # Build kind 9734 payout receipt events
│   │       └── publisher.rs           # Publish events to funnelcake relay via WS
│   ├── cashu/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                  # Re-exports
│   │       ├─�� mint.rs                # Moksha mint wrapper (deposit, issue, melt)
│   │       └── wallet.rs             # Clipper balance tracking + redemption
│   ├── trust/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                  # Trust level calculation + enforcement
│   │       └── fraud.rs              # Fraud flag checks (velocity, duplicates)
│   └── platform-apis/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs                  # Platform trait + re-exports
│           ├── youtube.rs             # YouTube Data API v3 client
│           ├── tiktok.rs              # TikTok oEmbed + Display API
│           ├── instagram.rs           # Instagram Graph API (stub for MVP)
│           └── twitter.rs             # X API v2 client
├── migrations/
│   ├── 001_initial_schema.sql         # Postgres tables + indexes
│   └── 002_clickhouse_schema.sql      # ClickHouse tables (run manually)
└── tests/
    └── integration/
        ├── campaign_test.rs           # Campaign CRUD integration tests
        ├── submission_test.rs         # Submission flow tests
        └── helpers.rs                 # Test fixtures, DB setup

clips-verifier/
├── package.json
├── wrangler.toml                      # CF Worker config + cron triggers
├── tsconfig.json
├── vitest.config.ts
├── src/
│   ├── index.ts                       # Worker entry: scheduled handler
│   ├── clipcrate-client.ts           # HTTP client for clipcrate internal API
│   ├── platforms/
│   │   ├── youtube.ts                # YouTube Data API v3
│   │   ├── tiktok.ts                 # TikTok oEmbed
│   │   └── types.ts                  # ViewCount result type
│   ├── phyllo.ts                     # Phyllo fallback client
│   └── fraud.ts                      # Basic fraud checks
└── test/
    ├── youtube.test.ts
    ├── tiktok.test.ts
    └── scheduled.test.ts
```

---

## Chunk 1: Project Scaffold + Database

### Task 1: Initialize Cargo workspace

**Files:**
- Create: `clipcrate/Cargo.toml` (workspace root)
- Create: `clipcrate/clipcrate/Cargo.toml` (binary crate)
- Create: `clipcrate/clipcrate/src/main.rs`

- [ ] **Step 1: Create workspace Cargo.toml**

```toml
# clipcrate/Cargo.toml
[workspace]
resolver = "2"
members = [
    "clipcrate",
    "crates/api",
    "crates/db",
    "crates/nostr_events",
    "crates/cashu",
    "crates/trust",
    "crates/platform-apis",
]

[workspace.dependencies]
# Web
axum = { version = "0.8", features = ["macros"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace"] }
tokio = { version = "1", features = ["full"] }

# Database
sqlx = { version = "0.8", features = ["runtime-tokio", "tls-rustls", "postgres", "uuid", "chrono"] }
clickhouse = "0.13"

# Nostr
nostr = "0.37"
nostr-sdk = "0.37"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }

# Utilities
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1"
thiserror = "2"
```

- [ ] **Step 2: Create binary crate Cargo.toml**

```toml
# clipcrate/clipcrate/Cargo.toml
[package]
name = "clipcrate"
version = "0.1.0"
edition = "2021"

[dependencies]
clipcrate-api = { path = "../crates/api" }
clipcrate-db = { path = "../crates/db" }
clipcrate-nostr = { path = "../crates/nostr_events" }
clipcrate-cashu = { path = "../crates/cashu" }
clipcrate-trust = { path = "../crates/trust" }

axum.workspace = true
tokio.workspace = true
tower-http.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
serde.workspace = true
anyhow.workspace = true
```

- [ ] **Step 3: Create minimal main.rs**

```rust
// clipcrate/clipcrate/src/main.rs
use anyhow::Result;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    tracing::info!("clipcrate starting");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3100").await?;
    tracing::info!("listening on {}", listener.local_addr()?);

    let app = axum::Router::new()
        .route("/health", axum::routing::get(|| async { "ok" }));

    axum::serve(listener, app).await?;
    Ok(())
}
```

- [ ] **Step 4: Create stub Cargo.toml for each crate**

Create minimal `Cargo.toml` and `src/lib.rs` for each crate in `crates/`:
- `crates/api/` — `clipcrate-api`
- `crates/db/` — `clipcrate-db`
- `crates/nostr_events/` — `clipcrate-nostr`
- `crates/cashu/` — `clipcrate-cashu`
- `crates/trust/` — `clipcrate-trust`
- `crates/platform-apis/` — `clipcrate-platform-apis`

Each starts with just:
```rust
// src/lib.rs
// [crate name] - [one-line description]
```

- [ ] **Step 5: Verify workspace builds**

Run: `cd clipcrate && cargo check`
Expected: Successful compilation with no errors.

- [ ] **Step 6: Commit**

```bash
git add clipcrate/
git commit -m "feat: initialize clipcrate Cargo workspace with stub crates"
```

---

### Task 2: Postgres schema + migrations

**Files:**
- Create: `clipcrate/migrations/001_initial_schema.sql`
- Modify: `clipcrate/crates/db/Cargo.toml`
- Create: `clipcrate/crates/db/src/models.rs`
- Modify: `clipcrate/crates/db/src/lib.rs`

- [ ] **Step 1: Write the Postgres migration**

```sql
-- clipcrate/migrations/001_initial_schema.sql

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Campaigns
CREATE TABLE campaigns (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    nostr_event_id TEXT,
    creator_pubkey TEXT NOT NULL,
    title TEXT NOT NULL,
    budget_total_sats BIGINT NOT NULL,
    budget_remaining_sats BIGINT NOT NULL,
    cpm_sats INT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    target_platforms TEXT[] NOT NULL DEFAULT '{}',
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT valid_status CHECK (status IN ('pending', 'active', 'paused', 'completed', 'exhausted')),
    CONSTRAINT positive_budget CHECK (budget_total_sats > 0),
    CONSTRAINT budget_not_negative CHECK (budget_remaining_sats >= 0)
);

-- Submissions
CREATE TABLE submissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    nostr_event_id TEXT,
    campaign_id UUID NOT NULL REFERENCES campaigns(id),
    clipper_pubkey TEXT NOT NULL,
    external_url TEXT NOT NULL,
    platform TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    total_verified_views BIGINT NOT NULL DEFAULT 0,
    total_paid_sats BIGINT NOT NULL DEFAULT 0,
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_verified_at TIMESTAMPTZ,
    CONSTRAINT valid_platform CHECK (platform IN ('tiktok', 'instagram', 'youtube', 'x')),
    CONSTRAINT valid_sub_status CHECK (status IN ('pending', 'active', 'verified', 'rejected', 'abandoned')),
    UNIQUE (external_url, campaign_id)
);

-- Clippers
CREATE TABLE clippers (
    pubkey TEXT PRIMARY KEY,
    trust_level INT NOT NULL DEFAULT 1,
    total_verified_views BIGINT NOT NULL DEFAULT 0,
    total_earned_sats BIGINT NOT NULL DEFAULT 0,
    weekly_views_used BIGINT NOT NULL DEFAULT 0,
    weekly_views_reset_at TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '7 days',
    phyllo_account_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Payout ledger
CREATE TABLE payouts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    submission_id UUID NOT NULL REFERENCES submissions(id),
    campaign_id UUID NOT NULL REFERENCES campaigns(id),
    clipper_pubkey TEXT NOT NULL,
    amount_sats BIGINT NOT NULL,
    views_at_payout BIGINT NOT NULL,
    cashu_token_id TEXT,
    zap_event_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_campaigns_status ON campaigns(status);
CREATE INDEX idx_campaigns_creator ON campaigns(creator_pubkey);
CREATE INDEX idx_submissions_campaign ON submissions(campaign_id);
CREATE INDEX idx_submissions_status ON submissions(status);
CREATE INDEX idx_submissions_clipper ON submissions(clipper_pubkey);
CREATE INDEX idx_payouts_clipper ON payouts(clipper_pubkey);
CREATE INDEX idx_payouts_submission ON payouts(submission_id);
```

- [ ] **Step 2: Write the db models**

```rust
// clipcrate/crates/db/src/models.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Campaign {
    pub id: Uuid,
    pub nostr_event_id: Option<String>,
    pub creator_pubkey: String,
    pub title: String,
    pub budget_total_sats: i64,
    pub budget_remaining_sats: i64,
    pub cpm_sats: i32,
    pub status: String,
    pub target_platforms: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Submission {
    pub id: Uuid,
    pub nostr_event_id: Option<String>,
    pub campaign_id: Uuid,
    pub clipper_pubkey: String,
    pub external_url: String,
    pub platform: String,
    pub status: String,
    pub total_verified_views: i64,
    pub total_paid_sats: i64,
    pub submitted_at: DateTime<Utc>,
    pub last_verified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Clipper {
    pub pubkey: String,
    pub trust_level: i32,
    pub total_verified_views: i64,
    pub total_earned_sats: i64,
    pub weekly_views_used: i64,
    pub weekly_views_reset_at: DateTime<Utc>,
    pub phyllo_account_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Payout {
    pub id: Uuid,
    pub submission_id: Uuid,
    pub campaign_id: Uuid,
    pub clipper_pubkey: String,
    pub amount_sats: i64,
    pub views_at_payout: i64,
    pub cashu_token_id: Option<String>,
    pub zap_event_id: Option<String>,
    pub created_at: DateTime<Utc>,
}
```

- [ ] **Step 3: Write the db crate Cargo.toml and lib.rs**

```toml
# clipcrate/crates/db/Cargo.toml
[package]
name = "clipcrate-db"
version = "0.1.0"
edition = "2021"

[dependencies]
sqlx.workspace = true
clickhouse.workspace = true
serde.workspace = true
serde_json.workspace = true
chrono.workspace = true
uuid.workspace = true
anyhow.workspace = true
tracing.workspace = true
```

```rust
// clipcrate/crates/db/src/lib.rs
pub mod models;
pub mod postgres;
pub mod clickhouse;
```

- [ ] **Step 4: Write Postgres connection pool helper**

```rust
// clipcrate/crates/db/src/postgres.rs
use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;
    Ok(pool)
}
```

- [ ] **Step 5: Write ClickHouse client stub**

```rust
// clipcrate/crates/db/src/clickhouse.rs
use anyhow::Result;

pub struct ClickHouseClient {
    client: clickhouse::Client,
}

impl ClickHouseClient {
    pub fn new(url: &str) -> Self {
        let client = clickhouse::Client::default().with_url(url);
        Self { client }
    }
}
```

- [ ] **Step 6: Verify crate compiles**

Run: `cd clipcrate && cargo check`
Expected: Successful compilation.

- [ ] **Step 7: Commit**

```bash
git add clipcrate/migrations/ clipcrate/crates/db/
git commit -m "feat: add Postgres schema, migrations, and db models"
```

---

### Task 3: Campaign CRUD API

**Files:**
- Modify: `clipcrate/crates/api/Cargo.toml`
- Create: `clipcrate/crates/api/src/lib.rs`
- Create: `clipcrate/crates/api/src/error.rs`
- Create: `clipcrate/crates/api/src/campaigns.rs`
- Create: `clipcrate/crates/api/src/auth.rs`
- Modify: `clipcrate/crates/db/src/postgres.rs` (add campaign queries)

- [ ] **Step 1: Write API error type**

```rust
// clipcrate/crates/api/src/error.rs
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("not found")]
    NotFound,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            ApiError::Internal(e) => {
                tracing::error!("internal error: {e:#}");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error".into())
            }
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
```

- [ ] **Step 2: Write auth middleware (Keycast UCAN stub)**

```rust
// clipcrate/crates/api/src/auth.rs
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use crate::error::ApiError;

/// Extracted from Keycast UCAN Bearer token in Authorization header.
/// For MVP, this validates the token format and extracts the pubkey.
/// Full UCAN signature validation to be integrated with keycast crate.
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
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(ApiError::Unauthorized)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(ApiError::Unauthorized)?;

        // TODO: Full Keycast UCAN validation (signature, expiry, tenant)
        // For now, treat the token as the pubkey for development
        if token.is_empty() {
            return Err(ApiError::Unauthorized);
        }

        Ok(AuthenticatedUser {
            pubkey: token.to_string(),
        })
    }
}

/// Service-to-service auth for internal endpoints.
/// Validates HMAC-signed token from clips-verifier.
#[derive(Debug, Clone)]
pub struct ServiceAuth;

impl<S> FromRequestParts<S> for ServiceAuth
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("x-service-token")
            .and_then(|v| v.to_str().ok())
            .ok_or(ApiError::Unauthorized)?;

        // TODO: HMAC validation against shared secret
        if auth_header.is_empty() {
            return Err(ApiError::Unauthorized);
        }

        Ok(ServiceAuth)
    }
}
```

- [ ] **Step 3: Add campaign query functions to postgres.rs**

```rust
// Append to clipcrate/crates/db/src/postgres.rs
use crate::models::Campaign;
use uuid::Uuid;

pub async fn create_campaign(
    pool: &sqlx::PgPool,
    creator_pubkey: &str,
    title: &str,
    budget_sats: i64,
    cpm_sats: i32,
    target_platforms: &[String],
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<Campaign> {
    let campaign = sqlx::query_as::<_, Campaign>(
        r#"
        INSERT INTO campaigns (creator_pubkey, title, budget_total_sats, budget_remaining_sats, cpm_sats, status, target_platforms, expires_at)
        VALUES ($1, $2, $3, $3, $4, 'pending', $5, $6)
        RETURNING *
        "#,
    )
    .bind(creator_pubkey)
    .bind(title)
    .bind(budget_sats)
    .bind(cpm_sats)
    .bind(target_platforms)
    .bind(expires_at)
    .fetch_one(pool)
    .await?;

    Ok(campaign)
}

pub async fn get_campaign(pool: &sqlx::PgPool, id: Uuid) -> Result<Option<Campaign>> {
    let campaign = sqlx::query_as::<_, Campaign>("SELECT * FROM campaigns WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(campaign)
}

pub async fn list_active_campaigns(pool: &sqlx::PgPool, limit: i64, offset: i64) -> Result<Vec<Campaign>> {
    let campaigns = sqlx::query_as::<_, Campaign>(
        "SELECT * FROM campaigns WHERE status = 'active' ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(campaigns)
}

pub async fn update_campaign_status(pool: &sqlx::PgPool, id: Uuid, creator_pubkey: &str, status: &str) -> Result<Option<Campaign>> {
    let campaign = sqlx::query_as::<_, Campaign>(
        r#"
        UPDATE campaigns SET status = $1, updated_at = NOW()
        WHERE id = $2 AND creator_pubkey = $3
        RETURNING *
        "#,
    )
    .bind(status)
    .bind(id)
    .bind(creator_pubkey)
    .fetch_optional(pool)
    .await?;
    Ok(campaign)
}
```

- [ ] **Step 4: Write campaign API handlers**

```rust
// clipcrate/crates/api/src/campaigns.rs
use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::AuthenticatedUser;
use crate::error::ApiError;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateCampaignRequest {
    pub title: String,
    pub budget_sats: i64,
    pub cpm_sats: i32,
    pub target_platforms: Vec<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub content_refs: Vec<String>,  // DiVine video "a" tag references
    pub guidelines: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 { 20 }

#[derive(Debug, Deserialize)]
pub struct UpdateCampaignRequest {
    pub status: Option<String>,
}

pub async fn create_campaign(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(req): Json<CreateCampaignRequest>,
) -> Result<Json<clipcrate_db::models::Campaign>, ApiError> {
    if req.budget_sats < 1000 {
        return Err(ApiError::BadRequest("minimum budget is 1000 sats".into()));
    }
    if req.cpm_sats < 1 {
        return Err(ApiError::BadRequest("cpm must be positive".into()));
    }

    let valid_platforms = ["tiktok", "instagram", "youtube", "x"];
    for p in &req.target_platforms {
        if !valid_platforms.contains(&p.as_str()) {
            return Err(ApiError::BadRequest(format!("invalid platform: {p}")));
        }
    }

    let campaign = clipcrate_db::postgres::create_campaign(
        &state.db,
        &user.pubkey,
        &req.title,
        req.budget_sats,
        req.cpm_sats,
        &req.target_platforms,
        req.expires_at,
    )
    .await
    .map_err(|e| ApiError::Internal(e))?;

    // TODO: Publish kind 30402 Nostr event to funnelcake
    // TODO: Generate Cashu deposit invoice

    Ok(Json(campaign))
}

pub async fn list_campaigns(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<clipcrate_db::models::Campaign>>, ApiError> {
    let campaigns = clipcrate_db::postgres::list_active_campaigns(&state.db, params.limit, params.offset)
        .await
        .map_err(|e| ApiError::Internal(e))?;
    Ok(Json(campaigns))
}

pub async fn get_campaign(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<clipcrate_db::models::Campaign>, ApiError> {
    let campaign = clipcrate_db::postgres::get_campaign(&state.db, id)
        .await
        .map_err(|e| ApiError::Internal(e))?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(campaign))
}

pub async fn update_campaign(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCampaignRequest>,
) -> Result<Json<clipcrate_db::models::Campaign>, ApiError> {
    if let Some(ref status) = req.status {
        let valid = ["active", "paused", "completed"];
        if !valid.contains(&status.as_str()) {
            return Err(ApiError::BadRequest(format!("invalid status: {status}")));
        }
    }

    let status = req.status.as_deref().unwrap_or("active");
    let campaign = clipcrate_db::postgres::update_campaign_status(&state.db, id, &user.pubkey, status)
        .await
        .map_err(|e| ApiError::Internal(e))?
        .ok_or(ApiError::NotFound)?;

    // TODO: Update kind 30402 Nostr event

    Ok(Json(campaign))
}
```

- [ ] **Step 5: Write router builder (lib.rs)**

```rust
// clipcrate/crates/api/src/lib.rs
pub mod auth;
pub mod campaigns;
pub mod error;
pub mod submissions;
pub mod wallet;
pub mod dashboard;
pub mod internal;
pub mod feed;

use axum::routing::{get, patch, post};
use axum::Router;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    // pub clickhouse: clipcrate_db::clickhouse::ClickHouseClient,
    // pub nostr_publisher: clipcrate_nostr::publisher::Publisher,
    // pub cashu_mint: clipcrate_cashu::mint::CashuMint,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        // Campaigns
        .route("/api/campaigns", post(campaigns::create_campaign))
        .route("/api/campaigns", get(campaigns::list_campaigns))
        .route("/api/campaigns/{id}", get(campaigns::get_campaign))
        .route("/api/campaigns/{id}", patch(campaigns::update_campaign))
        // TODO: Submissions, wallet, dashboard, internal, feed
        .with_state(state)
}
```

- [ ] **Step 6: Wire router into main.rs**

```rust
// clipcrate/clipcrate/src/main.rs
use anyhow::Result;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost:5432/clipcrate".into());

    let db = clipcrate_db::postgres::create_pool(&database_url).await?;
    tracing::info!("connected to postgres");

    let state = clipcrate_api::AppState { db };
    let app = clipcrate_api::router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3100").await?;
    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
```

- [ ] **Step 7: Update API crate Cargo.toml**

```toml
# clipcrate/crates/api/Cargo.toml
[package]
name = "clipcrate-api"
version = "0.1.0"
edition = "2021"

[dependencies]
clipcrate-db = { path = "../db" }

axum.workspace = true
tokio.workspace = true
tower-http.workspace = true
serde.workspace = true
serde_json.workspace = true
chrono.workspace = true
uuid.workspace = true
tracing.workspace = true
thiserror.workspace = true
anyhow.workspace = true
sqlx.workspace = true
```

- [ ] **Step 8: Create stub files for remaining API modules**

Create empty modules that lib.rs references:
- `clipcrate/crates/api/src/submissions.rs`
- `clipcrate/crates/api/src/wallet.rs`
- `clipcrate/crates/api/src/dashboard.rs`
- `clipcrate/crates/api/src/internal.rs`
- `clipcrate/crates/api/src/feed.rs`

Each just: `// TODO: implement`

- [ ] **Step 9: Verify compilation**

Run: `cd clipcrate && cargo check`
Expected: Successful compilation.

- [ ] **Step 10: Commit**

```bash
git add clipcrate/
git commit -m "feat: add campaign CRUD API with Axum handlers and auth middleware"
```

---

## Chunk 2: Submissions + Trust System

### Task 4: Submission API handlers

**Files:**
- Modify: `clipcrate/crates/db/src/postgres.rs` (add submission + clipper queries)
- Modify: `clipcrate/crates/api/src/submissions.rs`
- Modify: `clipcrate/crates/api/src/lib.rs` (add routes)

- [ ] **Step 1: Add submission + clipper queries to postgres.rs**

```rust
// Append to clipcrate/crates/db/src/postgres.rs
use crate::models::{Submission, Clipper};

pub async fn get_or_create_clipper(pool: &sqlx::PgPool, pubkey: &str) -> Result<Clipper> {
    let clipper = sqlx::query_as::<_, Clipper>(
        r#"
        INSERT INTO clippers (pubkey) VALUES ($1)
        ON CONFLICT (pubkey) DO UPDATE SET pubkey = EXCLUDED.pubkey
        RETURNING *
        "#,
    )
    .bind(pubkey)
    .fetch_one(pool)
    .await?;
    Ok(clipper)
}

pub async fn create_submission(
    pool: &sqlx::PgPool,
    campaign_id: Uuid,
    clipper_pubkey: &str,
    external_url: &str,
    platform: &str,
) -> Result<Submission> {
    let submission = sqlx::query_as::<_, Submission>(
        r#"
        INSERT INTO submissions (campaign_id, clipper_pubkey, external_url, platform)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
    )
    .bind(campaign_id)
    .bind(clipper_pubkey)
    .bind(external_url)
    .bind(platform)
    .fetch_one(pool)
    .await?;
    Ok(submission)
}

pub async fn list_clipper_submissions(pool: &sqlx::PgPool, clipper_pubkey: &str, limit: i64, offset: i64) -> Result<Vec<Submission>> {
    let subs = sqlx::query_as::<_, Submission>(
        "SELECT * FROM submissions WHERE clipper_pubkey = $1 ORDER BY submitted_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(clipper_pubkey)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(subs)
}

pub async fn get_submission(pool: &sqlx::PgPool, id: Uuid) -> Result<Option<Submission>> {
    let sub = sqlx::query_as::<_, Submission>("SELECT * FROM submissions WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(sub)
}

pub async fn count_active_submissions(pool: &sqlx::PgPool, clipper_pubkey: &str) -> Result<i64> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM submissions WHERE clipper_pubkey = $1 AND status IN ('pending', 'active')",
    )
    .bind(clipper_pubkey)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}
```

- [ ] **Step 2: Write submission handlers**

```rust
// clipcrate/crates/api/src/submissions.rs
use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::AuthenticatedUser;
use crate::error::ApiError;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateSubmissionRequest {
    pub campaign_id: Uuid,
    pub external_url: String,
    pub platform: String,
}

pub async fn create_submission(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(req): Json<CreateSubmissionRequest>,
) -> Result<Json<clipcrate_db::models::Submission>, ApiError> {
    // Validate platform
    let valid_platforms = ["tiktok", "instagram", "youtube", "x"];
    if !valid_platforms.contains(&req.platform.as_str()) {
        return Err(ApiError::BadRequest(format!("invalid platform: {}", req.platform)));
    }

    // Check campaign exists and is active
    let campaign = clipcrate_db::postgres::get_campaign(&state.db, req.campaign_id)
        .await
        .map_err(|e| ApiError::Internal(e))?
        .ok_or(ApiError::NotFound)?;

    if campaign.status != "active" {
        return Err(ApiError::BadRequest("campaign is not active".into()));
    }

    // Ensure clipper exists
    let clipper = clipcrate_db::postgres::get_or_create_clipper(&state.db, &user.pubkey)
        .await
        .map_err(|e| ApiError::Internal(e))?;

    // Check trust-based submission limits
    let active_count = clipcrate_db::postgres::count_active_submissions(&state.db, &user.pubkey)
        .await
        .map_err(|e| ApiError::Internal(e))?;

    let max_active = clipcrate_trust::max_active_submissions(clipper.trust_level);
    if active_count >= max_active as i64 {
        return Err(ApiError::BadRequest(format!(
            "trust level {} allows max {} active submissions",
            clipper.trust_level, max_active
        )));
    }

    // Create submission (unique constraint handles duplicate URL+campaign)
    let submission = clipcrate_db::postgres::create_submission(
        &state.db,
        req.campaign_id,
        &user.pubkey,
        &req.external_url,
        &req.platform,
    )
    .await
    .map_err(|e| {
        if let Some(db_err) = e.downcast_ref::<sqlx::Error>() {
            if let sqlx::Error::Database(ref pg_err) = db_err {
                if pg_err.code().as_deref() == Some("23505") {
                    return ApiError::Conflict("this URL is already submitted for this campaign".into());
                }
            }
        }
        ApiError::Internal(e)
    })?;

    // TODO: Publish kind 30403 Nostr event to funnelcake

    Ok(Json(submission))
}

pub async fn list_submissions(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Query(params): Query<super::campaigns::ListParams>,
) -> Result<Json<Vec<clipcrate_db::models::Submission>>, ApiError> {
    let subs = clipcrate_db::postgres::list_clipper_submissions(
        &state.db,
        &user.pubkey,
        params.limit,
        params.offset,
    )
    .await
    .map_err(|e| ApiError::Internal(e))?;
    Ok(Json(subs))
}

pub async fn get_submission(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<clipcrate_db::models::Submission>, ApiError> {
    let sub = clipcrate_db::postgres::get_submission(&state.db, id)
        .await
        .map_err(|e| ApiError::Internal(e))?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(sub))
}
```

- [ ] **Step 3: Add submission routes to lib.rs**

Add to the `router` function in `clipcrate/crates/api/src/lib.rs`:

```rust
        // Submissions
        .route("/api/submissions", post(submissions::create_submission))
        .route("/api/submissions", get(submissions::list_submissions))
        .route("/api/submissions/{id}", get(submissions::get_submission))
```

- [ ] **Step 4: Verify compilation**

Run: `cd clipcrate && cargo check`
Expected: Successful compilation.

- [ ] **Step 5: Commit**

```bash
git add clipcrate/
git commit -m "feat: add submission API with trust-based limits and duplicate detection"
```

---

### Task 5: Trust system

**Files:**
- Modify: `clipcrate/crates/trust/Cargo.toml`
- Create: `clipcrate/crates/trust/src/lib.rs`
- Create: `clipcrate/crates/trust/src/fraud.rs`

- [ ] **Step 1: Write trust level logic**

```rust
// clipcrate/crates/trust/src/lib.rs
pub mod fraud;

/// Trust level thresholds from spec:
/// Level 1 (new): max 50K views/week, 5 submissions
/// Level 2 (100K views + 30 days): max 500K views/week, 20 submissions
/// Level 3 (1M views + 90 days): unlimited
pub fn calculate_trust_level(total_verified_views: i64, account_age_days: i64) -> i32 {
    if total_verified_views >= 1_000_000 && account_age_days >= 90 {
        3
    } else if total_verified_views >= 100_000 && account_age_days >= 30 {
        2
    } else {
        1
    }
}

pub fn max_weekly_views(trust_level: i32) -> i64 {
    match trust_level {
        1 => 50_000,
        2 => 500_000,
        _ => i64::MAX,  // Level 3: unlimited
    }
}

pub fn max_active_submissions(trust_level: i32) -> i32 {
    match trust_level {
        1 => 5,
        2 => 20,
        _ => i32::MAX,  // Level 3: unlimited
    }
}

/// Hours to hold payout before making Cashu tokens mintable.
pub fn payout_hold_hours(trust_level: i32) -> i64 {
    match trust_level {
        1 => 48,
        2 => 24,
        _ => 0,  // Level 3: instant
    }
}
```

- [ ] **Step 2: Write fraud detection**

```rust
// clipcrate/crates/trust/src/fraud.rs
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct FraudFlag {
    pub flag_type: FraudFlagType,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum FraudFlagType {
    VelocitySpike,
    YoungAccount,
    SuspiciousPattern,
}

/// Check if view growth rate indicates bought views.
/// Flag if views grew >10x in under 6 hours.
pub fn check_velocity(
    prev_views: i64,
    current_views: i64,
    prev_time: DateTime<Utc>,
    current_time: DateTime<Utc>,
) -> Option<FraudFlag> {
    let hours = (current_time - prev_time).num_hours();
    if hours > 6 || hours == 0 {
        return None;
    }

    let prev = prev_views.max(1);
    let growth_ratio = current_views as f64 / prev as f64;

    if growth_ratio > 10.0 {
        Some(FraudFlag {
            flag_type: FraudFlagType::VelocitySpike,
            message: format!(
                "views grew {:.1}x in {}h ({} -> {})",
                growth_ratio, hours, prev_views, current_views
            ),
        })
    } else {
        None
    }
}

/// Check if view growth shows exact increments (bot signature).
pub fn check_suspicious_pattern(view_history: &[(i64, DateTime<Utc>)]) -> Option<FraudFlag> {
    if view_history.len() < 3 {
        return None;
    }

    let deltas: Vec<i64> = view_history
        .windows(2)
        .map(|w| w[1].0 - w[0].0)
        .collect();

    // Check if all deltas are identical (bot pattern)
    if deltas.len() >= 3 && deltas.iter().all(|&d| d == deltas[0] && d > 0) {
        Some(FraudFlag {
            flag_type: FraudFlagType::SuspiciousPattern,
            message: format!("steady exact increments of {} views", deltas[0]),
        })
    } else {
        None
    }
}
```

- [ ] **Step 3: Update trust crate Cargo.toml**

```toml
# clipcrate/crates/trust/Cargo.toml
[package]
name = "clipcrate-trust"
version = "0.1.0"
edition = "2021"

[dependencies]
chrono.workspace = true
serde.workspace = true
```

- [ ] **Step 4: Write tests for trust logic**

```rust
// Add to clipcrate/crates/trust/src/lib.rs at bottom
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_level_new_user() {
        assert_eq!(calculate_trust_level(0, 0), 1);
        assert_eq!(calculate_trust_level(50_000, 20), 1);
    }

    #[test]
    fn test_trust_level_2() {
        assert_eq!(calculate_trust_level(100_000, 30), 2);
        assert_eq!(calculate_trust_level(500_000, 60), 2);
        // Not enough days
        assert_eq!(calculate_trust_level(100_000, 29), 1);
        // Not enough views
        assert_eq!(calculate_trust_level(99_999, 30), 1);
    }

    #[test]
    fn test_trust_level_3() {
        assert_eq!(calculate_trust_level(1_000_000, 90), 3);
        assert_eq!(calculate_trust_level(5_000_000, 365), 3);
        // Not enough days
        assert_eq!(calculate_trust_level(1_000_000, 89), 2);
    }

    #[test]
    fn test_max_submissions() {
        assert_eq!(max_active_submissions(1), 5);
        assert_eq!(max_active_submissions(2), 20);
        assert_eq!(max_active_submissions(3), i32::MAX);
    }
}
```

- [ ] **Step 5: Run tests**

Run: `cd clipcrate && cargo test -p clipcrate-trust`
Expected: All tests pass.

- [ ] **Step 6: Commit**

```bash
git add clipcrate/crates/trust/
git commit -m "feat: add graduated trust system with fraud detection"
```

---

## Chunk 3: Nostr Events + Internal API

### Task 6: Nostr event builders

**Files:**
- Modify: `clipcrate/crates/nostr_events/Cargo.toml`
- Create: `clipcrate/crates/nostr_events/src/lib.rs`
- Create: `clipcrate/crates/nostr_events/src/campaign.rs`
- Create: `clipcrate/crates/nostr_events/src/submission.rs`
- Create: `clipcrate/crates/nostr_events/src/payout.rs`
- Create: `clipcrate/crates/nostr_events/src/publisher.rs`

- [ ] **Step 1: Update nostr_events crate Cargo.toml**

```toml
# clipcrate/crates/nostr_events/Cargo.toml
[package]
name = "clipcrate-nostr"
version = "0.1.0"
edition = "2021"

[dependencies]
nostr.workspace = true
nostr-sdk.workspace = true
serde.workspace = true
serde_json.workspace = true
chrono.workspace = true
uuid.workspace = true
anyhow.workspace = true
tracing.workspace = true
tokio.workspace = true
```

- [ ] **Step 2: Write campaign event builder**

```rust
// clipcrate/crates/nostr_events/src/campaign.rs
use nostr::prelude::*;

/// Build a kind 30402 (NIP-15 classified listing) campaign event.
pub fn build_campaign_event(
    title: &str,
    summary: &str,
    content_refs: &[String],  // "34236:<pubkey>:<d-tag>" format
    cpm_sats: i32,
    budget_sats: i64,
    target_platforms: &[String],
    expires_at: Option<u64>,
    campaign_id: &str,
    guidelines: Option<&str>,
) -> UnsignedEvent {
    let mut tags = vec![
        Tag::custom(TagKind::custom("d"), vec![campaign_id.to_string()]),
        Tag::custom(TagKind::custom("title"), vec![title.to_string()]),
        Tag::custom(TagKind::custom("summary"), vec![summary.to_string()]),
        Tag::custom(
            TagKind::custom("price"),
            vec![cpm_sats.to_string(), "sats".into(), "per_1000_views".into()],
        ),
        Tag::custom(TagKind::custom("budget"), vec![budget_sats.to_string()]),
        Tag::custom(TagKind::custom("status"), vec!["active".into()]),
        Tag::custom(TagKind::custom("L"), vec!["divine-clips".into()]),
        Tag::custom(TagKind::custom("l"), vec!["campaign".into(), "divine-clips".into()]),
    ];

    for content_ref in content_refs {
        tags.push(Tag::custom(TagKind::custom("a"), vec![content_ref.clone()]));
    }

    for platform in target_platforms {
        tags.push(Tag::custom(TagKind::custom("t"), vec![platform.clone()]));
    }

    if let Some(exp) = expires_at {
        tags.push(Tag::custom(TagKind::custom("expiration"), vec![exp.to_string()]));
    }

    let content = guidelines.unwrap_or("").to_string();
    UnsignedEvent::new(Kind::from(30402), content, tags)
}
```

- [ ] **Step 3: Write submission event builder**

```rust
// clipcrate/crates/nostr_events/src/submission.rs
use nostr::prelude::*;

/// Build a kind 30403 (custom parameterized replaceable) submission event.
pub fn build_submission_event(
    submission_id: &str,
    campaign_ref: &str,  // "30402:<creator-pubkey>:<campaign-d-tag>"
    external_url: &str,
    platform: &str,
) -> UnsignedEvent {
    let tags = vec![
        Tag::custom(TagKind::custom("d"), vec![submission_id.to_string()]),
        Tag::custom(TagKind::custom("a"), vec![campaign_ref.to_string()]),
        Tag::custom(TagKind::custom("r"), vec![external_url.to_string()]),
        Tag::custom(TagKind::custom("platform"), vec![platform.to_string()]),
        Tag::custom(TagKind::custom("status"), vec!["pending".into()]),
        Tag::custom(TagKind::custom("L"), vec!["divine-clips".into()]),
        Tag::custom(TagKind::custom("l"), vec!["submission".into(), "divine-clips".into()]),
    ];

    UnsignedEvent::new(Kind::from(30403), String::new(), tags)
}
```

- [ ] **Step 4: Write payout event builder**

```rust
// clipcrate/crates/nostr_events/src/payout.rs
use nostr::prelude::*;

/// Build a kind 9734-style payout receipt event.
pub fn build_payout_event(
    clipper_pubkey: &str,
    submission_ref: &str,
    campaign_ref: &str,
    amount_msats: u64,
    description: &str,
) -> UnsignedEvent {
    let tags = vec![
        Tag::custom(TagKind::custom("p"), vec![clipper_pubkey.to_string()]),
        Tag::custom(TagKind::custom("a"), vec![submission_ref.to_string()]),
        Tag::custom(TagKind::custom("a"), vec![campaign_ref.to_string()]),
        Tag::custom(TagKind::custom("amount"), vec![amount_msats.to_string()]),
    ];

    UnsignedEvent::new(Kind::from(9734), description.to_string(), tags)
}
```

- [ ] **Step 5: Write publisher (WebSocket to funnelcake)**

```rust
// clipcrate/crates/nostr_events/src/publisher.rs
use anyhow::Result;
use nostr_sdk::prelude::*;
use tracing;

pub struct Publisher {
    client: Client,
}

impl Publisher {
    pub async fn new(relay_url: &str, keys: Keys) -> Result<Self> {
        let client = Client::new(keys);
        client.add_relay(relay_url).await?;
        client.connect().await;
        tracing::info!("connected to relay: {relay_url}");
        Ok(Self { client })
    }

    pub async fn publish(&self, event: Event) -> Result<()> {
        self.client.send_event(event).await?;
        Ok(())
    }
}
```

- [ ] **Step 6: Write lib.rs**

```rust
// clipcrate/crates/nostr_events/src/lib.rs
pub mod campaign;
pub mod submission;
pub mod payout;
pub mod publisher;
```

- [ ] **Step 7: Verify compilation**

Run: `cd clipcrate && cargo check`
Expected: Successful compilation.

- [ ] **Step 8: Commit**

```bash
git add clipcrate/crates/nostr_events/
git commit -m "feat: add Nostr event builders for campaigns, submissions, and payouts"
```

---

### Task 7: Internal API (verification + payout endpoints)

**Files:**
- Modify: `clipcrate/crates/api/src/internal.rs`
- Modify: `clipcrate/crates/db/src/postgres.rs` (add verification/payout queries)
- Modify: `clipcrate/crates/db/src/clickhouse.rs` (add verification snapshot writes)
- Modify: `clipcrate/crates/api/src/lib.rs` (add routes)

- [ ] **Step 1: Add verification + payout queries to postgres.rs**

```rust
// Append to clipcrate/crates/db/src/postgres.rs
use crate::models::Payout;

pub async fn list_pending_submissions(pool: &sqlx::PgPool) -> Result<Vec<Submission>> {
    let subs = sqlx::query_as::<_, Submission>(
        r#"
        SELECT * FROM submissions
        WHERE status IN ('pending', 'active')
        AND submitted_at > NOW() - INTERVAL '30 days'
        ORDER BY submitted_at ASC
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(subs)
}

pub async fn update_submission_views(
    pool: &sqlx::PgPool,
    id: Uuid,
    views: i64,
) -> Result<Option<Submission>> {
    let sub = sqlx::query_as::<_, Submission>(
        r#"
        UPDATE submissions
        SET total_verified_views = $1, last_verified_at = NOW(), status = 'active'
        WHERE id = $2
        RETURNING *
        "#,
    )
    .bind(views)
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(sub)
}

pub async fn create_payout(
    pool: &sqlx::PgPool,
    submission_id: Uuid,
    campaign_id: Uuid,
    clipper_pubkey: &str,
    amount_sats: i64,
    views_at_payout: i64,
) -> Result<Payout> {
    let payout = sqlx::query_as::<_, Payout>(
        r#"
        INSERT INTO payouts (submission_id, campaign_id, clipper_pubkey, amount_sats, views_at_payout)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(submission_id)
    .bind(campaign_id)
    .bind(clipper_pubkey)
    .bind(amount_sats)
    .bind(views_at_payout)
    .fetch_one(pool)
    .await?;
    Ok(payout)
}

pub async fn deduct_campaign_budget(pool: &sqlx::PgPool, campaign_id: Uuid, amount_sats: i64) -> Result<Option<Campaign>> {
    let campaign = sqlx::query_as::<_, Campaign>(
        r#"
        UPDATE campaigns
        SET budget_remaining_sats = GREATEST(budget_remaining_sats - $1, 0),
            status = CASE WHEN budget_remaining_sats - $1 <= 0 THEN 'exhausted' ELSE status END,
            updated_at = NOW()
        WHERE id = $2 AND budget_remaining_sats > 0
        RETURNING *
        "#,
    )
    .bind(amount_sats)
    .bind(campaign_id)
    .fetch_optional(pool)
    .await?;
    Ok(campaign)
}
```

- [ ] **Step 2: Add ClickHouse verification snapshot write**

```rust
// Replace clipcrate/crates/db/src/clickhouse.rs
use anyhow::Result;
use clickhouse::Row;
use serde::Serialize;
use uuid::Uuid;

pub struct ClickHouseClient {
    client: clickhouse::Client,
}

#[derive(Debug, Row, Serialize)]
pub struct VerificationSnapshot {
    pub submission_id: Uuid,
    pub campaign_id: Uuid,
    pub clipper_pubkey: String,
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub timestamp: time::OffsetDateTime,
    pub view_count: u64,
    pub source: String,
    pub fraud_score: f32,
}

impl ClickHouseClient {
    pub fn new(url: &str) -> Self {
        let client = clickhouse::Client::default().with_url(url);
        Self { client }
    }

    pub async fn insert_verification(&self, snapshot: VerificationSnapshot) -> Result<()> {
        let mut insert = self.client.insert("verification_snapshots")?;
        insert.write(&snapshot).await?;
        insert.end().await?;
        Ok(())
    }
}
```

- [ ] **Step 3: Write internal API handlers**

```rust
// clipcrate/crates/api/src/internal.rs
use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::ServiceAuth;
use crate::error::ApiError;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct VerificationResult {
    pub submission_id: Uuid,
    pub view_count: i64,
    pub source: String,  // youtube_api, tiktok_api, phyllo, manual
    pub fraud_score: f32,
}

#[derive(Debug, Deserialize)]
pub struct VerificationBatch {
    pub results: Vec<VerificationResult>,
}

pub async fn list_pending_submissions(
    State(state): State<AppState>,
    _auth: ServiceAuth,
) -> Result<Json<Vec<clipcrate_db::models::Submission>>, ApiError> {
    let subs = clipcrate_db::postgres::list_pending_submissions(&state.db)
        .await
        .map_err(|e| ApiError::Internal(e))?;
    Ok(Json(subs))
}

pub async fn post_verifications(
    State(state): State<AppState>,
    _auth: ServiceAuth,
    Json(batch): Json<VerificationBatch>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut processed = 0;

    for result in batch.results {
        // Update submission view count in Postgres
        let sub = clipcrate_db::postgres::update_submission_views(
            &state.db,
            result.submission_id,
            result.view_count,
        )
        .await
        .map_err(|e| ApiError::Internal(e))?;

        let Some(sub) = sub else { continue };

        // TODO: Write ClickHouse snapshot when CH client is wired into AppState
        // TODO: Check if payout threshold crossed, trigger payout

        // Calculate payout if views earned new sats
        let campaign = clipcrate_db::postgres::get_campaign(&state.db, sub.campaign_id)
            .await
            .map_err(|e| ApiError::Internal(e))?;

        if let Some(campaign) = campaign {
            let earned_sats = (result.view_count as f64 / 1000.0 * campaign.cpm_sats as f64) as i64;
            let unpaid_sats = earned_sats - sub.total_paid_sats;

            if unpaid_sats > 0 && campaign.budget_remaining_sats > 0 {
                let payout_amount = unpaid_sats.min(campaign.budget_remaining_sats);

                clipcrate_db::postgres::deduct_campaign_budget(&state.db, campaign.id, payout_amount)
                    .await
                    .map_err(|e| ApiError::Internal(e))?;

                clipcrate_db::postgres::create_payout(
                    &state.db,
                    sub.id,
                    campaign.id,
                    &sub.clipper_pubkey,
                    payout_amount,
                    result.view_count,
                )
                .await
                .map_err(|e| ApiError::Internal(e))?;

                // Update submission's total_paid_sats
                sqlx::query("UPDATE submissions SET total_paid_sats = total_paid_sats + $1 WHERE id = $2")
                    .bind(payout_amount)
                    .bind(sub.id)
                    .execute(&state.db)
                    .await
                    .map_err(|e| ApiError::Internal(anyhow::anyhow!(e)))?;

                // TODO: Mint Cashu tokens for clipper
                // TODO: Publish payout Nostr event
            }
        }

        processed += 1;
    }

    Ok(Json(serde_json::json!({ "processed": processed })))
}
```

- [ ] **Step 4: Add internal routes to lib.rs**

```rust
        // Internal (service-to-service)
        .route("/api/internal/submissions", get(internal::list_pending_submissions))
        .route("/api/internal/verifications", post(internal::post_verifications))
```

- [ ] **Step 5: Verify compilation**

Run: `cd clipcrate && cargo check`
Expected: Successful compilation.

- [ ] **Step 6: Commit**

```bash
git add clipcrate/
git commit -m "feat: add internal verification API with payout calculation and budget deduction"
```

---

## Chunk 4: Cashu Mint + Wallet + Dashboard

### Task 8: Cashu mint integration

**Files:**
- Modify: `clipcrate/crates/cashu/Cargo.toml`
- Create: `clipcrate/crates/cashu/src/lib.rs`
- Create: `clipcrate/crates/cashu/src/mint.rs`
- Create: `clipcrate/crates/cashu/src/wallet.rs`

- [ ] **Step 1: Update Cargo.toml**

Note: Moksha is a Rust Cashu mint implementation. If moksha-core is not available on crates.io, use cdk (Cashu Development Kit) instead — `cdk = "0.6"` and `cdk-axum = "0.6"`. Check availability before proceeding.

```toml
# clipcrate/crates/cashu/Cargo.toml
[package]
name = "clipcrate-cashu"
version = "0.1.0"
edition = "2021"

[dependencies]
# Try moksha-core first, fall back to cdk if unavailable
# moksha-core = "0.4"
# OR:
# cdk = "0.6"
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
tracing.workspace = true
uuid.workspace = true
```

- [ ] **Step 2: Write mint abstraction**

```rust
// clipcrate/crates/cashu/src/mint.rs
use anyhow::Result;
use uuid::Uuid;

/// Abstraction over Cashu mint operations.
/// MVP: tracks balances in Postgres, real Cashu mint integration comes later.
/// This allows the full flow to work while Moksha/CDK integration is developed.
pub struct CashuMint {
    // In production: moksha_core::Mint or cdk::Mint
    // For MVP: balance tracking only (IOU model)
}

impl CashuMint {
    pub fn new() -> Self {
        Self {}
    }

    /// Generate a Lightning invoice for campaign deposit.
    /// Returns (invoice_string, payment_hash).
    pub async fn create_deposit_invoice(&self, amount_sats: u64) -> Result<(String, String)> {
        // TODO: Generate real LN invoice via mint
        // MVP: return placeholder
        tracing::warn!("cashu mint: using placeholder invoice (MVP mode)");
        Ok(("lnbc...placeholder".into(), Uuid::new_v4().to_string()))
    }

    /// Confirm deposit and lock tokens to campaign.
    pub async fn confirm_deposit(&self, _payment_hash: &str, _campaign_id: Uuid) -> Result<()> {
        // TODO: Verify LN payment, issue ecash tokens locked to campaign
        tracing::info!("cashu mint: deposit confirmed (MVP: balance-only)");
        Ok(())
    }

    /// Issue tokens to clipper from campaign pool.
    pub async fn issue_payout(&self, campaign_id: Uuid, clipper_pubkey: &str, amount_sats: u64) -> Result<String> {
        // TODO: Transfer ecash from campaign pool to clipper
        // Returns cashu token ID
        tracing::info!(
            campaign_id = %campaign_id,
            clipper = clipper_pubkey,
            amount = amount_sats,
            "cashu mint: issued payout (MVP: balance-only)"
        );
        Ok(Uuid::new_v4().to_string())
    }

    /// Melt tokens and pay Lightning invoice.
    pub async fn withdraw(&self, _clipper_pubkey: &str, _invoice: &str, _amount_sats: u64) -> Result<String> {
        // TODO: Melt ecash tokens, pay LN invoice
        // Returns payment preimage
        tracing::info!("cashu mint: withdrawal processed (MVP: placeholder)");
        Ok("preimage_placeholder".into())
    }
}
```

- [ ] **Step 3: Write wallet balance tracker**

```rust
// clipcrate/crates/cashu/src/wallet.rs
use anyhow::Result;
use sqlx::PgPool;

/// Query clipper's available balance (earned - withdrawn).
pub async fn get_balance(pool: &PgPool, clipper_pubkey: &str) -> Result<i64> {
    let row: (i64,) = sqlx::query_as(
        r#"
        SELECT COALESCE(
            (SELECT SUM(amount_sats) FROM payouts WHERE clipper_pubkey = $1),
            0
        ) - COALESCE(
            (SELECT SUM(amount_sats) FROM withdrawals WHERE clipper_pubkey = $1),
            0
        )
        "#,
    )
    .bind(clipper_pubkey)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}
```

Note: This requires a `withdrawals` table. Add to migrations:

```sql
-- Add to migration: withdrawals tracking
CREATE TABLE withdrawals (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    clipper_pubkey TEXT NOT NULL,
    amount_sats BIGINT NOT NULL,
    lightning_invoice TEXT,
    payment_preimage TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT valid_withdrawal_status CHECK (status IN ('pending', 'completed', 'failed'))
);
```

- [ ] **Step 4: Write lib.rs**

```rust
// clipcrate/crates/cashu/src/lib.rs
pub mod mint;
pub mod wallet;
```

- [ ] **Step 5: Verify compilation**

Run: `cd clipcrate && cargo check`
Expected: Successful compilation.

- [ ] **Step 6: Commit**

```bash
git add clipcrate/crates/cashu/ clipcrate/migrations/
git commit -m "feat: add Cashu mint abstraction with balance-only MVP mode"
```

---

### Task 9: Wallet + Dashboard API endpoints

**Files:**
- Modify: `clipcrate/crates/api/src/wallet.rs`
- Modify: `clipcrate/crates/api/src/dashboard.rs`
- Modify: `clipcrate/crates/api/src/lib.rs` (add routes)

- [ ] **Step 1: Write wallet handlers**

```rust
// clipcrate/crates/api/src/wallet.rs
use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::auth::AuthenticatedUser;
use crate::error::ApiError;
use crate::AppState;

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

pub async fn get_balance(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<BalanceResponse>, ApiError> {
    let balance = clipcrate_cashu::wallet::get_balance(&state.db, &user.pubkey)
        .await
        .map_err(|e| ApiError::Internal(e))?;

    Ok(Json(BalanceResponse { balance_sats: balance }))
}

pub async fn withdraw(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(req): Json<WithdrawRequest>,
) -> Result<Json<WithdrawResponse>, ApiError> {
    if req.amount_sats <= 0 {
        return Err(ApiError::BadRequest("amount must be positive".into()));
    }

    let balance = clipcrate_cashu::wallet::get_balance(&state.db, &user.pubkey)
        .await
        .map_err(|e| ApiError::Internal(e))?;

    if req.amount_sats > balance {
        return Err(ApiError::BadRequest(format!(
            "insufficient balance: {} sats available",
            balance
        )));
    }

    // TODO: Call cashu mint withdraw, create withdrawal record

    Ok(Json(WithdrawResponse {
        status: "pending".into(),
        amount_sats: req.amount_sats,
    }))
}
```

- [ ] **Step 2: Write dashboard handler**

```rust
// clipcrate/crates/api/src/dashboard.rs
use axum::extract::State;
use axum::Json;
use serde::Serialize;

use crate::auth::AuthenticatedUser;
use crate::error::ApiError;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct DashboardResponse {
    pub trust_level: i32,
    pub total_verified_views: i64,
    pub total_earned_sats: i64,
    pub balance_sats: i64,
    pub active_submissions: i64,
    pub weekly_views_used: i64,
    pub weekly_views_limit: i64,
}

pub async fn get_dashboard(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<DashboardResponse>, ApiError> {
    let clipper = clipcrate_db::postgres::get_or_create_clipper(&state.db, &user.pubkey)
        .await
        .map_err(|e| ApiError::Internal(e))?;

    let active = clipcrate_db::postgres::count_active_submissions(&state.db, &user.pubkey)
        .await
        .map_err(|e| ApiError::Internal(e))?;

    let balance = clipcrate_cashu::wallet::get_balance(&state.db, &user.pubkey)
        .await
        .map_err(|e| ApiError::Internal(e))?;

    Ok(Json(DashboardResponse {
        trust_level: clipper.trust_level,
        total_verified_views: clipper.total_verified_views,
        total_earned_sats: clipper.total_earned_sats,
        balance_sats: balance,
        active_submissions: active,
        weekly_views_used: clipper.weekly_views_used,
        weekly_views_limit: clipcrate_trust::max_weekly_views(clipper.trust_level),
    }))
}
```

- [ ] **Step 3: Add routes to lib.rs**

```rust
        // Wallet
        .route("/api/wallet/balance", get(wallet::get_balance))
        .route("/api/wallet/withdraw", post(wallet::withdraw))
        // Dashboard
        .route("/api/dashboard", get(dashboard::get_dashboard))
```

- [ ] **Step 4: Verify compilation**

Run: `cd clipcrate && cargo check`
Expected: Successful compilation.

- [ ] **Step 5: Commit**

```bash
git add clipcrate/
git commit -m "feat: add wallet balance/withdraw and clipper dashboard endpoints"
```

---

## Chunk 5: Platform APIs + clips-verifier CF Worker

### Task 10: Platform API clients (Rust shared crate)

**Files:**
- Modify: `clipcrate/crates/platform-apis/Cargo.toml`
- Create: `clipcrate/crates/platform-apis/src/lib.rs`
- Create: `clipcrate/crates/platform-apis/src/youtube.rs`
- Create: `clipcrate/crates/platform-apis/src/tiktok.rs`

- [ ] **Step 1: Update Cargo.toml**

```toml
# clipcrate/crates/platform-apis/Cargo.toml
[package]
name = "clipcrate-platform-apis"
version = "0.1.0"
edition = "2021"

[dependencies]
reqwest = { version = "0.12", features = ["json"] }
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
tracing.workspace = true
url = "2"
```

- [ ] **Step 2: Write platform trait and types**

```rust
// clipcrate/crates/platform-apis/src/lib.rs
pub mod youtube;
pub mod tiktok;
pub mod instagram;
pub mod twitter;

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct ViewCount {
    pub view_count: u64,
    pub source: String,
}

/// Trait for platform API clients.
pub trait PlatformClient {
    /// Extract video ID from URL.
    fn extract_video_id(url: &str) -> Option<String>;

    /// Get view count for a video URL.
    async fn get_view_count(&self, url: &str) -> Result<ViewCount>;
}
```

- [ ] **Step 3: Write YouTube client**

```rust
// clipcrate/crates/platform-apis/src/youtube.rs
use anyhow::{anyhow, Result};
use crate::ViewCount;

pub struct YouTubeClient {
    api_key: String,
    http: reqwest::Client,
}

impl YouTubeClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            http: reqwest::Client::new(),
        }
    }

    pub fn extract_video_id(url: &str) -> Option<String> {
        let url = url::Url::parse(url).ok()?;
        match url.host_str()? {
            "youtube.com" | "www.youtube.com" => {
                if url.path() == "/watch" {
                    url.query_pairs().find(|(k, _)| k == "v").map(|(_, v)| v.to_string())
                } else if url.path().starts_with("/shorts/") {
                    url.path().strip_prefix("/shorts/").map(|s| s.to_string())
                } else {
                    None
                }
            }
            "youtu.be" => url.path().strip_prefix('/').map(|s| s.to_string()),
            _ => None,
        }
    }

    pub async fn get_view_count(&self, url: &str) -> Result<ViewCount> {
        let video_id = Self::extract_video_id(url)
            .ok_or_else(|| anyhow!("could not extract YouTube video ID from URL"))?;

        let resp: serde_json::Value = self.http
            .get("https://www.googleapis.com/youtube/v3/videos")
            .query(&[
                ("part", "statistics"),
                ("id", &video_id),
                ("key", &self.api_key),
            ])
            .send()
            .await?
            .json()
            .await?;

        let views = resp["items"][0]["statistics"]["viewCount"]
            .as_str()
            .and_then(|s| s.parse::<u64>().ok())
            .ok_or_else(|| anyhow!("could not parse view count from YouTube API response"))?;

        Ok(ViewCount {
            view_count: views,
            source: "youtube_api".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_youtube_id() {
        assert_eq!(
            YouTubeClient::extract_video_id("https://www.youtube.com/watch?v=abc123"),
            Some("abc123".into())
        );
        assert_eq!(
            YouTubeClient::extract_video_id("https://youtu.be/abc123"),
            Some("abc123".into())
        );
        assert_eq!(
            YouTubeClient::extract_video_id("https://youtube.com/shorts/abc123"),
            Some("abc123".into())
        );
        assert_eq!(
            YouTubeClient::extract_video_id("https://tiktok.com/something"),
            None
        );
    }
}
```

- [ ] **Step 4: Write TikTok oEmbed client**

```rust
// clipcrate/crates/platform-apis/src/tiktok.rs
use anyhow::{anyhow, Result};
use crate::ViewCount;

pub struct TikTokClient {
    http: reqwest::Client,
}

impl TikTokClient {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
        }
    }

    pub fn extract_video_id(url: &str) -> Option<String> {
        let url = url::Url::parse(url).ok()?;
        if url.host_str()? != "tiktok.com" && url.host_str()? != "www.tiktok.com" {
            return None;
        }
        // URL format: tiktok.com/@user/video/1234567890
        let path = url.path();
        if let Some(pos) = path.find("/video/") {
            Some(path[pos + 7..].to_string())
        } else {
            None
        }
    }

    pub async fn get_view_count(&self, url: &str) -> Result<ViewCount> {
        // TikTok oEmbed doesn't return view counts directly,
        // but we can get video existence and basic metadata.
        // For actual view counts, Phyllo fallback is needed.
        let resp: serde_json::Value = self.http
            .get("https://www.tiktok.com/oembed")
            .query(&[("url", url)])
            .send()
            .await?
            .json()
            .await?;

        // oEmbed confirms video exists but doesn't give view count
        if resp.get("title").is_none() {
            return Err(anyhow!("TikTok video not found or not accessible"));
        }

        // Return 0 views — actual count requires Phyllo or Display API
        Ok(ViewCount {
            view_count: 0,
            source: "tiktok_oembed".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tiktok_id() {
        assert_eq!(
            TikTokClient::extract_video_id("https://tiktok.com/@user/video/1234567890"),
            Some("1234567890".into())
        );
        assert_eq!(
            TikTokClient::extract_video_id("https://youtube.com/watch?v=abc"),
            None
        );
    }
}
```

- [ ] **Step 5: Create stub files for instagram.rs and twitter.rs**

```rust
// clipcrate/crates/platform-apis/src/instagram.rs
// Instagram requires Business account OAuth — Phyllo-only for MVP.

// clipcrate/crates/platform-apis/src/twitter.rs
// X API v2 requires Bearer token — Phyllo fallback for MVP.
```

- [ ] **Step 6: Run tests**

Run: `cd clipcrate && cargo test -p clipcrate-platform-apis`
Expected: YouTube and TikTok URL extraction tests pass.

- [ ] **Step 7: Commit**

```bash
git add clipcrate/crates/platform-apis/
git commit -m "feat: add YouTube and TikTok platform API clients with URL extraction"
```

---

### Task 11: clips-verifier CF Worker

**Files:**
- Create: `clips-verifier/package.json`
- Create: `clips-verifier/wrangler.toml`
- Create: `clips-verifier/tsconfig.json`
- Create: `clips-verifier/vitest.config.ts`
- Create: `clips-verifier/src/index.ts`
- Create: `clips-verifier/src/clipcrate-client.ts`
- Create: `clips-verifier/src/platforms/youtube.ts`
- Create: `clips-verifier/src/platforms/tiktok.ts`
- Create: `clips-verifier/src/platforms/types.ts`
- Create: `clips-verifier/src/phyllo.ts`
- Create: `clips-verifier/src/fraud.ts`

- [ ] **Step 1: Initialize CF Worker project**

```json
// clips-verifier/package.json
{
  "name": "clips-verifier",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "dev": "wrangler dev",
    "deploy": "wrangler deploy",
    "test": "vitest run"
  },
  "devDependencies": {
    "@cloudflare/workers-types": "^4",
    "typescript": "^5",
    "vitest": "^2",
    "wrangler": "^4"
  }
}
```

```toml
# clips-verifier/wrangler.toml
name = "clips-verifier"
main = "src/index.ts"
compatibility_date = "2025-01-01"

[triggers]
crons = ["0 */6 * * *"]  # Every 6 hours

[vars]
CLIPCRATE_API_URL = "https://clipcrate.divine.video"
```

```json
// clips-verifier/tsconfig.json
{
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "types": ["@cloudflare/workers-types"]
  }
}
```

- [ ] **Step 2: Write shared types**

```typescript
// clips-verifier/src/platforms/types.ts
export interface ViewCountResult {
  viewCount: number;
  source: string;
}

export interface Submission {
  id: string;
  campaign_id: string;
  clipper_pubkey: string;
  external_url: string;
  platform: string;
  total_verified_views: number;
}

export interface VerificationResult {
  submission_id: string;
  view_count: number;
  source: string;
  fraud_score: number;
}
```

- [ ] **Step 3: Write clipcrate API client**

```typescript
// clips-verifier/src/clipcrate-client.ts
import type { Submission, VerificationResult } from "./platforms/types";

export class ClipcrateClient {
  constructor(
    private baseUrl: string,
    private serviceToken: string,
  ) {}

  async getPendingSubmissions(): Promise<Submission[]> {
    const resp = await fetch(`${this.baseUrl}/api/internal/submissions`, {
      headers: { "x-service-token": this.serviceToken },
    });
    if (!resp.ok) throw new Error(`clipcrate API error: ${resp.status}`);
    return resp.json();
  }

  async postVerifications(results: VerificationResult[]): Promise<void> {
    const resp = await fetch(`${this.baseUrl}/api/internal/verifications`, {
      method: "POST",
      headers: {
        "x-service-token": this.serviceToken,
        "content-type": "application/json",
      },
      body: JSON.stringify({ results }),
    });
    if (!resp.ok) throw new Error(`clipcrate verification post error: ${resp.status}`);
  }
}
```

- [ ] **Step 4: Write YouTube API client (TypeScript)**

```typescript
// clips-verifier/src/platforms/youtube.ts
import type { ViewCountResult } from "./types";

export function extractYouTubeId(url: string): string | null {
  try {
    const u = new URL(url);
    if (u.hostname === "youtu.be") return u.pathname.slice(1);
    if (u.hostname.includes("youtube.com")) {
      if (u.pathname === "/watch") return u.searchParams.get("v");
      if (u.pathname.startsWith("/shorts/")) return u.pathname.slice(8);
    }
  } catch {}
  return null;
}

export async function getYouTubeViews(
  url: string,
  apiKey: string,
): Promise<ViewCountResult | null> {
  const videoId = extractYouTubeId(url);
  if (!videoId) return null;

  const resp = await fetch(
    `https://www.googleapis.com/youtube/v3/videos?part=statistics&id=${videoId}&key=${apiKey}`,
  );
  if (!resp.ok) return null;

  const data: any = await resp.json();
  const viewCount = parseInt(data.items?.[0]?.statistics?.viewCount ?? "0", 10);

  return { viewCount, source: "youtube_api" };
}
```

- [ ] **Step 5: Write TikTok oEmbed client (TypeScript)**

```typescript
// clips-verifier/src/platforms/tiktok.ts
import type { ViewCountResult } from "./types";

export async function getTikTokViews(url: string): Promise<ViewCountResult | null> {
  // TikTok oEmbed confirms existence but doesn't return view counts
  const resp = await fetch(
    `https://www.tiktok.com/oembed?url=${encodeURIComponent(url)}`,
  );
  if (!resp.ok) return null;

  const data: any = await resp.json();
  if (!data.title) return null;

  // oEmbed doesn't give views — return 0, Phyllo will fill in
  return { viewCount: 0, source: "tiktok_oembed" };
}
```

- [ ] **Step 6: Write Phyllo fallback client**

```typescript
// clips-verifier/src/phyllo.ts
import type { ViewCountResult } from "./platforms/types";

export class PhylloClient {
  constructor(
    private clientId: string,
    private clientSecret: string,
  ) {}

  async getViewCount(
    _externalUrl: string,
    _platform: string,
  ): Promise<ViewCountResult | null> {
    // TODO: Implement Phyllo API integration
    // 1. Look up clipper's connected account for this platform
    // 2. Query Phyllo for content metrics
    // 3. Return view count
    console.log("phyllo: fallback not yet implemented");
    return null;
  }
}
```

- [ ] **Step 7: Write fraud checks**

```typescript
// clips-verifier/src/fraud.ts
export function calculateFraudScore(
  currentViews: number,
  previousViews: number,
  hoursSinceLastCheck: number,
): number {
  if (hoursSinceLastCheck <= 0 || previousViews <= 0) return 0;

  const growthRatio = currentViews / previousViews;

  // >10x growth in <6h is suspicious
  if (hoursSinceLastCheck <= 6 && growthRatio > 10) {
    return 0.8;
  }

  // >5x growth in <6h is mildly suspicious
  if (hoursSinceLastCheck <= 6 && growthRatio > 5) {
    return 0.4;
  }

  return 0;
}
```

- [ ] **Step 8: Write main scheduled handler**

```typescript
// clips-verifier/src/index.ts
import { ClipcrateClient } from "./clipcrate-client";
import { getYouTubeViews } from "./platforms/youtube";
import { getTikTokViews } from "./platforms/tiktok";
import { PhylloClient } from "./phyllo";
import { calculateFraudScore } from "./fraud";
import type { VerificationResult } from "./platforms/types";

export interface Env {
  CLIPCRATE_API_URL: string;
  CLIPCRATE_SERVICE_TOKEN: string;
  YOUTUBE_API_KEY: string;
  PHYLLO_CLIENT_ID: string;
  PHYLLO_CLIENT_SECRET: string;
}

export default {
  async scheduled(event: ScheduledEvent, env: Env, ctx: ExecutionContext) {
    const client = new ClipcrateClient(env.CLIPCRATE_API_URL, env.CLIPCRATE_SERVICE_TOKEN);
    const phyllo = new PhylloClient(env.PHYLLO_CLIENT_ID, env.PHYLLO_CLIENT_SECRET);

    const submissions = await client.getPendingSubmissions();
    console.log(`verifying ${submissions.length} submissions`);

    const results: VerificationResult[] = [];

    for (const sub of submissions) {
      try {
        let viewResult = null;

        // Try public API first
        switch (sub.platform) {
          case "youtube":
            viewResult = await getYouTubeViews(sub.external_url, env.YOUTUBE_API_KEY);
            break;
          case "tiktok":
            viewResult = await getTikTokViews(sub.external_url);
            break;
        }

        // Fallback to Phyllo if no views from public API
        if (!viewResult || viewResult.viewCount === 0) {
          const phylloResult = await phyllo.getViewCount(sub.external_url, sub.platform);
          if (phylloResult) viewResult = phylloResult;
        }

        if (viewResult) {
          const fraudScore = calculateFraudScore(
            viewResult.viewCount,
            sub.total_verified_views,
            6, // hours since last check (cron interval)
          );

          results.push({
            submission_id: sub.id,
            view_count: viewResult.viewCount,
            source: viewResult.source,
            fraud_score: fraudScore,
          });
        }
      } catch (err) {
        console.error(`error verifying ${sub.id}: ${err}`);
      }
    }

    if (results.length > 0) {
      await client.postVerifications(results);
      console.log(`posted ${results.length} verification results`);
    }
  },
};
```

- [ ] **Step 9: Install dependencies**

Run: `cd clips-verifier && npm install`
Expected: Successful install.

- [ ] **Step 10: Verify TypeScript compiles**

Run: `cd clips-verifier && npx wrangler deploy --dry-run`
Expected: Successful dry-run build.

- [ ] **Step 11: Commit**

```bash
git add clips-verifier/
git commit -m "feat: add clips-verifier CF Worker with YouTube/TikTok polling and Phyllo fallback"
```

---

## Chunk 6: SSE Feed + CORS + Final Wiring

### Task 12: Live activity SSE feed

**Files:**
- Modify: `clipcrate/crates/api/src/feed.rs`
- Modify: `clipcrate/crates/api/src/lib.rs` (add route)

- [ ] **Step 1: Write SSE feed handler**

```rust
// clipcrate/crates/api/src/feed.rs
use axum::response::sse::{Event, Sse};
use axum::extract::State;
use futures::stream::Stream;
use std::convert::Infallible;
use tokio_stream::StreamExt;

use crate::AppState;

/// SSE endpoint for live activity feed.
/// Streams new campaigns and submission updates.
/// In production, this subscribes to funnelcake relay via WebSocket.
/// For MVP, polls Postgres for recent activity.
pub async fn live_feed(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = async_stream::stream! {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
        loop {
            interval.tick().await;

            // Query recent campaigns (last 5 minutes)
            if let Ok(campaigns) = clipcrate_db::postgres::list_active_campaigns(&state.db, 5, 0).await {
                for campaign in campaigns {
                    let data = serde_json::json!({
                        "type": "campaign",
                        "id": campaign.id,
                        "title": campaign.title,
                        "cpm_sats": campaign.cpm_sats,
                        "budget_remaining_sats": campaign.budget_remaining_sats,
                    });
                    yield Ok(Event::default().event("campaign").data(data.to_string()));
                }
            }
        }
    };

    Sse::new(stream)
}
```

- [ ] **Step 2: Add SSE route and dependencies**

Add to router in `lib.rs`:
```rust
        .route("/api/feed/live", get(feed::live_feed))
```

Add to `clipcrate/crates/api/Cargo.toml`:
```toml
futures = "0.3"
tokio-stream = "0.1"
async-stream = "0.3"
```

- [ ] **Step 3: Commit**

```bash
git add clipcrate/
git commit -m "feat: add SSE live activity feed endpoint"
```

---

### Task 13: CORS + rate limiting + final main.rs wiring

**Files:**
- Modify: `clipcrate/clipcrate/src/main.rs`
- Modify: `clipcrate/crates/api/src/lib.rs`

- [ ] **Step 1: Add CORS layer to router**

```rust
// In clipcrate/crates/api/src/lib.rs, update the router function:
use tower_http::cors::{Any, CorsLayer};
use axum::http::{HeaderValue, Method};

pub fn router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin([
            "https://clips.divine.video".parse::<HeaderValue>().unwrap(),
            "https://divine.video".parse::<HeaderValue>().unwrap(),
            "http://localhost:5173".parse::<HeaderValue>().unwrap(),  // SvelteKit dev
        ])
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::OPTIONS])
        .allow_headers(Any);

    Router::new()
        .route("/health", get(|| async { "ok" }))
        // ... all routes ...
        .layer(cors)
        .with_state(state)
}
```

- [ ] **Step 2: Add environment config to main.rs**

```rust
// clipcrate/clipcrate/src/main.rs
use anyhow::Result;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost:5432/clipcrate".into());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3100".into());

    let db = clipcrate_db::postgres::create_pool(&database_url).await?;
    tracing::info!("connected to postgres");

    // Run migrations
    sqlx::migrate!("../migrations")
        .run(&db)
        .await?;
    tracing::info!("migrations applied");

    let state = clipcrate_api::AppState { db };
    let app = clipcrate_api::router(state);

    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("clipcrate listening on {addr}");
    axum::serve(listener, app).await?;

    Ok(())
}
```

- [ ] **Step 3: Verify full workspace compiles**

Run: `cd clipcrate && cargo check`
Expected: Successful compilation.

- [ ] **Step 4: Commit**

```bash
git add clipcrate/
git commit -m "feat: add CORS, environment config, and auto-migrations"
```

---

### Task 14: Docker + docker-compose for local development

**Files:**
- Create: `clipcrate/Dockerfile`
- Create: `docker-compose.yml` (at repo root)

- [ ] **Step 1: Write Dockerfile**

```dockerfile
# clipcrate/Dockerfile
FROM rust:1.83 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p clipcrate

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/clipcrate /usr/local/bin/clipcrate
EXPOSE 3100
CMD ["clipcrate"]
```

- [ ] **Step 2: Write docker-compose.yml**

```yaml
# docker-compose.yml
services:
  postgres:
    image: postgres:16
    environment:
      POSTGRES_DB: clipcrate
      POSTGRES_USER: clipcrate
      POSTGRES_PASSWORD: clipcrate
    ports:
      - "5432:5432"
    volumes:
      - pgdata:/var/lib/postgresql/data

  clickhouse:
    image: clickhouse/clickhouse-server:24
    ports:
      - "8123:8123"
      - "9000:9000"
    volumes:
      - chdata:/var/lib/clickhouse

  clipcrate:
    build:
      context: clipcrate
    environment:
      DATABASE_URL: postgres://clipcrate:clipcrate@postgres:5432/clipcrate
      CLICKHOUSE_URL: http://clickhouse:8123
      RUST_LOG: clipcrate=debug,info
      PORT: "3100"
    ports:
      - "3100:3100"
    depends_on:
      - postgres
      - clickhouse

volumes:
  pgdata:
  chdata:
```

- [ ] **Step 3: Commit**

```bash
git add clipcrate/Dockerfile docker-compose.yml
git commit -m "feat: add Dockerfile and docker-compose for local development"
```

---

### Task 15: ClickHouse schema setup script

**Files:**
- Create: `clipcrate/migrations/002_clickhouse_schema.sql`

- [ ] **Step 1: Write ClickHouse migration**

```sql
-- clipcrate/migrations/002_clickhouse_schema.sql
-- Run manually against ClickHouse: clickhouse-client < migrations/002_clickhouse_schema.sql

CREATE TABLE IF NOT EXISTS verification_snapshots (
    submission_id UUID,
    campaign_id UUID,
    clipper_pubkey String,
    timestamp DateTime,
    view_count UInt64,
    source Enum8('youtube_api' = 1, 'tiktok_api' = 2, 'instagram_api' = 3, 'phyllo' = 4, 'manual' = 5, 'tiktok_oembed' = 6),
    fraud_score Float32 DEFAULT 0.0
) ENGINE = MergeTree()
ORDER BY (submission_id, timestamp);

CREATE MATERIALIZED VIEW IF NOT EXISTS campaign_analytics
ENGINE = AggregatingMergeTree()
ORDER BY (campaign_id, date)
AS SELECT
    campaign_id,
    toDate(timestamp) AS date,
    maxState(view_count) AS total_views,
    uniqState(submission_id) AS active_submissions
FROM verification_snapshots
GROUP BY campaign_id, date;
```

- [ ] **Step 2: Commit**

```bash
git add clipcrate/migrations/002_clickhouse_schema.sql
git commit -m "feat: add ClickHouse schema for verification snapshots and analytics"
```

---

## Summary

After completing all 15 tasks, the backend will have:

- **clipcrate**: Rust API service with campaign CRUD, submission management, trust system, wallet, dashboard, internal verification API, SSE feed, CORS, and Docker setup
- **clips-verifier**: CF Worker that polls YouTube/TikTok APIs every 6h, falls back to Phyllo, and posts results to clipcrate
- **Cashu mint**: MVP abstraction (balance-only) ready for Moksha/CDK integration
- **Platform APIs**: Shared Rust crate with YouTube and TikTok clients
- **Database**: Postgres schema with migrations, ClickHouse analytics tables

**What's deferred to Plan 2 (SvelteKit frontend):**
- clips.divine.video web UI
- Keycast login flow
- Campaign browsing, submission, wallet UI
- Live feed display

**What needs integration work after both plans:**
- Real Cashu mint (Moksha/CDK) replacing the balance-only stub
- Full Keycast UCAN validation replacing the auth stub
- Funnelcake relay WebSocket subscription for the SSE feed
- Phyllo OAuth flow for clipper social account linking
