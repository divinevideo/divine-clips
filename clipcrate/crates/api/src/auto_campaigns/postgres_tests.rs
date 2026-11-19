use super::*;
use clipcrate_db::postgres::get_leaderboard;

// SQLx gives each test its own migrated database and removes it on success.
// CI runs these explicitly; ordinary library tests need no database service.
#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires DATABASE_URL pointing to a disposable PostgreSQL server"]
async fn postgres_auto_campaign_insert_is_idempotent(pool: PgPool) {
    let keys = Keys::generate();
    let event = EventBuilder::new(Kind::from(34236), "synthetic test video")
        .tags([
            Tag::parse(["title", "Synthetic title"]).unwrap(),
            Tag::parse(["d", "synthetic-coordinate"]).unwrap(),
        ])
        .sign_with_keys(&keys)
        .unwrap();

    // Two deliveries exercise the real handler and the migrated partial index.
    let (first, duplicate) = tokio::join!(handle_video(&pool, &event), handle_video(&pool, &event));
    first.unwrap();
    duplicate.unwrap();

    let rows: Vec<(String, String, i64, i64, String, Vec<String>)> = sqlx::query_as(
        "SELECT creator_pubkey, title, budget_total_sats, budget_remaining_sats, source, content_refs
         FROM campaigns WHERE divine_video_event_id = $1",
    )
    .bind(event.id.to_hex())
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        rows,
        vec![(
            keys.public_key().to_hex(),
            "Clip: Synthetic title".into(),
            10_000,
            10_000,
            "auto".into(),
            vec!["synthetic-coordinate".into()],
        )]
    );
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "requires DATABASE_URL pointing to a disposable PostgreSQL server"]
async fn postgres_leaderboard_uses_each_metric_timestamp(pool: PgPool) {
    let pubkey = "11".repeat(32);
    sqlx::query("INSERT INTO clippers (pubkey, trust_level) VALUES ($1, 2)")
        .bind(&pubkey)
        .execute(&pool)
        .await
        .unwrap();
    let campaign: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO campaigns (creator_pubkey, title, budget_total_sats, budget_remaining_sats, cpm_sats)
         VALUES ($1, 'synthetic campaign', 10000, 10000, 1000) RETURNING id",
    )
    .bind(&pubkey)
    .fetch_one(&pool)
    .await
    .unwrap();

    let mut old_submission = None;
    for (days, views) in [(45_i32, 500_i64), (1, 50)] {
        let id: uuid::Uuid = sqlx::query_scalar(
            "INSERT INTO submissions (campaign_id, clipper_pubkey, external_url, platform, total_verified_views, submitted_at)
             VALUES ($1, $2, $3, 'youtube', $4, NOW() - $5 * INTERVAL '1 day') RETURNING id",
        )
        .bind(campaign)
        .bind(&pubkey)
        .bind(format!("https://example.invalid/synthetic/{days}"))
        .bind(views)
        .bind(days)
        .fetch_one(&pool)
        .await
        .unwrap();
        if days == 45 {
            old_submission = Some(id);
        }
    }
    // All payouts refer to an old submission: earnings must use payout time.
    for (days, amount) in [(1_i32, 100_i64), (15, 200), (45, 400)] {
        sqlx::query(
            "INSERT INTO payouts (submission_id, campaign_id, clipper_pubkey, amount_sats, views_at_payout, created_at)
             VALUES ($1, $2, $3, $4, 1, NOW() - $5 * INTERVAL '1 day')",
        )
        .bind(old_submission.unwrap())
        .bind(campaign)
        .bind(&pubkey)
        .bind(amount)
        .bind(days)
        .execute(&pool)
        .await
        .unwrap();
    }

    for (metric, expected) in [
        ("earnings", [100, 300, 700]),
        ("unknown-defaults-to-earnings", [100, 300, 700]),
        ("views", [50, 50, 550]),
        ("best_clip", [50, 50, 500]),
    ] {
        for (period, value) in ["week", "month", "all"].into_iter().zip(expected) {
            let rows = get_leaderboard(&pool, metric, period, 10).await.unwrap();
            assert_eq!(rows.len(), 1, "{metric}/{period}");
            assert_eq!(rows[0].pubkey, pubkey, "{metric}/{period}");
            assert_eq!(rows[0].value, value, "{metric}/{period}");
        }
    }
}
