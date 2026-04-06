# DiVine Clips v2 — Research Findings

**Date:** 2026-04-07
**Status:** Complete
**Method:** 8 parallel research agents covering all feature gaps

---

## Executive Summary

### Key Decisions

| Area | Recommendation |
|------|---------------|
| **Payments** | CDK (not Moksha) — already in codebase as `cdk = "0.16"`. Phoenixd for Lightning backend. |
| **View verification** | YouTube direct API (free, 500K videos/day). Phyllo ($199/mo) for TikTok, Instagram, X. |
| **Video player** | Native `<video loop muted playsinline>` with hover-to-play + Intersection Observer. No library needed. |
| **Auto-campaigns** | Real DB rows with `source='auto'`. Nostr REQ subscription for new kind 34236 events. |
| **Analytics charts** | LayerCake (Svelte-native, 8KB gzipped). Three charts: views over time, earnings, platform breakdown. |
| **Leaderboard** | Three metrics (earnings, views, best clip) x three periods (week, month, all time). Trust Level 2+ to appear. |
| **Notifications** | Web Push (primary), Telegram bot (secondary), email digest (weekly). Custom built, not Novu/Knock. |
| **Compilations** | Primary source pays model. Optional `source_refs[]` for additional tagged videos. |
| **Community** | Discord + Telegram at launch, in-app showcase + stories by month 3, Nostr-native by month 6. |
| **Referrals** | 5% of referred clipper's payouts for 6 months, capped at 50K sats. Activates at Trust Level 2. |
| **Content policy** | No mandatory branding. Prohibit deepfakes, misleading edits, hate speech. Trust-level gating as enforcement. |
| **C2PA** | Platforms strip metadata. Build verification page linking clips back to C2PA-signed sources. |

---

## 1. Payments — CDK + Phoenixd

**Mint:** CDK (cdk-mintd) is the right choice. Moksha is stale (no updates since mid-2024). LNbits Cashu extension is dead (archived July 2025). The codebase already uses `cdk = "0.16"` for wallet operations — run cdk-mintd as the mint (same library, same Rust toolchain).

**Lightning backend:** Phoenixd for MVP — handles liquidity automatically, simple HTTP API, self-custodial. Upgrade path: Voltage-hosted LND when volume grows.

**Payout UX:** Cashu balance accrual → clipper withdraws via Lightning invoice or LNURL. Non-crypto users see "sats earned" as a balance with a withdraw button. LNURL-withdraw as redemption method.

**Regulatory:** Operating a Cashu mint is legally gray (money transmission). Mitigate with: small balance caps, encourage frequent withdrawal, consult Bitcoin-native attorney before launch.

**Implementation:** Replace Moksha references in spec with CDK. Stand up cdk-mintd alongside clipcrate. Phoenixd as a sidecar container in GKE.

---

## 2. Platform APIs

| Platform | Method | Auth | Rate Limit | View Counts? | Approach |
|----------|--------|------|-----------|-------------|----------|
| **YouTube** | Data API v3 | API key only | 500K videos/day | Yes | **Direct API from day 1** |
| **TikTok** | Display API | OAuth per user | 100 req/min | Yes (play_count) | **Phyllo MVP**, direct Phase 2 |
| **Instagram** | Graph API | Business/Creator OAuth | 200 calls/user/hr | Yes | **Phyllo only** (requires Business account) |
| **X** | API v2 | Bearer token ($100+/mo) | 10K-1M tweets/mo | Yes (impression_count) | **Phyllo MVP** |

**Key insight:** YouTube is free and generous — no user OAuth needed. TikTok and Instagram both require the clip poster to OAuth-connect their account regardless of whether we use Phyllo or direct API. Instagram is hardest — no public stats API, requires Business/Creator account.

**Phyllo:** $199/mo, covers all platforms, handles OAuth complexity. Best for MVP. Sign up for trial, test with real accounts.

---

## 3. Competitive Landscape

**Vyro:** $0.30-$1.00 CPM, monthly PayPal payments (30-45 day lag), application-based. Top complaint: opaque view counting, sudden CPM drops.

