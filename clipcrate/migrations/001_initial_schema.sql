CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE campaigns (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    nostr_event_id TEXT,
    creator_pubkey TEXT NOT NULL,
    title TEXT NOT NULL,
    budget_total_sats BIGINT NOT NULL,
    budget_remaining_sats BIGINT NOT NULL,
    cpm_sats INT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    target_platforms TEXT[] NOT NULL DEFAULT '{}',
    content_refs TEXT[] NOT NULL DEFAULT '{}',
    guidelines TEXT,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT valid_status CHECK (status IN ('pending', 'active', 'paused', 'completed', 'exhausted')),
    CONSTRAINT positive_budget CHECK (budget_total_sats > 0),
    CONSTRAINT budget_not_negative CHECK (budget_remaining_sats >= 0)
);

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

CREATE INDEX idx_campaigns_status ON campaigns(status);
CREATE INDEX idx_campaigns_creator ON campaigns(creator_pubkey);
CREATE INDEX idx_submissions_campaign ON submissions(campaign_id);
CREATE INDEX idx_submissions_status ON submissions(status);
CREATE INDEX idx_submissions_clipper ON submissions(clipper_pubkey);
CREATE INDEX idx_payouts_clipper ON payouts(clipper_pubkey);
CREATE INDEX idx_payouts_submission ON payouts(submission_id);
CREATE INDEX idx_withdrawals_clipper ON withdrawals(clipper_pubkey);
