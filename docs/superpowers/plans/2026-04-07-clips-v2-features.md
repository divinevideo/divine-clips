# DiVine Clips v2 Features Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add 6 key features to close the gap with competing clipping platforms: video player, auto-campaigns, leaderboard, CDK mint, Web Push notifications, and analytics charts.

**Architecture:** Tasks 1-3 are frontend-only (clips-web SvelteKit). Task 4 is backend Rust (clipcrate). Tasks 5-6 span both. All tasks are independent and can run in parallel.

**Tech Stack:** SvelteKit 5, TailwindCSS, LayerCake (charts), Rust/Axum, CDK/cdk-mintd, Phoenixd, Web Push (VAPID), nostr-sdk.

**Spec:** `docs/superpowers/specs/2026-04-07-clips-v2-research-findings.md`

---

## Task 1: Video Player (Hover-to-Play)

**Scope:** Frontend only (`clips-web/`). Create a VideoCard Svelte component and integrate it into the campaigns browse page.

**Files:**
- Create: `clips-web/src/lib/components/VideoCard.svelte`
- Modify: `clips-web/src/routes/campaigns/+page.svelte` — replace thumbnail `<img>` with VideoCard

**VideoCard.svelte:**

```svelte
<script lang="ts">
  let { src, thumbnail, title, subtitle }: {
    src: string; thumbnail?: string; title: string; subtitle?: string;
  } = $props();

  let videoEl = $state<HTMLVideoElement | null>(null);
  let isHovering = $state(false);
  let hasLoaded = $state(false);

  function observe(node: HTMLElement) {
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (!entry.isIntersecting && videoEl) {
          videoEl.pause();
          videoEl.removeAttribute('src');
          videoEl.load();
          hasLoaded = false;
        }
      },
      { rootMargin: '200px', threshold: 0.1 }
    );
    observer.observe(node);
    return { destroy: () => observer.disconnect() };
  }

  function startPlay() {
    isHovering = true;
    if (videoEl) {
      if (!hasLoaded) { videoEl.src = src; hasLoaded = true; }
      videoEl.play().catch(() => {});
    }
  }

  function stopPlay() {
    isHovering = false;
    if (videoEl) { videoEl.pause(); videoEl.currentTime = 0; }
  }
</script>

<div
  use:observe
  onmouseenter={startPlay}
  onmouseleave={stopPlay}
  ontouchstart={startPlay}
  class="relative aspect-[9/16] bg-gray-900 rounded-xl overflow-hidden cursor-pointer group"
>
  {#if thumbnail}
    <img
      src={thumbnail}
      alt={title}
      class="absolute inset-0 w-full h-full object-cover transition-opacity duration-300"
      class:opacity-0={isHovering && hasLoaded}
      loading="lazy"
    />
  {/if}

  <video
    bind:this={videoEl}
    loop muted playsinline
    preload="none"
    class="absolute inset-0 w-full h-full object-cover"
  ></video>

  <div class="absolute bottom-0 inset-x-0 bg-gradient-to-t from-black/90 via-black/50 to-transparent p-3">
    <p class="text-white text-sm font-medium truncate">{title}</p>
    {#if subtitle}
      <p class="text-gray-400 text-xs">{subtitle}</p>
    {/if}
  </div>

  {#if !isHovering}
    <div class="absolute inset-0 flex items-center justify-center">
      <div class="w-12 h-12 bg-black/50 rounded-full flex items-center justify-center backdrop-blur-sm">
        <span class="text-white text-lg ml-1">&#x25B6;</span>
      </div>
    </div>
  {/if}

  <div class="absolute top-2 right-2 bg-purple-600 text-white text-xs font-bold px-2 py-1 rounded-full opacity-0 group-hover:opacity-100 transition-opacity">
    Clip it
  </div>
</div>
```

