CREATE TABLE campaign_funding (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    campaign_id UUID NOT NULL REFERENCES campaigns(id),
    quote_id TEXT NOT NULL UNIQUE,
    amount_sats BIGINT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT valid_funding_status CHECK (status IN ('pending', 'completed', 'expired'))
);

CREATE INDEX idx_campaign_funding_campaign ON campaign_funding(campaign_id);
CREATE INDEX idx_campaign_funding_quote ON campaign_funding(quote_id);
