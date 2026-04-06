# DiVine Clips — Design Specification

**Date:** 2026-04-06
**Status:** Approved
**Author:** rabble + Claude

## Overview

DiVine Clips is a clipping marketplace integrated into the DiVine platform. It incentivizes "clippers" to distribute DiVine 6-second loops across TikTok, Instagram Reels, YouTube Shorts, and X in exchange for Bitcoin Lightning micropayments based on verified view counts.

The 6-second loop format is distinctive enough that no branding or CTAs are needed — the content format itself drives curiosity and discovery back to DiVine.

### Goals

- Give creators a self-serve tool to promote their DiVine content
- Give the DiVine platform a way to drive awareness via grassroots distribution
- Attract existing clippers from TikTok/Reels communities into the DiVine/Nostr ecosystem
- Pay clippers in Bitcoin Lightning via Cashu ecash tokens
- Build portable clipper reputation on Nostr identity

### Non-Goals (MVP)

- Compilation/remix editing tools (clippers use their own)
- AI-powered clip suggestions
- Advanced fraud detection (manual review for suspicious activity)
- CPA (cost per acquisition) tracking
- Multi-creator campaigns
- Clipper leaderboards

---

## Architecture

### System Components

Three primary components:

**1. clipcrate (Rust service)**
The main API server. Handles:
- Campaign management (CRUD, budget tracking)
- Clip submissions (intake, status tracking)
- Clipper profiles (identity, trust levels, connected accounts)
- Cashu mint operations (campaign escrow, payout minting, redemption)
- Nostr event publishing to funnelcake
- REST + WebSocket API for the SvelteKit frontend

Owns mutable state in Postgres. Reads ClickHouse for analytics.

**2. clips-verifier (Cloudflare Worker, TypeScript)**
Runs on Cron Triggers (every 6 hours). Handles:
- Fetching pending/active submissions from clipcrate API
- Polling platform APIs for view counts
- Basic fraud checks (velocity, duplicates)
- Posting verification results back to clipcrate
- Triggering payouts when thresholds are met

Stateless — reads from and writes to clipcrate's API.