**Integration in campaigns/+page.svelte:** Replace the existing video thumbnail grid (`{#each popularVideos as video}` block) to use `<VideoCard>` with `src={video.video_url || 'https://media.divine.video/' + video.id}`.

**Steps:**
- [ ] Create `VideoCard.svelte` with the code above
- [ ] Import and use in `campaigns/+page.svelte` — replace the `<a>` block inside `{#each popularVideos}` with `<a href="/submit?video={video.id}"><VideoCard src={video video_url} thumbnail={video.thumbnail} title={video.title} subtitle={video.author_name || truncatePubkey(video.author_pubkey)} /></a>`
- [ ] Add `video_url` to the `PopularVideo` interface and map it from the API response (`v.video_url`)
- [ ] Run `npm run check` — 0 errors
- [ ] Commit: `feat: add hover-to-play VideoCard component for campaign video grid`
- [ ] Build + deploy: `npm run build && wrangler pages deploy .svelte-kit/cloudflare --project-name clips-divine-video --branch main --commit-dirty=true`

---

## Task 2: Leaderboard Page + API

**Scope:** Backend API endpoint + frontend page.

**Backend files:**
- Create: `clipcrate/crates/api/src/leaderboard.rs`
- Modify: `clipcrate/crates/api/src/lib.rs` — add module + routes
- Modify: `clipcrate/crates/db/src/postgres.rs` — add leaderboard queries

**Frontend files:**
- Create: `clips-web/src/routes/leaderboard/+page.svelte`
- Modify: `clips-web/src/lib/api.ts` — add leaderboard endpoint
- Modify: `clips-web/src/routes/+layout.svelte` — add nav link

**Backend — leaderboard queries in postgres.rs:**

```rust
pub async fn get_leaderboard(
    pool: &PgPool,
    metric: &str,    // "earnings", "views", "best_clip"
    period: &str,    // "week", "month", "all"
    limit: i64,
) -> Result<Vec<LeaderboardEntry>> {
    let time_filter = match period {
        "week" => "AND p.created_at > NOW() - INTERVAL '7 days'",
        "month" => "AND p.created_at > NOW() - INTERVAL '30 days'",
        _ => "",
    };

    let query = match metric {
        "earnings" => format!(
            "SELECT c.pubkey, c.trust_level, COALESCE(SUM(p.amount_sats), 0) as value
             FROM clippers c LEFT JOIN payouts p ON c.pubkey = p.clipper_pubkey {time_filter}
             WHERE c.trust_level >= 2
             GROUP BY c.pubkey, c.trust_level
             ORDER BY value DESC LIMIT $1"
        ),
        "views" => format!(
            "SELECT c.pubkey, c.trust_level, COALESCE(SUM(s.total_verified_views), 0) as value
             FROM clippers c LEFT JOIN submissions s ON c.pubkey = s.clipper_pubkey {time_filter}
             WHERE c.trust_level >= 2
             GROUP BY c.pubkey, c.trust_level
             ORDER BY value DESC LIMIT $1"
        ),
        _ => format!(  // best_clip
            "SELECT s.clipper_pubkey as pubkey, c.trust_level, s.total_verified_views as value
             FROM submissions s JOIN clippers c ON s.clipper_pubkey = c.pubkey
             WHERE c.trust_level >= 2 AND s.status = 'verified' {time_filter}
             ORDER BY value DESC LIMIT $1"
        ),
    };

    sqlx::query_as::<_, LeaderboardEntry>(&query)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}
```

Add `LeaderboardEntry` to models.rs:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LeaderboardEntry {
    pub pubkey: String,
    pub trust_level: i32,
    pub value: i64,
}
```

**Backend — leaderboard handler:**
```rust
// clipcrate/crates/api/src/leaderboard.rs
pub async fn get_leaderboard(
    State(state): State<AppState>,
    Query(params): Query<LeaderboardParams>,
) -> Result<Json<Vec<LeaderboardEntry>>, ApiError> { ... }

