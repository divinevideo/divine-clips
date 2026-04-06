# DiVine Clips v2 — Research Plan

**Goal:** Close the feature gaps between DiVine Clips and established clipping platforms (Vyro, Clipping, Whop, ClipFarm). Research each gap area to inform implementation.

---

## 1. Video Playback & Content Preview

**Question:** How should we embed 6-second Blossom-hosted videos in the campaigns/browse UI?

**Research tasks:**
- Test Blossom video URLs in a `<video>` tag — do they support range requests? CORS headers?
- Check if DiVine videos are MP4 or WebM — what codec/container?
- Look at how divine-web (Flutter) renders video loops — what player/approach?
- Evaluate lightweight video players: native `<video loop autoplay muted>` vs. hls.js vs. plyr
- Test mobile performance with a grid of 12+ autoplaying 6-second loops
- Check CDN caching — will Blossom URLs be fast enough for thumbnail-to-video transitions?

**Output:** Recommended approach for inline video preview on campaign cards (autoplay on hover? click to play? always looping?)

---

## 2. Real Payment Infrastructure (Cashu/Lightning)

**Question:** What's the fastest path to real Bitcoin micropayments for clippers?

**Research tasks:**
- Evaluate Cashu mint options:
  - **Moksha** (Rust) — maturity, API stability, embeddability
  - **CDK** (cdk-mintd) — Cashu Development Kit, more actively maintained?
  - **Nutshell** (Python) — most battle-tested, but different language
  - **Cashu.me** — hosted mint as a service?
- Research NUT specifications we need: NUT-00 (tokens), NUT-03 (swap), NUT-04 (mint), NUT-05 (melt)
- Evaluate Lightning backend options: LND vs CLN vs LDK vs Phoenixd vs hosted (Voltage, Start9)
- Research Cashu wallet UX — how do non-crypto users redeem tokens?
  - Can we generate a QR code that opens in Wallet of Satoshi?
  - LNURL-withdraw as an alternative to Cashu?
- Check how Stacker News, Fountain, and Wavlake handle Lightning payouts
- Research regulatory implications of operating a Cashu mint (is it money transmission?)

**Output:** Decision: which mint + Lightning backend to use, hosted vs self-hosted, and the payout UX flow

---

## 3. Auto-Campaigns (Base Rate for All Content)

**Question:** How do we make every DiVine video clippable without manual campaign creation?

**Research tasks:**
- Design the "platform campaign" concept — a system-level campaign funded by DiVine itself
- How should budget be managed? Fixed monthly allocation? Per-video cap? Dynamic based on content quality?
- Should Gorse recommendation scores influence the base rate? (better content = higher CPM?)
- How to sync with funnelcake — listen for new kind 34236 events and auto-create campaigns?
  - NATS subscription from clipcrate to funnelcake?
  - Periodic polling of `/api/videos?sort=recent`?
  - Nostr relay subscription (REQ filter for kind 34236)?
- What's the fraud risk of auto-campaigns? (clippers posting their own DiVine videos to farm sats)
- Look at how YouTube handles monetization for all videos (Partner Program thresholds)

**Output:** Design for auto-campaign system: funding model, sync mechanism, fraud safeguards

---

## 4. View Analytics & Performance Graphs

**Question:** What data do clippers need to see about their clip performance?

**Research tasks:**
- Map out the data we already have in ClickHouse (verification_snapshots: submission_id, timestamp, view_count)
- What charts/visualizations do Vyro and Whop show clippers?
  - View count over time (line chart)
  - Views by platform breakdown (pie/bar chart)
  - Earnings accumulation graph
  - Comparison to other clips
- Evaluate lightweight charting libraries for SvelteKit:
  - **Chart.js** — most popular, heavy
  - **Layercake** — Svelte-native, composable
  - **uPlot** — lightweight, performant
  - **Pancake** (from Svelte team) — experimental
- What's the query pattern for ClickHouse? Time-series aggregation per submission
- Should analytics be served from clipcrate (ClickHouse queries) or a separate analytics endpoint?

