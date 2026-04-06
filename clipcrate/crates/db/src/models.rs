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
    pub content_refs: Vec<String>,
    pub guidelines: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub source: String,
    pub divine_video_event_id: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LeaderboardEntry {
    pub pubkey: String,
    pub trust_level: i32,
    pub value: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Withdrawal {
    pub id: Uuid,
    pub clipper_pubkey: String,
    pub amount_sats: i64,
    pub lightning_invoice: Option<String>,
    pub payment_preimage: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PushSubscription {
    pub id: Uuid,
    pub clipper_pubkey: String,
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
    pub created_at: DateTime<Utc>,
}