**Whop:** Campaign/bounty marketplace, weekly payouts via Stripe/crypto. Strengths: diverse creators, transparent terms. Weakness: semi-manual verification.

**Clipping:** Discord-first, 23K editors, $0.30-$1.50 CPM. Strengths: community, mentorship. Weakness: spreadsheet ops, doesn't scale.

**ClipFarm:** Long-term creator partnerships, 40B+ views. High barrier to entry.

**DiVine Clips differentiators:**
1. **Transparency** — real-time auditable analytics (biggest complaint about competitors is opaque counting)
2. **Payment speed** — target 7-day cycles vs industry 30-45 days
3. **No approval barrier** — anyone can clip at base rate
4. **Portable identity** — Nostr npub, not locked to a platform
5. **C2PA provenance** — verifiable human-made content

**Competitive CPM floor:** $0.50 minimum to attract quality clippers. $1.00 base rate is strong.

---

## 4. Video Player

**Blossom findings:** Videos are MP4 (H.264), ~9MB for 6 seconds at 1080p. CORS fully open (`*`). Range requests supported. Immutable caching (`max-age=31536000`). CDN via Varnish/Fastly.

**Recommendation:** Hover-to-play grid using native `<video>` elements:
- `preload="none"` — no src until hover/tap (avoids loading 216MB for 24 cards)
- Intersection Observer with 200px rootMargin — free memory for off-screen videos
- Thumbnail overlay fades out when video starts
- Mobile: `muted` + `playsinline` for iOS autoplay
- No third-party player library needed

**Component:** `VideoCard.svelte` with `use:observe` directive, hover/touch handlers, lazy src assignment.

---

## 5. Auto-Campaigns

**Sync mechanism:** Nostr REQ subscription for kind 34236 from funnelcake relay. Protocol-native, no REST API coupling.

**Data model:** Real rows in `campaigns` table with `source = 'auto'` column. Each new DiVine video creates a campaign at base CPM (1,000 sats/1K views). Unique constraint on `divine_video_event_id` prevents duplicates.

**Budget:** Fixed monthly pool (e.g., 1M sats/month). Per-video cap of 10,000 sats (~10K views). When pool exhausts, new auto-campaigns created as `paused`. Gorse trending scores can modulate: top-10% content gets 2x CPM.

**Fraud safeguards:**
- Self-clipping prevention: reject if clipper pubkey = creator pubkey
- Cross-reference Phyllo social account against external video poster
- Existing velocity and pattern checks apply
- New clippers limited to 5,000 weekly views on auto-campaigns specifically
- Require at least 1 completed manual campaign before auto-campaign payouts activate

---

## 6. Analytics

**Chart library:** LayerCake — Svelte-native, 8KB gzipped, composable SVG/Canvas layers. Fits the minimal stack without bloat.

**Three key visualizations:**
1. **View count over time** — line chart from `verification_snapshots` grouped by day
2. **Earnings accumulation** — stacked area chart from `payouts` table
3. **Platform breakdown** — horizontal bar chart from submissions joined with snapshots

**API endpoints:**
- `GET /api/analytics/submission/{id}` — per-clip time-series (snapshots + payouts)
- `GET /api/analytics/overview` — aggregated clipper stats (by platform, daily views)

**Batched, not real-time.** Verification polls on schedule (24h, 48h, 7d, 30d). Client-side 5-minute cache.

---

## 7. Leaderboard

**Metrics:** Top Earners (total sats), Top Viewers (total views), Best Single Clip (highest-view submission).

**Time periods:** This week, this month, all time.

**Anti-gaming:** Only `verified` submissions count. Trust Level 2+ required to appear on public board. Flagged submissions excluded. Existing fraud pipeline applies.

**Social proof elements:**
- Banner: "X clippers earned Y sats this week"
- Recently-paid feed via SSE
- New Clipper of the Week spotlight
- Badges: First Payout, 1K/10K/100K/1M Views, Trust Level upgrades

**API:** `GET /api/leaderboard?metric=earnings|views|best_clip&period=week|month|all&limit=50` — cacheable, 5-min TTL.

