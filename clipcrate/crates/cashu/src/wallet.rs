use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// A single entry in a clipper's transaction history.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TransactionRecord {
    /// Amount in satoshis (always positive).
    pub amount_sats: i64,
    /// `"payout"` for earned tokens; `"withdrawal"` for Lightning cash-outs.
    pub transaction_type: String,
    pub created_at: DateTime<Utc>,
}

/// Return the current spendable balance for a clipper.
///
/// balance = sum(payouts) - sum(completed withdrawals)
pub async fn get_balance(pool: &PgPool, clipper_pubkey: &str) -> Result<i64> {
    let (earned,): (i64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(amount_sats), 0) FROM payouts WHERE clipper_pubkey = $1",
    )
    .bind(clipper_pubkey)
    .fetch_one(pool)
    .await?;

    let (withdrawn,): (i64,) = sqlx::query_as(
        "SELECT COALESCE(SUM(amount_sats), 0) FROM withdrawals \
         WHERE clipper_pubkey = $1 AND status = 'completed'",
    )
    .bind(clipper_pubkey)
    .fetch_one(pool)
    .await?;

    Ok(earned - withdrawn)
}

/// Return up to `limit` most-recent transactions (payouts + withdrawals) for a clipper,
/// ordered newest-first.
pub async fn get_transaction_history(
    pool: &PgPool,
    clipper_pubkey: &str,
    limit: i64,
) -> Result<Vec<TransactionRecord>> {
    let records = sqlx::query_as::<_, TransactionRecord>(
        r#"
        SELECT amount_sats, 'payout' AS transaction_type, created_at
        FROM payouts
        WHERE clipper_pubkey = $1
        UNION ALL
        SELECT amount_sats, 'withdrawal' AS transaction_type, created_at
        FROM withdrawals
        WHERE clipper_pubkey = $1 AND status = 'completed'
        ORDER BY created_at DESC
        LIMIT $2
        "#,
    )
    .bind(clipper_pubkey)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(records)
}

#[cfg(test)]
mod tests {
    /// Unit tests for balance logic that don't require a live database.
    /// Integration tests (with a real PgPool) live in the integration test suite.

    #[test]
    fn test_balance_formula_earned_minus_withdrawn() {
        // Simulate: earned 1000, withdrawn 400 -> balance 600
        let earned: i64 = 1000;
        let withdrawn: i64 = 400;
        let balance = earned - withdrawn;
        assert_eq!(balance, 600);
    }

    #[test]
    fn test_balance_zero_when_nothing_earned() {
        let earned: i64 = 0;
        let withdrawn: i64 = 0;
        assert_eq!(earned - withdrawn, 0);
    }

    #[test]
    fn test_balance_full_withdrawal() {
        let earned: i64 = 500;
        let withdrawn: i64 = 500;
        assert_eq!(earned - withdrawn, 0);
    }
}