**3. clips.divine.video (SvelteKit)**
Clipper-facing web UI. Handles:
- Keycast login for Nostr identity (clippers don't need to understand Nostr)
- Campaign browsing with live activity feed
- Content download from Blossom
- Clip link submission
- Earnings dashboard and Cashu wallet
- Social account linking (Phyllo OAuth)

Also serves a creator-facing campaign creation flow.

### Shared Infrastructure

- **divine-platform-apis** (Rust crate) — Shared library for TikTok, YouTube, Instagram, X API clients. Used by clipcrate and verifier.divine.video. The CF Worker reimplements the same API calls in TypeScript.
- **Funnelcake relay** — Clipcrate publishes Nostr events here for interoperability and real-time subscriptions
- **ClickHouse** — Verification snapshots, view count time-series, analytics
- **Cashu mint (Moksha)** — Embedded in clipcrate or run as a sidecar. Handles campaign escrow and clipper payouts.

### Architecture Diagram

```
┌──────────────────┐     ┌─────────────────────┐
│  clips.divine.video  │────▶│     clipcrate        │
│  (SvelteKit)       │◀────│     (Rust API)       │
└──────────────────┘     │                     │
        │ WS              │  ┌───────────────┐ │
        │                 │  │  Moksha mint   │ │
        ▼                 │  │  (Cashu)       │ │
┌──────────────────┐     │  └───────────────┘ │
│  funnelcake relay  │◀────│                     │
│  (Nostr events)    │     └──────┬──────┬──────┘
└──────────────────┘            │      │
                                │      │
                          ┌─────▼──┐ ┌─▼──────────┐
                          │Postgres│ │ ClickHouse  │
                          │(state) │ │(analytics)  │
                          └────────┘ └─────────────┘
                                          ▲
┌──────────────────────┐                  │
│  clips-verifier        │──────────────────┘
│  (CF Worker, cron)     │───▶ YouTube, TikTok, IG, X APIs
│                        │───▶ Phyllo (fallback)
└──────────────────────┘
```

---

## Nostr Event Design

Reuses existing NIPs where possible.

### Campaign Event (NIP-15 Classified Listing — Kind 30402)

A clip campaign is a marketplace listing offering sats for clipping services.

```json
{
  "kind": 30402,
  "pubkey": "<creator or divine platform pubkey>",
  "tags": [
    ["d", "<campaign-uuid>"],
    ["title", "Promote: @creator's loop collection"],
    ["summary", "Clip these loops to TikTok/Reels/Shorts and earn sats"],
    ["a", "34236:<creator-pubkey>:<d-tag>", "<relay>"],
    ["price", "3000", "sats", "per_1000_views"],
    ["budget", "500000"],
    ["expiration", "<unix-timestamp>"],
    ["t", "tiktok"],
    ["t", "instagram"],
    ["t", "youtube"],
    ["t", "x"],
    ["status", "active"],
    ["L", "divine-clips"],
    ["l", "campaign", "divine-clips"]
  ],
  "content": "Optional campaign guidelines or notes for clippers"
}
```

- `a` tags reference DiVine video events (kind 34236) being promoted
- Multiple `a` tags for campaigns promoting several videos
- `price` tag uses NIP-15 pricing convention extended with `per_1000_views` frequency
- `t` tags specify target platforms
- NIP-32 labels (`divine-clips` namespace) enable filtering
- NIP-40 `expiration` tag for campaign duration
- Parameterized replaceable: creator can update status, budget, settings

### Submission Event (Kind 30403 — Parameterized Replaceable)

```json
{
  "kind": 30403,
  "pubkey": "<clipper-pubkey>",
  "tags": [
    ["d", "<submission-uuid>"],
    ["a", "30402:<campaign-creator-pubkey>:<campaign-d-tag>", "<relay>"],
    ["r", "https://tiktok.com/@clipper/video/123456"],
    ["platform", "tiktok"],
    ["status", "pending"],
    ["L", "divine-clips"],
    ["l", "submission", "divine-clips"]
  ],
  "content": ""
}
```

- References campaign via `a` tag
- `r` tag holds the external clip URL
- Parameterized replaceable so clipper/system can update status
- One submission per external clip URL per campaign

### Payout Events (NIP-57 Zaps)

Payouts use standard zap infrastructure. Zap request (kind 9734) references both the submission and campaign:

```json
{
  "kind": 9734,
  "pubkey": "<clipcrate-service-pubkey>",
  "tags": [
    ["p", "<clipper-pubkey>"],
    ["a", "30403:<clipper-pubkey>:<submission-d-tag>"],
    ["a", "30402:<creator-pubkey>:<campaign-d-tag>"],
    ["amount", "15000"],
    ["relays", "<relay-urls>"]
  ],
  "content": "Payout for 5,000 verified views on campaign X"
}
```

### Verification Records (Off-Chain)

Verification data lives in ClickHouse, not on Nostr relays. This is operational/analytics data that doesn't benefit from decentralization and would bloat relays.

### Live Activity

No new event kinds needed. The live feed in clips.divine.video is built from filtered Nostr subscriptions on funnelcake:
- Kind 34236 (new DiVine videos) → "Creator just posted"
- Kind 30402 (new/updated campaigns) → "New campaign available"
- Gorse recommendation signals via NATS → "Trending content"

---

## Verification Pipeline

### Flow

```
Clipper submits link
  → clipcrate validates URL format, stores submission (Postgres, status: pending)
  → publishes kind 30403 to funnelcake

clips-verifier (CF Worker, cron every 6h):
  → GET /api/internal/submissions?status=active from clipcrate
  → for each submission:
      → try public API (YouTube Data API, TikTok oEmbed)
      → fallback to Phyllo if no public data
      → POST /api/internal/verifications with view count snapshot
  → clipcrate writes snapshot to ClickHouse
  → if cumulative views cross next payout threshold:
      → clipcrate calculates payout amount
      → mints Cashu tokens from campaign escrow → clipper balance
      → publishes zap event to funnelcake
```

### Verification Schedule

| Checkpoint | Timing | Purpose |
|-----------|--------|---------|
| T+0 | On submission | Validate URL exists, get baseline views |
| T+24h | Next cron after 24h | Early traction check |
| T+48h | Next cron after 48h | Confirm growth trajectory |
| T+7d | Next cron after 7d | Mid-term performance |
| T+30d | Next cron after 30d | Final count |

Views after 30 days are not credited. This keeps campaign budgets predictable.

### Platform API Strategy

| Platform | Public access | Auth required | Fallback |
|----------|-------------|---------------|----------|
| YouTube | Data API v3 (video stats by ID) | API key only | Phyllo |
| TikTok | oEmbed (basic) + Display API (detailed) | App approval (5-7 day review) | Phyllo |
| Instagram | No public stats API | Business account OAuth | Phyllo only |
| X | API v2 (tweet metrics) | Bearer token | Phyllo |

Phase 1 (MVP): Phyllo ($199/mo) as primary for TikTok, Instagram, X. YouTube via public API.
Phase 2: Direct API integrations replace Phyllo. Share clients with verifier.divine.video via divine-platform-apis crate.

### Clip Formats

Clippers may:
- Repost a single DiVine loop as-is
- Compile multiple DiVine loops into one video (Vine compilation style)
- Use DiVine content in commentary/reaction videos
- Remix or add context around loops

For MVP, one submission = one campaign. The clipper selects the primary campaign when submitting a compilation. Multi-campaign attribution is a future enhancement.

---

## Fraud Prevention

### Graduated Trust System

**Trust Level 1 (new clippers):**
- Max 50,000 credited views/week
- Max 5 active submissions
- Payouts held 48 hours before Cashu tokens are mintable
- All submissions eligible for spot-check

**Trust Level 2 (after 100K verified views + 30 days):**
- Max 500,000 credited views/week
- Max 20 active submissions
- Payouts available after 24 hours

**Trust Level 3 (after 1M verified views + 90 days):**
- No view cap
- Unlimited active submissions
- Instant payouts

### Automated Fraud Flags

- **View velocity spike**: >10x normal growth rate in <6 hours
- **Duplicate URL**: Same external URL submitted to multiple campaigns
- **Young account**: Platform account age < 30 days
- **Suspicious patterns**: Steady exact-increment view growth (bot signature)

Flagged submissions enter a manual review queue. For MVP, the DiVine team reviews these manually.

---

## Payment System (Cashu)

### Mint Architecture

Moksha (Rust Cashu mint) embedded in or co-deployed with clipcrate. Backed by a Lightning node (LND or CLN).

### Campaign Deposit Flow

1. Creator initiates campaign on clips.divine.video or divine.video
2. Creator pays Lightning invoice generated by the Moksha mint
3. Mint issues ecash tokens locked to the campaign ID
4. Campaign status becomes "active"

### Payout Flow

1. Verification triggers payout calculation (views × CPM rate)
2. Clipcrate instructs mint to issue new tokens to clipper's pubkey
3. Clipper's Cashu balance increases in clips.divine.video
4. Zap receipt (kind 9735) published to funnelcake for transparency

### Redemption Flow

1. Clipper clicks "Withdraw" in their wallet view
2. Provides a Lightning invoice (from any wallet) or LNURL
3. Mint melts tokens and pays the invoice
4. Alternative: clipper can export Cashu token strings for use in any Cashu wallet

### Campaign Exhaustion

When a campaign's token pool reaches zero:
- Campaign status flips to "exhausted" (kind 30402 updated)
- No new submissions accepted
- In-progress submissions continue through verification (unfunded views aren't credited)

---

## Data Storage

### Postgres (Mutable State — clipcrate)

```sql
-- Campaigns
campaigns (
  id UUID PRIMARY KEY,
  nostr_event_id TEXT,
  creator_pubkey TEXT,
  title TEXT,
  budget_total_sats BIGINT,
  budget_remaining_sats BIGINT,
  cpm_sats INT,              -- sats per 1,000 views
  status TEXT,                -- active, paused, completed, exhausted
  target_platforms TEXT[],
  expires_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ
)

-- Submissions
submissions (
  id UUID PRIMARY KEY,
  nostr_event_id TEXT,
  campaign_id UUID REFERENCES campaigns,
  clipper_pubkey TEXT,
  external_url TEXT UNIQUE,
  platform TEXT,              -- tiktok, instagram, youtube, x
  status TEXT,                -- pending, active, verified, rejected
  total_verified_views BIGINT DEFAULT 0,
  total_paid_sats BIGINT DEFAULT 0,
  submitted_at TIMESTAMPTZ,
  last_verified_at TIMESTAMPTZ
)

-- Clippers
clippers (
  pubkey TEXT PRIMARY KEY,
  trust_level INT DEFAULT 1,
  total_verified_views BIGINT DEFAULT 0,
  total_earned_sats BIGINT DEFAULT 0,
  weekly_views_used BIGINT DEFAULT 0,
  weekly_views_reset_at TIMESTAMPTZ,
  phyllo_account_id TEXT,     -- if connected via Phyllo
  created_at TIMESTAMPTZ
)

-- Payout ledger
payouts (
  id UUID PRIMARY KEY,
  submission_id UUID REFERENCES submissions,
  campaign_id UUID REFERENCES campaigns,
  clipper_pubkey TEXT,
  amount_sats BIGINT,
  views_at_payout BIGINT,
  cashu_token_id TEXT,
  zap_event_id TEXT,
  created_at TIMESTAMPTZ
)
```

### ClickHouse (Analytics / Time-Series)

```sql
-- Verification snapshots (append-only)
CREATE TABLE verification_snapshots (
  submission_id UUID,
  timestamp DateTime,
  view_count UInt64,
  source Enum('youtube_api', 'tiktok_api', 'phyllo', 'manual'),
  fraud_score Float32 DEFAULT 0.0
) ENGINE = MergeTree()
ORDER BY (submission_id, timestamp);

-- Campaign analytics (materialized view)
CREATE MATERIALIZED VIEW campaign_analytics
ENGINE = SummingMergeTree()
ORDER BY (campaign_id, date)
AS SELECT
  campaign_id,
  toDate(timestamp) AS date,
  max(view_count) AS total_views,
  count(DISTINCT submission_id) AS active_submissions
FROM verification_snapshots
GROUP BY campaign_id, date;
```

---

## SvelteKit Frontend (clips.divine.video)

### Routes

| Route | Purpose | Auth |
|-------|---------|------|
| `/` | Live activity feed (new campaigns, creator bursts, trending) | Public |
| `/campaigns` | Browse active campaigns, filter by platform/CPM/budget | Public |
| `/campaigns/[id]` | Campaign detail, content preview, "Claim & Clip" | Public (submit requires auth) |
| `/submit` | Submit clip link, tag campaign | Clipper auth |
| `/dashboard` | Active submissions, verified views, earnings, trust level | Clipper auth |
| `/wallet` | Cashu balance, withdraw to Lightning, transaction history | Clipper auth |
| `/profile` | Connected social accounts, reputation, stats | Clipper auth |
| `/create` | Campaign creation (creator flow) | Creator auth |

### Auth

Keycast login provides a managed Nostr keypair. Clippers see "Sign in with DiVine" — no Nostr terminology exposed. Session persisted via cookie/JWT.

### Real-Time Feed

WebSocket subscription to funnelcake relay for:
- Kind 34236 → new DiVine videos posted (creator burst detection)
- Kind 30402 → new/updated campaigns
- Kind 30403 → submission status changes

### Content Download

Campaigns link to Blossom URLs. Clipper downloads source video directly from Blossom CDN. No DRM — redistribution is the entire point.

---

## API Surface (clipcrate)

### Public / Clipper Endpoints

```
POST   /api/campaigns              Create campaign (creator auth)
GET    /api/campaigns              List active campaigns (public)
GET    /api/campaigns/:id          Campaign detail (public)
PATCH  /api/campaigns/:id          Pause/resume/update campaign (creator auth)

POST   /api/submissions            Submit clip link (clipper auth)
GET    /api/submissions            List my submissions (clipper auth)
GET    /api/submissions/:id        Submission detail + verification history

GET    /api/dashboard              Clipper stats summary (clipper auth)

GET    /api/wallet/balance         Cashu token balance (clipper auth)
POST   /api/wallet/withdraw        Redeem tokens to Lightning invoice (clipper auth)
GET    /api/wallet/history         Transaction history (clipper auth)

POST   /api/auth/keycast           Keycast login flow
POST   /api/auth/phyllo/connect    Initiate Phyllo OAuth for social linking

GET    /api/feed/live              Live activity (SSE stream)
```

### Internal Endpoints (service-to-service auth)

```
GET    /api/internal/submissions   Get submissions pending verification
POST   /api/internal/verifications Post verification results
POST   /api/internal/payouts       Trigger payout for a submission
```

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Platform API access denied/revoked | Medium | High | Phyllo as fallback, multiple verification sources |
| Fake views / bot traffic | High | Medium | Graduated trust, velocity checks, manual review |
| Cashu mint security | Low | Critical | Use audited Moksha, limit mint balance, regular sweeps |
| Phyllo integration breaks | Medium | Medium | Monitor aggressively, direct API migration path |
| Low clipper adoption | Medium | High | Competitive CPM rates, target existing clipper communities |
| Campaign budget griefing | Low | Low | Minimum campaign budgets, creator verification |
| Lightning payment failures | Low | Medium | Retry logic, Cashu token fallback (redeem later) |

---

## MVP Task Breakdown (High-Level)

1. **clipcrate service scaffold** — Rust project, Postgres schema, basic REST API
2. **Nostr event integration** — Campaign + submission events to/from funnelcake
3. **Cashu mint integration** — Moksha embed, campaign deposit, payout, redemption
4. **clips-verifier CF Worker** — Cron trigger, YouTube API polling, Phyllo integration
5. **SvelteKit frontend** — Keycast auth, campaign browsing, submission flow, wallet
6. **Graduated trust system** — Trust levels, payout caps, fraud flags
7. **Live activity feed** — Funnelcake WebSocket subscriptions, real-time UI
8. **Creator campaign flow** — Campaign creation UI (divine.video or clips.divine.video)

Detailed task breakdown to be produced in implementation planning phase.