---

## 8. Notifications

**Channels (priority order):**
1. **Web Push** (primary) — service worker on clips.divine.video, push sent from clipcrate backend via VAPID
2. **Telegram bot** (secondary) — clipper links account via `/start` deep link
3. **Email** (weekly digest) — Resend or Postmark, Rust SDKs available
4. **Nostr DMs** (future) — NIP-17, Phase 2 when clipper base is Nostr-native

**Triggers:** Payout credited, view milestone (1K/10K/100K), new matching campaign, trust level upgrade, campaign expiring, weekly summary.

**Infrastructure:** Custom `notifications` crate in clipcrate. `NotificationDispatcher` trait with WebPush/Telegram/Email implementations. NATS for event decoupling.

**Web Push on Cloudflare Pages:** Confirmed feasible. Static SW served by Pages, subscriptions stored in clipcrate, pushes sent from Rust backend.

---

## 9. Compilations

**Model:** Primary Source Pays. Clipper picks one campaign for payout. Additional videos tagged via optional `source_refs TEXT[]` on submissions (analytics credit only, no payout).

**UX:** Simple "tag additional sources" field during submission. No multi-select wizard.

**Schema:** Add `source_refs TEXT[]` to submissions table. Add corresponding `a` tags to kind 30403 event.

**Future:** Compilation bonus multiplier (e.g., 1.2x CPM for 3+ sources). Weighted attribution deferred indefinitely.

---

## 10. Community

**Month 1-3:** Discord server + Telegram group (where clippers already live).

**Month 2-4:** In-app features:
- `/showcase` — top clips of the week (auto-populated from ClickHouse)
- `/stories` — curated success stories (kind 30023 long-form on Nostr)
- Activity feed enrichment — trust level upgrades and milestones as feed events

**Month 6+:** Nostr-native NIP-72 community. Bridge Discord messages to Nostr.

---

## 11. Referrals

**Link:** `clips.divine.video/r/<npub-short>` (first 12 chars, collision-checked).

**Reward:** 5% of referred clipper's payouts for 6 months, capped at 50K sats per referral. Funded from platform referral budget (not deducted from campaigns or referred clipper).

**Anti-fraud:** Same-device fingerprinting (IP + UA hash). Rewards activate only at Trust Level 2 (100K views + 30 days).

---

## 12. Content Guidelines & C2PA

**Content policy:** No mandatory branding or CTAs. Prohibit: deepfakes, misleading edits, NSFW additions, hate speech, bot views. Enforcement via trust-level gating + community reporting + manual review queue.

**C2PA finding:** TikTok detects C2PA on upload (for AI labels) but strips on re-share. Instagram, YouTube, X have no C2PA preservation. The manifest does NOT survive cross-platform posting.

**Solution:** Build `clips.divine.video/verify` — a verification page that links any clip back to its C2PA-signed DiVine source. The clipcrate submission database IS the binding layer between the external clip and the verified source.

**Future:** C2PA 2.1 Soft Binding (imperceptible watermarks) could survive transcoding. Digimarc-style watermarks embedded before download. Phase 2+.

**Disputes:** 48-hour first response target. ClickHouse time-series as source of truth. Manual re-check once per submission per 7 days. Graduated consequences: warning → rejection → suspension → ban.

---

## Implementation Priority

Based on research findings, recommended build order:

1. **Video player** (1 day) — immediate UX improvement, no backend changes
2. **Auto-campaigns** (2 days) — fills the empty campaigns page with real content
3. **CDK mint + Phoenixd** (3 days) — real payments, the biggest functional gap
4. **Leaderboard + social proof** (1 day) — drives engagement
5. **Analytics charts** (2 days) — clipper retention
6. **Web Push notifications** (2 days) — re-engagement
7. **Compilation support** (0.5 day) — schema addition
8. **Referral system** (1 day) — growth
9. **Content guidelines page** (0.5 day) — policy
10. **C2PA verification page** (1 day) — differentiator
11. **Telegram bot** (1 day) — secondary notification channel
12. **Discord/community setup** (0.5 day) — launch community