pub async fn get_social_proof(
    State(state): State<AppState>,
) -> Result<Json<SocialProofResponse>, ApiError> { ... }
```

Routes: `GET /api/leaderboard?metric=earnings&period=week&limit=50` and `GET /api/stats/social-proof`.

**Frontend — leaderboard page:** Tabbed interface (Earnings / Views / Best Clip) with week/month/all toggle. Show rank, truncated npub, trust badge, and value. Social proof banner at top.

**Steps:**
- [ ] Add `LeaderboardEntry` to `clipcrate/crates/db/src/models.rs`
- [ ] Add leaderboard query to `clipcrate/crates/db/src/postgres.rs`
- [ ] Create `clipcrate/crates/api/src/leaderboard.rs` with handler + types
- [ ] Add `pub mod leaderboard;` and routes to `clipcrate/crates/api/src/lib.rs`
- [ ] Run `cd clipcrate && cargo check` — 0 errors
- [ ] Commit backend: `feat: add leaderboard API with earnings/views/best_clip metrics`
- [ ] Add `leaderboard` method to `clips-web/src/lib/api.ts`
- [ ] Create `clips-web/src/routes/leaderboard/+page.svelte` with tabbed UI
- [ ] Add "Leaderboard" to nav in `+layout.svelte`
- [ ] Run `npm run check` — 0 errors
- [ ] Commit frontend: `feat: add leaderboard page with tabs and social proof`
- [ ] Build + deploy both (Docker push + CF Pages deploy)

---

## Task 3: Auto-Campaigns (Nostr Subscription)

**Scope:** Backend Rust service. Subscribes to funnelcake relay for new DiVine videos (kind 34236), creates campaign rows automatically.

**Files:**
- Create: `clipcrate/crates/api/src/auto_campaigns.rs` — background task
- Create: `clipcrate/migrations/004_auto_campaigns.sql`
- Modify: `clipcrate/crates/db/src/postgres.rs` — add auto-campaign insert
- Modify: `clipcrate/crates/db/src/models.rs` — add `source` and `divine_video_event_id` fields
- Modify: `clipcrate/clipcrate/src/main.rs` — spawn auto-campaign background task

**Migration:**
```sql
-- clipcrate/migrations/004_auto_campaigns.sql
ALTER TABLE campaigns ADD COLUMN source TEXT NOT NULL DEFAULT 'manual';
ALTER TABLE campaigns ADD COLUMN divine_video_event_id TEXT;
CREATE UNIQUE INDEX idx_campaigns_divine_video ON campaigns(divine_video_event_id) WHERE divine_video_event_id IS NOT NULL;
```

**Auto-campaign background task:**
```rust
// clipcrate/crates/api/src/auto_campaigns.rs
use nostr_sdk::prelude::*;
use sqlx::PgPool;

const BASE_CPM_SATS: i32 = 1000;  // $1 CPM
const PER_VIDEO_BUDGET: i64 = 10000;  // 10K sats per auto-campaign
const RELAY_URL: &str = "wss://relay.divine.video";

pub async fn run_auto_campaign_listener(db: PgPool) {
    let keys = Keys::generate();  // Ephemeral keys for subscription
    let client = Client::new(keys);
    client.add_relay(RELAY_URL).await.ok();
    client.connect().await;

    let filter = Filter::new().kind(Kind::from(34236));
    client.subscribe(filter, None).await.ok();

    client.handle_notifications(|notification| async {
        if let RelayPoolNotification::Event { event, .. } = notification {
            if let Err(e) = handle_new_video(&db, &event).await {
                tracing::error!("auto-campaign error: {e:#}");
            }
        }
        Ok(false)  // don't stop
    }).await.ok();
}

