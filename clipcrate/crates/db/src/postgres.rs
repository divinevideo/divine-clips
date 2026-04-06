use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{Campaign, Clipper, LeaderboardEntry, Payout, Submission};

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

pub async fn list_recent_campaigns(pool: &sqlx::PgPool, since_minutes: i64) -> Result<Vec<Campaign>> {
    let campaigns = sqlx::query_as::<_, Campaign>(
        "SELECT * FROM campaigns WHERE updated_at > NOW() - make_interval(mins => $1) ORDER BY updated_at DESC LIMIT 10"
    )
    .bind(since_minutes)
    .fetch_all(pool)
    .await?;
    Ok(campaigns)
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

pub async fn list_pending_submissions(pool: &PgPool) -> Result<Vec<Submission>> {
    let submissions = sqlx::query_as::<_, Submission>(
        r#"
        SELECT * FROM submissions
        WHERE status IN ('pending', 'active')
          AND submitted_at > NOW() - INTERVAL '30 days'
        ORDER BY submitted_at ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(submissions)
}

pub async fn update_submission_views(
    pool: &PgPool,
    id: Uuid,
    views: i64,
) -> Result<Option<Submission>> {
    let submission = sqlx::query_as::<_, Submission>(
        r#"
        UPDATE submissions
        SET total_verified_views = $2,
            last_verified_at = NOW(),
            status = 'active'
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(views)
    .fetch_optional(pool)
    .await?;

    Ok(submission)
}

pub async fn create_payout(
    pool: &PgPool,
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

pub async fn deduct_campaign_budget(
    pool: &PgPool,
    campaign_id: Uuid,
    amount_sats: i64,
) -> Result<Option<Campaign>> {
    let mut tx = pool.begin().await?;

    let campaign = sqlx::query_as::<_, Campaign>(
        "SELECT * FROM campaigns WHERE id = $1 FOR UPDATE",
    )
    .bind(campaign_id)
    .fetch_optional(&mut *tx)
    .await?;

    let campaign = match campaign {
        Some(c) => c,
        None => {
            tx.rollback().await?;
            return Ok(None);
        }
    };

    let new_budget = (campaign.budget_remaining_sats - amount_sats).max(0);
    let new_status = if new_budget == 0 { "exhausted" } else { campaign.status.as_str() };

    let updated = sqlx::query_as::<_, Campaign>(
        r#"
        UPDATE campaigns
        SET budget_remaining_sats = $2,
            status = $3,
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(campaign_id)
    .bind(new_budget)
    .bind(new_status)
    .fetch_optional(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(updated)
}

pub async fn get_leaderboard(
    pool: &PgPool,
    metric: &str,
    period: &str,
    limit: i64,
) -> Result<Vec<LeaderboardEntry>> {
    let period_filter = match period {
        "week" => "AND s.submitted_at > NOW() - INTERVAL '7 days'",
        "month" => "AND s.submitted_at > NOW() - INTERVAL '30 days'",
        _ => "",
    };

    let sql = match metric {
        "views" => format!(
            r#"
            SELECT c.pubkey, c.trust_level, COALESCE(SUM(s.total_verified_views), 0)::bigint AS value
            FROM clippers c
            LEFT JOIN submissions s ON s.clipper_pubkey = c.pubkey
            WHERE c.trust_level >= 2 {}
            GROUP BY c.pubkey, c.trust_level
            ORDER BY value DESC
            LIMIT $1
            "#,
            period_filter
        ),
        "best_clip" => format!(
            r#"
            SELECT c.pubkey, c.trust_level, COALESCE(MAX(s.total_verified_views), 0)::bigint AS value
            FROM clippers c
            LEFT JOIN submissions s ON s.clipper_pubkey = c.pubkey
            WHERE c.trust_level >= 2 {}
            GROUP BY c.pubkey, c.trust_level
            ORDER BY value DESC
            LIMIT $1
            "#,
            period_filter
        ),
        // default: earnings
        _ => format!(
            r#"
            SELECT c.pubkey, c.trust_level, COALESCE(SUM(p.amount_sats), 0)::bigint AS value
            FROM clippers c
            LEFT JOIN payouts p ON p.clipper_pubkey = c.pubkey
            WHERE c.trust_level >= 2 {}
            GROUP BY c.pubkey, c.trust_level
            ORDER BY value DESC
            LIMIT $1
            "#,
            period_filter
        ),
    };

    let entries = sqlx::query_as::<_, LeaderboardEntry>(&sql)
        .bind(limit)
        .fetch_all(pool)
        .await?;

    Ok(entries)
}

pub async fn get_social_proof_stats(pool: &PgPool) -> Result<(i64, i64)> {
    let clippers_row: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(DISTINCT clipper_pubkey)
        FROM submissions
        WHERE submitted_at > NOW() - INTERVAL '7 days'
        "#,
    )
    .fetch_one(pool)
    .await?;

    let sats_row: (i64,) = sqlx::query_as(
        r#"
        SELECT COALESCE(SUM(amount_sats), 0)::bigint
        FROM payouts
        WHERE created_at > NOW() - INTERVAL '7 days'
        "#,
    )
    .fetch_one(pool)
    .await?;

    Ok((clippers_row.0, sats_row.0))
}