**Output:** Chart library recommendation, API endpoint spec for analytics data, mockup of clipper analytics view

---

## 5. Leaderboard & Social Proof

**Question:** How do we create competitive/aspirational energy among clippers?

**Research tasks:**
- Study Whop's leaderboard UI — what metrics? Time periods? Anonymity?
- What should DiVine Clips rank by?
  - Total views generated (this week / month / all time)
  - Total sats earned
  - Number of clips submitted
  - Best single clip performance
- Should leaderboards be public or only visible to authenticated clippers?
- How do existing platforms handle gaming? (fake views → inflated leaderboard)
- Research "social proof" patterns that drive engagement:
  - "X clippers earned Y sats this week"
  - "Top clip this week: Z views by @clipper"
  - New clipper welcome messages
- Should we show a "recently paid" feed? (like Stacker News's zap feed)

**Output:** Leaderboard spec: metrics, time periods, privacy settings, anti-gaming measures

---

## 6. Notifications & Engagement

**Question:** How do we keep clippers coming back?

**Research tasks:**
- What notification channels make sense for our audience?
  - **Email** — universally accessible, async
  - **Nostr DMs** (NIP-04 or NIP-17) — on-protocol, but clippers may not use Nostr clients
  - **Push notifications** (Web Push API) — requires service worker setup
  - **Telegram/Discord bot** — where clippers already hang out
- What events should trigger notifications?
  - New campaign matching clipper's interests
  - View milestone hit (1K, 10K, 100K views)
  - Payout available / credited
  - Trust level upgrade
  - Campaign about to expire / budget running low
- How do Vyro/Whop handle notification preferences?
- Research Web Push API implementation in SvelteKit + Cloudflare Pages
- Look at Novu or Knock as notification infrastructure services

**Output:** Notification system design: channels, triggers, preferences, infrastructure recommendation

---

## 7. Multi-Clip Compilations

**Question:** How should we handle clippers who combine multiple DiVine videos into one post?

**Research tasks:**
- How common are compilations in the existing clipping ecosystem? (very common on TikTok/YouTube)
- Study Whop's approach to multi-source clips
- Design the UX: how does a clipper indicate "this TikTok contains videos from campaigns A, B, and C"?
  - Multi-select campaigns during submission?
  - Auto-detect from video analysis? (too complex for MVP)
  - Let clipper tag which videos are in the compilation?
- Attribution model: full views to each campaign? Split? Primary + secondary?
- How does this affect fraud detection? (one clip earning from 5 campaigns)
- Should compilations earn a bonus rate? (they're often higher quality)

**Output:** Compilation UX spec, attribution model decision, fraud implications

---

## 8. Community & Social Features

**Question:** How do we build the clipper community without Discord?

**Research tasks:**
- What community features do Whop/Clipping offer beyond Discord?
  - In-app chat, forums, tips sharing
  - Clipper-to-clipper messaging
  - "How I got 100K views" success stories
- Nostr-native community options:
  - NIP-72 community events
  - NIP-28 group chat
  - In-app comment/reaction system
- Should we have a Discord/Telegram as a bridge to get started, even if long-term we go Nostr-native?
- Research how Stacker News and Nostr apps build community engagement
- What's the minimum community feature set to launch? (probably just a tips/blog section)

**Output:** Community strategy: launch with Discord? Build Nostr-native? Minimum features?

---

## 9. Content Guidelines & Brand Safety

**Question:** How do we ensure clippers represent DiVine content well?

**Research tasks:**
- What guidelines do Vyro/Clipping enforce? (no misleading edits, keep attribution, no hate content)
- Should DiVine Clips have a content policy for clips? (separate from DiVine's C2PA human-only policy)
- How to detect if a clip violates guidelines? (manual review? automated?)
- Should creators be able to set per-campaign restrictions? (e.g., "no commentary", "must include @mention")
- How does C2PA provenance chain work for clips? Does the C2PA manifest survive being posted to TikTok?
- Research if TikTok/Instagram strip C2PA metadata from uploaded videos

**Output:** Content guidelines draft, enforcement approach (manual vs. automated), C2PA survival analysis

---

## 10. Referral & Growth System

**Question:** How do we grow the clipper pool virally?

**Research tasks:**
- Study Whop's referral program — what's the cut? How is it tracked?
- Referral model options:
  - Invite link → new clipper signs up → referrer gets X% of their earnings for Y months
  - Flat bonus per referral who reaches a milestone (e.g., 10K views)
  - Tiered: refer 5 clippers → unlock higher CPM
- How to prevent referral fraud? (self-referral with multiple accounts)
- Should DiVine Clips have an "ambassador" program for high-performing clippers?
- Research affiliate tracking in Nostr (is there a NIP for referrals? zap splits?)
- Look at how Fountain podcast app handles referral/growth incentives

**Output:** Referral program spec: mechanics, anti-fraud, implementation complexity

---

## 11. Platform API Deep Dive

**Question:** What's the real state of view count verification on each platform?

**Research tasks:**
- **TikTok Display API**: Apply for access, understand rate limits, what data is available
  - Does it return view counts for specific videos by URL?
  - Requirements: TikTok developer account, app approval (5-7 days)
  - Alternatives if rejected: Phyllo, unofficial APIs, scraping (ToS risk)
- **Instagram Graph API**: What's needed for Reels view counts?
  - Requires Business/Creator account connection via Meta OAuth
  - What permissions needed? `instagram_basic`, `instagram_insights`?
  - Rate limits and quotas
- **YouTube Data API v3**: Already working in our CF Worker. What are the quota limits?
  - Default: 10,000 units/day. Each video stats query = ~3 units
  - How many submissions can we verify per day? (~3,300)
  - Quota increase request process
- **X API v2**: Tweet metrics availability
  - Bearer token vs OAuth 2.0 user context
  - Rate limits for tweet lookup
  - Does it work for video view counts specifically?
- **Phyllo**: Real-world reliability assessment
  - Sign up for trial ($199/mo)
  - Test with real TikTok/Instagram accounts
  - Measure latency, accuracy, failure rate
  - What happens when Phyllo's integration breaks?

**Output:** Per-platform API access plan, rate limit analysis, Phyllo trial report

---

## 12. Competitive Intelligence

**Question:** What can we learn from the latest features and strategies of competitors?

**Research tasks:**
- Create accounts on Vyro, Whop (Content Rewards), and Clipping
- Document their complete clipper onboarding flow (screenshots/screen recordings)
- Map their feature set in detail: what do they show on dashboard, campaign pages, etc.
- What are clippers complaining about? (check Discord servers, Twitter, Reddit)
  - Common complaints: late payments, view count discrepancies, low CPM
- What CPM rates are currently competitive? (may have changed since initial research)
- How do they handle disputes? (clipper says "my video got 100K views but you only credited 50K")
- What's their creator-side experience? How easy is it to create a campaign?

**Output:** Competitive audit document with screenshots, feature comparison, pain points we can exploit

---

## Research Timeline

| Week | Focus Areas | Output |
|------|------------|--------|
| **Week 1** (Apr 7-11) | #2 Payments, #11 Platform APIs, #12 Competitive Intel | Cashu decision, API access applications, competitive audit |
| **Week 2** (Apr 14-18) | #1 Video Player, #3 Auto-Campaigns, #4 Analytics | Video embed approach, auto-campaign design, chart library |
| **Marfa Retreat** (Apr 18-25) | #5 Leaderboard, #7 Compilations, #8 Community | Workshop with team, validate designs |
| **Week 4** (Apr 28-May 2) | #6 Notifications, #9 Guidelines, #10 Referrals | Notification system, content policy, growth plan |

## How to Use This Plan

Each research area should produce a short document (1-2 pages) with:
1. **Findings** — what we learned
2. **Recommendation** — what to build
3. **Effort estimate** — how long to implement
4. **Dependencies** — what blocks this

These feed into the v2 implementation plan. The Marfa retreat is the natural checkpoint to review findings and prioritize the build order.