async fn handle_new_video(db: &PgPool, event: &Event) -> anyhow::Result<()> {
    let event_id = event.id.to_hex();
    let creator = event.pubkey.to_hex();
    let title = event.tags.iter()
        .find(|t| t.as_slice().first().map(|s| s.as_str()) == Some("title"))
        .and_then(|t| t.as_slice().get(1))
        .map(|s| s.to_string())
        .unwrap_or_else(|| "DiVine Loop".to_string());
    let d_tag = event.tags.iter()
        .find(|t| t.as_slice().first().map(|s| s.as_str()) == Some("d"))
        .and_then(|t| t.as_slice().get(1))
        .map(|s| s.to_string())
        .unwrap_or_default();

    // Insert auto-campaign (ON CONFLICT = skip if already exists)
    sqlx::query(
        "INSERT INTO campaigns (creator_pubkey, title, budget_total_sats, budget_remaining_sats, cpm_sats, status, source, divine_video_event_id, content_refs, target_platforms)
         VALUES ($1, $2, $3, $3, $4, 'active', 'auto', $5, ARRAY[$6], ARRAY['tiktok','instagram','youtube','x'])
         ON CONFLICT (divine_video_event_id) DO NOTHING"
    )
    .bind(&creator)
    .bind(format!("Clip: {}", &title))
    .bind(PER_VIDEO_BUDGET)
    .bind(BASE_CPM_SATS)
    .bind(&event_id)
    .bind(&d_tag)
    .execute(db)
    .await?;

    tracing::info!(event_id, creator, title, "auto-campaign created");
    Ok(())
}
```

**main.rs change:** Spawn the listener as a background task:
```rust
let db_for_auto = db.clone();
tokio::spawn(async move {
    clipcrate_api::auto_campaigns::run_auto_campaign_listener(db_for_auto).await;
});
```

**Steps:**
- [ ] Create migration `004_auto_campaigns.sql`
- [ ] Update `models.rs` — add `source: String` and `divine_video_event_id: Option<String>` to Campaign
- [ ] Add auto-campaign insert function to `postgres.rs`
- [ ] Create `auto_campaigns.rs` with the Nostr subscription background task
- [ ] Add `pub mod auto_campaigns;` to `lib.rs`
- [ ] Update `main.rs` to spawn the listener
- [ ] Run `cargo check` — 0 errors
- [ ] Commit: `feat: add auto-campaign creation from Nostr relay subscription`
- [ ] Rebuild Docker image and push

---

## Task 4: Analytics with LayerCake Charts

**Scope:** Backend API endpoints + frontend analytics page.

**Backend files:**
- Create: `clipcrate/crates/api/src/analytics.rs`
- Modify: `clipcrate/crates/db/src/clickhouse.rs` — add analytics queries
- Modify: `clipcrate/crates/api/src/lib.rs` — add module + routes

**Frontend files:**
- Create: `clips-web/src/lib/components/ViewsChart.svelte`
- Create: `clips-web/src/lib/components/EarningsChart.svelte`
- Create: `clips-web/src/routes/dashboard/analytics/+page.svelte`
- Modify: `clips-web/src/lib/api.ts` — add analytics endpoints

**Install LayerCake:**
```bash
cd clips-web && npm install layercake
```

**Backend — analytics endpoints:**
- `GET /api/analytics/submission/{id}` — per-clip time-series (snapshots + payouts)
- `GET /api/analytics/overview` — aggregated clipper stats (by platform, daily views)

**Frontend — charts using LayerCake:**
```svelte
<!-- ViewsChart.svelte (simplified) -->
<script lang="ts">
  import { LayerCake, Svg } from 'layercake';
  import Line from './Line.svelte';
  import AxisX from './AxisX.svelte';
  import AxisY from './AxisY.svelte';

  let { data }: { data: { date: string; views: number }[] } = $props();
</script>

<div class="h-48">
  <LayerCake
    x="date" y="views"
    {data}
    padding={{ top: 8, right: 8, bottom: 20, left: 40 }}
  >
    <Svg>
      <AxisX />
      <AxisY />
      <Line color="#a855f7" />
    </Svg>
  </LayerCake>
