# Claude Code Prompt: DiVine Clips — Clipping Marketplace

## Context

You are helping plan and build **DiVine Clips**, a clipping marketplace integrated into the DiVine platform (divine.video). DiVine is a Nostr-native 6-second looping video platform with C2PA content verification and a human-only content policy, built by andOtherStuff (a nonprofit collective). It's backed by Jack Dorsey.

DiVine Clips is a grassroots promotion tool that incentivizes people ("clippers") to distribute DiVine content across other social platforms (TikTok, Instagram Reels, YouTube Shorts, X) in exchange for Bitcoin Lightning micropayments (zaps) based on verified view counts. The 6-second looping format is distinctive enough that no branding or CTAs are needed — the content format itself drives curiosity and discovery back to DiVine.

## Existing Technical Stack

- **Protocol**: Nostr (DiVine content uses NIP-32222 addressable short video events)
- **Video storage**: Blossom (decentralized media storage)
- **CDN**: Currently evaluating (previously Bunny CDN)
- **Mobile app**: Flutter
- **Backend services**: Rust ("funnelcake" relay)
- **Analytics/Recommendations**: ClickHouse (analytics), NATS (messaging), Gorse (recommendation engine)
- **Content verification**: C2PA/CAWG content credentials (integration with Guardian Project's c2pa-rs)
- **Identity**: Nostr keypairs, with Keycast multi-tenant key custody system
- **Payments**: Bitcoin Lightning / zaps
- **Existing verifier service**: verifier.divine.video (needs TikTok, Instagram, YouTube API access for content provenance verification — same APIs needed for clip view tracking)

## What DiVine Clips Does

### The Core Loop

1. **Creators** post 6-second looping videos on DiVine
2. **Creators** (or DiVine platform itself) create **clip campaigns** — allocating a sats budget to promote specific content or their profile generally
3. **Clippers** browse available campaigns and select content to promote
4. **Clippers** download DiVine loops and post them (as-is, in compilations, or remixed) to their own TikTok, Instagram Reels, YouTube Shorts, or X accounts
5. **Clippers** submit links to their posted clips back to DiVine Clips
6. **DiVine Clips** verifies view counts on those clips via platform APIs
7. **Clippers** receive automated zap payouts based on verified views

### No Branding Required

The 6-second loop format is inherently distinctive — it IS the ad. Clippers don't need watermarks, overlays, or "download DiVine" CTAs. They just distribute the content. The format drives organic curiosity and discovery.

## Industry Reference: How Existing Clipping Platforms Work

Study these existing platforms for patterns and lessons:

- **Clipping** (by Anthony Fujiwara): 23,300 contract editors, operates via Discord, pays $300-$1,500 per million views, clients pay $2,500-$10,000/month subscription. Generated $7.7M in sales in 2025. Clients include MrBeast, IShowSpeed.
- **Whop**: Creator-focused platform with millions of clippers, claims $1.5B in cumulative sales. Has a Content Rewards program. Operates as a marketplace with "bounties" for clip campaigns.
- **Vyro** (MrBeast's platform): Pays $3 per 1,000 views, counts views across TikTok + Reels + Shorts combined. Campaign-based model.
- **ClipFarm** (by Airrack, via Whop): 40B+ views generated. Partnership model with brands including HBO Max.

Key takeaway: All existing platforms are centralized, use Discord for coordination, rely on manual or semi-automated view verification, and pay via traditional methods or crypto. DiVine Clips differentiates by being Nostr-native with Lightning payments, portable clipper identity/reputation, and integration with C2PA content provenance.

## Architecture Requirements

### Nostr-Native Design

Everything should be built on Nostr events where possible:

- **Clip campaigns** should be Nostr events (define a new kind or extend existing NIPs)
- **Clipper profiles/reputation** should be tied to Nostr identity (npub)
- **Campaign submissions** should be Nostr events referencing both the campaign and the external clip URL
- **Payout records** should be Nostr events for transparency
- **Collections/curation** already exists in DiVine via NIP-51 (kind 30004 curation sets, kind 30005 video sets) — DiVine Clips leverages this but doesn't duplicate it

### View Verification Pipeline

This is the hardest technical problem. Plan for a phased approach:

**Phase 1 (MVP)**: Use Phyllo (https://getphyllo.com) as an abstraction layer
- Phyllo starts at $199/month, covers TikTok, Instagram, YouTube, 20+ platforms
- Clipper connects their social accounts via Phyllo OAuth
- DiVine Clips queries Phyllo for view counts on submitted clip URLs
- Known limitation: Phyllo integrations can break, needs monitoring

**Phase 2**: Direct platform API integrations
- These are already needed for verifier.divine.video (C2PA provenance verification)
- TikTok: Display API + Research API (requires app approval, 5-7 day review)
- Instagram: Meta Graph API (requires Business/Creator account connection)
- YouTube: YouTube Data API v3 (most permissive, easiest to start with)
- X/Twitter: API v2

**Phase 3**: Fully self-hosted verification
- Replace Phyllo entirely with direct integrations
- Build fraud detection (fake views, bot detection)
- Cross-reference multiple data sources

### Payment System

- All payouts via Bitcoin Lightning zaps
- Campaign budgets held in escrow (could use Cashu ecash mints — see NIP-87)
- Automated payout triggers when view thresholds are met
- Support for different payout structures:
  - Per 1,000 views (CPM model, like Vyro)
  - Per confirmed install/signup (CPA model, harder but more valuable)
  - Flat bounty for specific campaigns
- Consider using NIP-47 (Nostr Wallet Connect) for automated zap payouts

### Clipper Identity & Reputation

- Clipper signs up with Nostr keypair
- Connects external social accounts (for view verification)
- Reputation score builds over time based on:
  - Total verified views generated
  - Campaign completion rate
  - Content quality signals (do their clips actually drive DiVine signups?)
  - Account age and consistency
- Reputation is portable — it's attached to their npub, not locked in a platform
- Higher reputation clippers could get access to premium campaigns or better rates

## Data Model (Nostr Events)

Design the Nostr event schemas for:

### Clip Campaign Event
- Creator's pubkey
- Reference to DiVine content being promoted (NIP-32222 event reference)
- Budget (in sats)
- Payout rate (sats per 1,000 views, or other model)
- Campaign duration / expiry
- Target platforms (TikTok, Reels, Shorts, X, or all)
- Campaign status (active, paused, completed, exhausted)
- Any content guidelines (optional — remember, no branding required by default)

### Clip Submission Event
- Clipper's pubkey
- Reference to the campaign event
- External URL(s) where the clip was posted
- Platform identifier (tiktok, instagram, youtube, x)
- Timestamp of submission
- Status (pending verification, verified, rejected)

### Verification Event
- Reference to the submission event
- Verified view count at time of check
- Verification timestamp
- Verification method (phyllo, direct_api, manual)
- Cumulative views (updated periodically)

### Payout Event
- Reference to the submission event
- Amount paid (sats)
- Lightning payment proof
- Views at time of payout

## MVP Scope

For the first version, keep it minimal:

1. **Campaign creation UI** — a "Promote" tab within divine.video where creators can set a budget and select content to promote
2. **Campaign discovery feed** — clippers browse active campaigns
3. **Content download** — clippers can download the source video(s) from a campaign
4. **Submission flow** — clipper posts to external platform, submits the link back
5. **View tracking** — periodic polling of view counts via Phyllo
6. **Payout automation** — zaps sent when view thresholds are met
7. **Basic clipper profile** — shows their connected accounts, total views generated, earnings

### What's NOT in MVP
- Compilation/remix tools (clippers use their own editing tools)
- AI-powered clip suggestions
- Advanced fraud detection (start with manual review for suspicious activity)
- CPA (cost per acquisition) tracking
- Multi-creator campaigns
- Clipper leaderboards (add later for gamification)

## Key Questions to Resolve in Planning

1. **Event kind numbers**: What kind numbers should we use for campaign, submission, verification, and payout events? Should these be new NIPs or use existing NIP-51 patterns?

2. **Campaign escrow**: How do we hold campaign budgets? Options include:
   - Cashu ecash mint (NIP-87)
   - Custodial wallet managed by DiVine
   - Multi-sig arrangement
   - Simple honor system for MVP (creator's wallet is debited per payout)

3. **Verification frequency**: How often do we poll view counts? Options:
   - On submission, then at 24h, 48h, 7d, 30d
   - Continuous polling every N hours
   - On-demand when clipper requests payout

4. **Fraud prevention**: What basic checks do we need from day one?
   - Minimum account age on external platforms
   - Suspicious view patterns (bought views)
   - Duplicate submissions across campaigns
   - Same content posted by multiple clippers

5. **Rate structure**: What's the default CPM? Research suggests:
   - Vyro: $3 per 1,000 views
   - Whop: $1.25 per 1,000 views (YouTube)
   - Clipping: $300-$1,500 per 1,000,000 views ($0.30-$1.50 CPM)
   - DiVine should probably start competitive to attract clippers

6. **Revolution.social integration**: The podcast also needs clipping. Should Revolution.social podcast episodes be promotable through the same system? This is long-form content (40+ min interviews) that gets clipped into short segments — a different workflow than promoting 6-second loops but could use the same infrastructure.

## Deliverables

Please produce:

1. **Technical architecture document** — system components, data flow, API integrations, Nostr event schemas
2. **Database/event schema** — detailed Nostr event structures for all event types
3. **API specification** — endpoints for campaign management, submission, verification, payouts
4. **Integration plan** — phased approach for Phyllo → direct API migration, with verifier.divine.video shared infrastructure
5. **MVP task breakdown** — prioritized list of implementation tasks with effort estimates
6. **Risk assessment** — technical risks, platform API risks, fraud risks, and mitigation strategies

## Additional Context

- The team is small (andOtherStuff nonprofit collective)
- Development resources are limited — lean heavily on existing Nostr infrastructure and avoid building what already exists
- The retreat in Marfa, Texas (April 18-25) could be a good time to workshop this with the team
- Jack Dorsey is a backer — Lightning/Bitcoin-native payments are aligned with the project's values
- The clipping marketplace should feel like a natural extension of DiVine, not a separate product
- Consider how this intersects with the existing Gorse recommendation engine — could recommendation signals help surface the most "clippable" content?
- The C2PA provenance chain is a unique selling point — clips carry their verification with them even on other platforms

## Start by...

1. Reading through this entire prompt carefully
2. Asking any clarifying questions before proceeding
3. Proposing a high-level architecture
4. Then drilling into each deliverable one at a time

Take your time. Think through the Nostr event design carefully — these schemas will be foundational and hard to change later. Consider edge cases and how existing NIPs can be leveraged rather than reinvented.
