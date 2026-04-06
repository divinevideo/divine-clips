use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{Campaign, Clipper, Submission};

pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;
    Ok(pool)
}

pub async fn create_campaign(
    pool: &PgPool,
    creator_pubkey: &str,
    title: &str,
    budget_sats: i64,
    cpm_sats: i32,
    target_platforms: Vec<String>,
    content_refs: Vec<String>,
    guidelines: Option<&str>,
    expires_at: Option<DateTime<Utc>>,
) -> Result<Campaign> {
    let campaign = sqlx::query_as::<_, Campaign>(
        r#"
        INSERT INTO campaigns (
            creator_pubkey, title, budget_total_sats, budget_remaining_sats,
            cpm_sats, status, target_platforms, content_refs, guidelines, expires_at
        )
        VALUES ($1, $2, $3, $3, $4, 'active', $5, $6, $7, $8)
        RETURNING *
        "#,
    )
    .bind(creator_pubkey)
    .bind(title)
    .bind(budget_sats)
    .bind(cpm_sats)
    .bind(&target_platforms)
    .bind(&content_refs)
    .bind(guidelines)
    .bind(expires_at)
    .fetch_one(pool)
    .await?;

    Ok(campaign)
}

pub async fn get_campaign(pool: &PgPool, id: Uuid) -> Result<Option<Campaign>> {
    let campaign = sqlx::query_as::<_, Campaign>(
        "SELECT * FROM campaigns WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(campaign)
}

pub async fn list_active_campaigns(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<Campaign>> {
    let campaigns = sqlx::query_as::<_, Campaign>(
        r#"
        SELECT * FROM campaigns
        WHERE status = 'active'
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(campaigns)
}

pub async fn update_campaign_status(
    pool: &PgPool,
    id: Uuid,
    creator_pubkey: &str,
    status: &str,
) -> Result<Option<Campaign>> {
    let campaign = sqlx::query_as::<_, Campaign>(
        r#"
        UPDATE campaigns
        SET status = $1, updated_at = NOW()
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

pub async fn get_or_create_clipper(pool: &PgPool, pubkey: &str) -> Result<Clipper> {
    let clipper = sqlx::query_as::<_, Clipper>(
        r#"
        INSERT INTO clippers (pubkey)
        VALUES ($1)
        ON CONFLICT (pubkey) DO UPDATE
            SET pubkey = EXCLUDED.pubkey
        RETURNING *
        "#,
    )
    .bind(pubkey)
    .fetch_one(pool)
    .await?;

    Ok(clipper)
}

pub async fn create_submission(
    pool: &PgPool,
    campaign_id: Uuid,
    clipper_pubkey: &str,
    external_url: &str,
    platform: &str,
) -> Result<Submission> {
    let submission = sqlx::query_as::<_, Submission>(
        r#"
        INSERT INTO submissions (campaign_id, clipper_pubkey, external_url, platform, status)
        VALUES ($1, $2, $3, $4, 'pending')
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

pub async fn list_clipper_submissions(
    pool: &PgPool,
    clipper_pubkey: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<Submission>> {
    let submissions = sqlx::query_as::<_, Submission>(
        r#"
        SELECT * FROM submissions
        WHERE clipper_pubkey = $1
        ORDER BY submitted_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(clipper_pubkey)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(submissions)
}

pub async fn get_submission(pool: &PgPool, id: Uuid) -> Result<Option<Submission>> {
    let submission = sqlx::query_as::<_, Submission>(
        "SELECT * FROM submissions WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(submission)
}

pub async fn count_active_submissions(pool: &PgPool, clipper_pubkey: &str) -> Result<i64> {
    let row: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM submissions
        WHERE clipper_pubkey = $1
          AND status IN ('pending', 'active')
        "#,
    )
    .bind(clipper_pubkey)
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}