</div>
```

**Steps:**
- [ ] Add ClickHouse analytics queries to `clickhouse.rs`
- [ ] Create `analytics.rs` with handler + types
- [ ] Add routes to `lib.rs`: `/api/analytics/submission/{id}` and `/api/analytics/overview`
- [ ] Run `cargo check` — 0 errors
- [ ] Commit backend: `feat: add analytics API for per-clip and overview stats`
- [ ] Install LayerCake: `cd clips-web && npm install layercake`
- [ ] Create chart components: `ViewsChart.svelte`, `EarningsChart.svelte`
- [ ] Create `dashboard/analytics/+page.svelte` with charts
- [ ] Add link from dashboard to analytics
- [ ] Run `npm run check` — 0 errors
- [ ] Commit frontend: `feat: add analytics charts with LayerCake`
- [ ] Deploy both

---

## Task 5: Web Push Notifications

**Scope:** Backend notification dispatch + frontend service worker + subscription.

**Backend files:**
- Create: `clipcrate/crates/notifications/Cargo.toml` + `src/lib.rs`
- Create: `clipcrate/crates/notifications/src/web_push.rs`
- Create: `clipcrate/crates/api/src/notifications.rs`
- Modify: `clipcrate/Cargo.toml` — add notifications crate to workspace
- Modify: `clipcrate/crates/api/src/lib.rs` — add routes
- Modify: `clipcrate/crates/db/src/postgres.rs` — add push_subscriptions table
- Create: `clipcrate/migrations/005_push_subscriptions.sql`

**Frontend files:**
- Create: `clips-web/static/sw.js` — service worker for push
- Create: `clips-web/src/lib/push.ts` — push subscription helper
- Modify: `clips-web/src/routes/+layout.svelte` — register SW on mount

**Migration:**
```sql
-- clipcrate/migrations/005_push_subscriptions.sql
CREATE TABLE push_subscriptions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    clipper_pubkey TEXT NOT NULL,
    endpoint TEXT NOT NULL UNIQUE,
    p256dh TEXT NOT NULL,
    auth TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_push_sub_clipper ON push_subscriptions(clipper_pubkey);
