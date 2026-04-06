ALTER TABLE campaigns ADD COLUMN IF NOT EXISTS source TEXT NOT NULL DEFAULT 'manual';
ALTER TABLE campaigns ADD COLUMN IF NOT EXISTS divine_video_event_id TEXT;
CREATE UNIQUE INDEX IF NOT EXISTS idx_campaigns_divine_video ON campaigns(divine_video_event_id) WHERE divine_video_event_id IS NOT NULL;