```

**Backend — web push crate:**
```toml
# clipcrate/crates/notifications/Cargo.toml
[dependencies]
web-push = "0.10"
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
tracing.workspace = true
sqlx.workspace = true
```

**Frontend — service worker:**
```javascript
// clips-web/static/sw.js
self.addEventListener('push', (event) => {
  const data = event.data?.json() || {};
  event.waitUntil(
    self.registration.showNotification(data.title || 'DiVine Clips', {
      body: data.body || '',
      icon: '/favicon.png',
      data: { url: data.url || '/' },
    })
  );
});
self.addEventListener('notificationclick', (event) => {
  event.notification.close();
  event.waitUntil(clients.openWindow(event.notification.data.url));
});
```

**API routes:**
- `POST /api/notifications/subscribe` — store push subscription
- `POST /api/notifications/unsubscribe` — remove subscription
- `GET /api/notifications/vapid-key` — return public VAPID key

**Steps:**
- [ ] Create migration `005_push_subscriptions.sql`
- [ ] Create `notifications` crate with web_push module
- [ ] Add `notifications.rs` to API crate with subscribe/unsubscribe handlers
- [ ] Add routes to `lib.rs`
- [ ] Run `cargo check` — 0 errors
- [ ] Commit backend: `feat: add Web Push notification infrastructure`
- [ ] Create `clips-web/static/sw.js`
- [ ] Create `clips-web/src/lib/push.ts` with subscribe/unsubscribe helpers
- [ ] Register service worker in `+layout.svelte` on mount
- [ ] Add notification permission prompt (bell icon in nav)
- [ ] Run `npm run check` — 0 errors
- [ ] Commit frontend: `feat: add Web Push service worker and subscription UI`
- [ ] Deploy both

---

## Task 6: CDK Mint + Phoenixd Lightning Backend

**Scope:** Infrastructure + backend. Stand up cdk-mintd with Phoenixd as the Lightning backend.

**Note:** The `clipcrate-cashu` crate already has CDK 0.16 wallet integration (`cashu_wallet.rs`). This task is about running the MINT side (cdk-mintd) and connecting it to a real Lightning node (Phoenixd).

**Files:**
- Create: `clipcrate/cdk-mint/Dockerfile` — cdk-mintd container
- Create: `clipcrate/cdk-mint/config.toml` — mint configuration
- Modify: `docker-compose.yml` — add phoenixd + cdk-mintd services
- Modify: `clipcrate/crates/cashu/src/cashu_wallet.rs` — point to real mint URL instead of testnut

**docker-compose additions:**
```yaml
  phoenixd:
    image: ghcr.io/acinq/phoenixd:latest
    environment:
      PHOENIXD_DATADIR: /data
    volumes:
      - phoenixd_data:/data
    ports:
      - "9740:9740"

  cdk-mintd:
    image: ghcr.io/cashubtc/cdk-mintd:latest
    environment:
      CDK_MINTD_LN_BACKEND: phoenixd
      CDK_MINTD_PHOENIXD_URL: http://phoenixd:9740
      CDK_MINTD_DB_PATH: /data/mint.db
      CDK_MINTD_LISTEN_ADDR: 0.0.0.0
      CDK_MINTD_LISTEN_PORT: 3338
    volumes:
      - mint_data:/data
    ports:
      - "3338:3338"
    depends_on:
      - phoenixd
```

**K8s manifests (for production):**
- Create: `k8s/applications/clipcrate/base/phoenixd-deployment.yaml`
- Create: `k8s/applications/clipcrate/base/cdk-mintd-deployment.yaml`
- Modify: `k8s/applications/clipcrate/base/kustomization.yaml` — add new resources

**Update cashu_wallet.rs:** Change `MINT_URL` from `testnut.cashu.space` to `http://cdk-mintd:3338` (or env var `CDK_MINT_URL`).

**Steps:**
- [ ] Create Phoenixd + cdk-mintd entries in `docker-compose.yml`
- [ ] Update `cashu_wallet.rs` to read mint URL from `CDK_MINT_URL` env var
- [ ] Test locally: `docker-compose up phoenixd cdk-mintd` — verify mint responds on :3338
- [ ] Commit: `feat: add Phoenixd + cdk-mintd infrastructure for real Lightning payments`
- [ ] Create K8s manifests for GKE deployment (phoenixd + cdk-mintd as sidecar containers in clipcrate namespace)
- [ ] Push to divine-iac-coreconfig
- [ ] Rebuild + push clipcrate Docker image with updated mint URL

---

## Parallel Execution Map

```
Task 1 (Video Player)     — Frontend only, clips-web/
Task 2 (Leaderboard)      — Backend (crates/api/leaderboard.rs) + Frontend (routes/leaderboard/)
Task 3 (Auto-Campaigns)   — Backend only (crates/api/auto_campaigns.rs + main.rs)
Task 4 (Analytics)         — Backend (crates/api/analytics.rs) + Frontend (routes/dashboard/analytics/)
Task 5 (Web Push)          — Backend (new crate + API) + Frontend (sw.js + push.ts)
Task 6 (CDK Mint)          — Infrastructure (docker-compose, k8s) + Backend config change
```

**No conflicts between tasks** — each touches different files. All 6 can run in parallel.

**Build order for deployment:**
1. Tasks 1 (frontend-only) → deploy to CF Pages immediately
2. Tasks 2-5 (backend changes) → rebuild single Docker image with all changes → push
3. Task 6 (infrastructure) → separate deployment of phoenixd + cdk-mintd
