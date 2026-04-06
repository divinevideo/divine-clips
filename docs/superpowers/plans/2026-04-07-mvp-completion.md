# DiVine Clips MVP Completion Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete all remaining gaps to make DiVine Clips a fully functional MVP where campaigns can be funded with real sats, clips can be submitted and verified, and clippers get paid.

**Architecture:** SvelteKit frontend on Cloudflare Pages, Rust/Axum API server with Postgres + ClickHouse, Cashu wallet via CDK for payments, Cloudflare Worker for view verification. Most backend code exists — gaps are primarily UI wiring, deployment, and integration testing.

**Tech Stack:** Rust/Axum, SvelteKit 5, CDK (Cashu), Cloudflare Pages/Workers, PostgreSQL, ClickHouse, Docker

---

## Chunk 1: Campaign Funding UI

The backend has `POST /api/campaigns/:id/fund` and `GET /api/campaigns/:id/fund/:quote_id` but the campaign detail page has no funding button.

### Task 1: Add Fund Campaign button and Lightning invoice modal

**Files:**
- Modify: `clips-web/src/routes/campaigns/[id]/+page.svelte`
- Modify: `clips-web/src/lib/api.ts` (add `campaigns.fund()` and `campaigns.checkFunding()` if missing)

- [ ] **Step 1: Check api.ts for funding methods**

Read `clips-web/src/lib/api.ts` and verify `campaigns.fund(id, amount_sats)` and `campaigns.checkFunding(id, quoteId)` exist. If not, add:

```typescript
fund: (id: string, amount_sats: number) =>
  apiFetch<{ invoice: string; quote_id: string; amount_sats: number }>(
    `/api/campaigns/${id}/fund`, { method: 'POST', body: JSON.stringify({ amount_sats }) }
  ),
checkFunding: (id: string, quoteId: string) =>
  apiFetch<{ paid: boolean; amount_credited_sats: number | null }>(
    `/api/campaigns/${id}/fund/${quoteId}`
  ),
```

- [ ] **Step 2: Add funding UI to campaign detail page**

In `clips-web/src/routes/campaigns/[id]/+page.svelte`, add state variables:

```typescript
let showFundModal = $state(false);
let fundAmount = $state(1000);
let fundInvoice = $state('');
let fundQuoteId = $state('');
let fundStatus = $state<'idle' | 'invoice' | 'polling' | 'done' | 'error'>('idle');
let fundError = $state('');
```

Add a "Fund Campaign" button next to (or below) the "Claim & Clip" button:

```svelte
{#if $isAuthenticated}
  <button
    onclick={() => { showFundModal = true; }}
    class="block w-full text-center bg-green-600 hover:bg-green-500 text-white font-semibold py-3 rounded-xl transition-colors mt-3"
  >
    Fund Campaign
  </button>
{/if}
```

- [ ] **Step 3: Add funding modal with invoice display**

Add a modal that: (a) lets user enter amount, (b) calls `api.campaigns.fund()`, (c) displays the Lightning invoice as a copyable string, (d) polls `api.campaigns.checkFunding()` every 3 seconds until paid, (e) shows success with amount credited.

Include a QR code of the invoice (use a simple text display + copy button for MVP — QR can come later).

```svelte
{#if showFundModal}
  <div class="fixed inset-0 bg-black/70 z-50 flex items-center justify-center p-4">
    <div class="bg-gray-900 rounded-2xl max-w-md w-full p-6">
      <h3 class="text-xl font-bold text-white mb-4">Fund Campaign</h3>

      {#if fundStatus === 'idle'}
        <label class="block text-sm text-gray-400 mb-1">Amount (sats)</label>
        <input type="number" bind:value={fundAmount} min="1"
          class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-3 text-white mb-4" />
        <button onclick={handleFund}
          class="w-full bg-green-600 hover:bg-green-500 text-white font-semibold py-3 rounded-xl">
          Get Lightning Invoice
        </button>

      {:else if fundStatus === 'invoice' || fundStatus === 'polling'}
        <p class="text-gray-400 text-sm mb-2">Pay this Lightning invoice:</p>
        <div class="bg-gray-800 rounded-lg p-3 break-all text-xs text-purple-300 font-mono mb-3 max-h-32 overflow-y-auto">
          {fundInvoice}
        </div>
        <button onclick={() => navigator.clipboard.writeText(fundInvoice)}
          class="w-full bg-gray-800 hover:bg-gray-700 text-gray-300 font-medium py-2 rounded-lg mb-3 text-sm">
          Copy Invoice
        </button>
        <div class="flex items-center gap-2 text-yellow-400 text-sm">
          <span class="w-4 h-4 border-2 border-yellow-400 border-t-transparent rounded-full animate-spin"></span>
          Waiting for payment...
        </div>

      {:else if fundStatus === 'done'}
        <div class="text-center py-4">
          <div class="text-green-400 text-lg font-bold mb-2">Payment received!</div>
          <p class="text-gray-400">Campaign funded successfully.</p>
        </div>

      {:else if fundStatus === 'error'}
        <div class="bg-red-500/10 border border-red-500/30 text-red-400 rounded-lg p-3 text-sm">{fundError}</div>
      {/if}

      <button onclick={() => { showFundModal = false; fundStatus = 'idle'; }}
        class="w-full mt-4 text-gray-500 hover:text-gray-300 text-sm">
        Close
      </button>
    </div>
  </div>
{/if}
```

- [ ] **Step 4: Add handleFund and polling logic**

```typescript
async function handleFund() {
  fundStatus = 'invoice';
  fundError = '';
  try {
    const result = await api.campaigns.fund(campaign.id, fundAmount);
    fundInvoice = result.invoice;
    fundQuoteId = result.quote_id;
    fundStatus = 'polling';
    pollFunding();
  } catch (e: unknown) {
    fundError = e instanceof Error ? e.message : 'Failed to create invoice';
    fundStatus = 'error';
  }
}

async function pollFunding() {
  const interval = setInterval(async () => {
    try {
      const status = await api.campaigns.checkFunding(campaign.id, fundQuoteId);
      if (status.paid) {
        clearInterval(interval);
        fundStatus = 'done';
        // Refresh campaign data
        campaign = await api.campaigns.get(campaign.id);
      }
    } catch { /* keep polling */ }
  }, 3000);
  // Stop after 10 minutes
  setTimeout(() => clearInterval(interval), 600_000);
}
```

- [ ] **Step 5: Build and verify**

Run: `cd clips-web && npm run build`
Expected: Build succeeds

- [ ] **Step 6: Deploy frontend**

Run: `wrangler pages deploy .svelte-kit/cloudflare --project-name clips-divine-video --commit-dirty=true`

- [ ] **Step 7: Commit**

```bash
git add clips-web/
git commit -m "feat: add campaign funding UI with Lightning invoice modal"
```

---

## Chunk 2: Backend Deployment

The backend needs to be deployed somewhere accessible. The Dockerfile and docker-compose exist but it's not deployed.

### Task 2: Set up backend deployment on Fly.io or Railway

**Files:**
- Create: `clipcrate/fly.toml` (or equivalent deployment config)
- Modify: `clipcrate/Dockerfile` (verify it works)
- Create: `clipcrate/migrations/run.sh` (migration runner script)

- [ ] **Step 1: Verify Docker build works locally**

```bash
cd clipcrate && docker build -t clipcrate .
```

Fix any build issues. The Dockerfile uses `rust:latest` and multi-stage build.

- [ ] **Step 2: Choose deployment target and configure**

Option A — **Fly.io** (recommended for quick deploy):
```bash
cd clipcrate
fly launch --name divine-clips-api --region sjc --no-deploy
```

Option B — **Railway** or **Render** via Docker.

- [ ] **Step 3: Set up managed Postgres**

Create a Postgres database on the deployment platform:
```bash
fly postgres create --name divine-clips-db --region sjc
fly postgres attach divine-clips-db
```

- [ ] **Step 4: Run database migrations**

```bash
# Connect to the database and run migrations
cat migrations/001_initial_schema.sql | fly postgres connect -a divine-clips-db
cat migrations/002_clickhouse_schema.sql  # Skip if no ClickHouse in prod
cat migrations/003_campaign_funding.sql | fly postgres connect -a divine-clips-db
```

- [ ] **Step 5: Set environment variables**

```bash
fly secrets set \
  CDK_MINT_URL="https://testnut.cashu.space" \
  CASHU_WALLET_SEED="$(openssl rand -hex 64)" \
  RUST_LOG="info"
```

- [ ] **Step 6: Deploy**

```bash
fly deploy
```

- [ ] **Step 7: Update frontend API URL**

Update `clips-web/src/lib/api.ts` to point to the deployed backend:
```typescript
const API_BASE = import.meta.env.VITE_API_URL || 'https://divine-clips-api.fly.dev';
```

Also update CORS in `clipcrate/crates/api/src/lib.rs` to allow the deployed domain.

- [ ] **Step 8: Rebuild and redeploy frontend**

```bash
cd clips-web && npm run build
wrangler pages deploy .svelte-kit/cloudflare --project-name clips-divine-video --commit-dirty=true
```

- [ ] **Step 9: Verify end-to-end**

```bash
curl https://divine-clips-api.fly.dev/health
# Expected: "ok"
curl https://divine-clips-api.fly.dev/api/campaigns
# Expected: [] (empty array)
```

- [ ] **Step 10: Commit**

```bash
git add clipcrate/fly.toml clips-web/src/lib/api.ts clipcrate/crates/api/src/lib.rs
git commit -m "feat: deploy backend to Fly.io, wire frontend to production API"
```

---

## Chunk 3: View Verification Worker Deployment

The clips-verifier Cloudflare Worker needs API keys and deployment.

### Task 3: Deploy verification worker

**Files:**
- Modify: `clips-verifier/wrangler.toml` (add env vars)
- Modify: `clips-verifier/src/index.ts` (update API base URL)

- [ ] **Step 1: Update wrangler.toml with API endpoint**

```toml
[vars]
CLIPCRATE_API_URL = "https://divine-clips-api.fly.dev"
```

- [ ] **Step 2: Set secrets for platform APIs**

```bash
cd clips-verifier
wrangler secret put YOUTUBE_API_KEY
# Paste your YouTube Data API key

wrangler secret put CLIPCRATE_AUTH_TOKEN
# Paste an internal auth token for the verifier
```

- [ ] **Step 3: Deploy the worker**

```bash
wrangler deploy
```

- [ ] **Step 4: Test the cron trigger manually**

```bash
curl "https://clips-verifier.<your-account>.workers.dev/__scheduled?cron=*+*/6+*+*+*"
```

- [ ] **Step 5: Commit**

```bash
git add clips-verifier/wrangler.toml
git commit -m "feat: deploy verification worker with YouTube API integration"
```

---

## Chunk 4: End-to-End Integration Test

Test the full flow: create campaign → fund → submit clip → verify → payout.

### Task 4: Manual end-to-end integration test

- [ ] **Step 1: Create a campaign via the UI**

1. Go to https://clips.divine.video/create
2. Sign in with DiVine OAuth
3. Create a campaign: title "Test Campaign", $1 budget, pick a DiVine video, target TikTok
4. Verify it appears on /campaigns

- [ ] **Step 2: Fund the campaign**

1. Open the campaign detail page
2. Click "Fund Campaign"
3. Enter amount (e.g. 1000 sats)
4. Copy the Lightning invoice
5. Pay it (via testnut, the invoice auto-pays on the test mint)
6. Verify "Payment received!" shows
7. Verify budget_remaining increased on campaign detail

- [ ] **Step 3: Submit a clip**

1. Go to /submit
2. Select the test campaign
3. Paste a YouTube video URL (any public video for testing)
4. Submit
5. Verify it appears on /dashboard as pending

- [ ] **Step 4: Verify view count (manual trigger)**

1. Trigger the verification worker
2. Check that the submission status updates
3. Check that a payout was created (if views > 0)

- [ ] **Step 5: Check wallet balance**

1. Go to /wallet
2. Verify balance reflects any payouts earned
3. (Optional) Test withdrawal with a test Lightning invoice

- [ ] **Step 6: Document results**

Write a brief summary of what worked and what didn't. File bugs for any failures.

---

## Chunk 5: CI/CD Pipeline

### Task 5: Add GitHub Actions for CI

**Files:**
- Create: `.github/workflows/ci.yml`

- [ ] **Step 1: Create CI workflow**

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  backend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: clipcrate
      - name: Check
        working-directory: clipcrate
        run: cargo check --all-targets
      - name: Test (unit only)
        working-directory: clipcrate
        run: cargo test --workspace --lib

  frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: npm
          cache-dependency-path: clips-web/package-lock.json
      - name: Install
        working-directory: clips-web
        run: npm ci
      - name: Build
        working-directory: clips-web
        run: npm run build

  verifier:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: npm
          cache-dependency-path: clips-verifier/package-lock.json
      - name: Install
        working-directory: clips-verifier
        run: npm ci
      - name: Test
        working-directory: clips-verifier
        run: npx vitest run
```

- [ ] **Step 2: Commit and push**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: add GitHub Actions for backend, frontend, and verifier"
git push origin main
```

- [ ] **Step 3: Verify CI passes**

Check https://github.com/divinevideo/divine-clips/actions

---

## Chunk 6: Platform API Improvements

TikTok returns 0 views (oEmbed only), Instagram and X are stubs.

### Task 6: Improve platform coverage in clips-verifier

**Files:**
- Modify: `clips-verifier/src/platforms/tiktok.ts`
- Create: `clips-verifier/src/platforms/instagram.ts` (if stub)
- Create: `clips-verifier/src/platforms/x.ts` (if stub)

- [ ] **Step 1: Check current platform implementations**

Read `clips-verifier/src/platforms/tiktok.ts` and assess what's there.
Read `clips-verifier/src/platforms/types.ts` for the interface.

- [ ] **Step 2: Improve TikTok implementation**

TikTok oEmbed doesn't return view counts. Options:
- Use TikTok Research API (requires approval — not MVP)
- Use Phyllo API as fallback (if API key available)
- Accept 0 views for TikTok for MVP and note this limitation

For MVP, update TikTok to at least validate the URL exists and return the metadata it can get.

- [ ] **Step 3: Add Instagram stub**

Create `clips-verifier/src/platforms/instagram.ts` with oEmbed validation (similar to TikTok):
```typescript
export async function getInstagramViews(url: string): Promise<PlatformResult> {
  // Instagram oEmbed doesn't expose view counts
  // Requires Instagram Graph API with business account access
  const oembedUrl = `https://www.instagram.com/api/v1/oembed/?url=${encodeURIComponent(url)}`;
  const res = await fetch(oembedUrl);
  if (!res.ok) return { views: 0, valid: false };
  return { views: 0, valid: true };
}
```

- [ ] **Step 4: Add X/Twitter stub**

Similar pattern — validate URL exists, return 0 views (X API requires paid tier).

- [ ] **Step 5: Run verifier tests**

```bash
cd clips-verifier && npx vitest run
```

- [ ] **Step 6: Commit**

```bash
git add clips-verifier/
git commit -m "feat: add Instagram and X platform stubs, document TikTok view count limitation"
```

---

## Chunk 7: Production Readiness Polish

### Task 7: Error handling, edge cases, and polish

- [ ] **Step 1: Fix home page SSE feed URL**

In `clips-web/src/routes/+page.svelte`, the SSE feed URL is hardcoded to `localhost:3100`. Update:
```typescript
const API_BASE = import.meta.env.VITE_API_URL || 'https://api.clips.divine.video';
```

- [ ] **Step 2: Remove duplicate min-h-screen wrappers**

Both wallet and dashboard pages have `<div class="min-h-screen bg-gray-950 text-white">` wrappers that duplicate the layout. Remove them — the layout already provides this.

- [ ] **Step 3: Add proper error boundaries**

Ensure all pages gracefully handle API failures (show content, not blank pages). The wallet and dashboard fixes from earlier should be applied to all pages that make API calls.

- [ ] **Step 4: Build and deploy final version**

```bash
cd clips-web && npm run build
wrangler pages deploy .svelte-kit/cloudflare --project-name clips-divine-video --commit-dirty=true
```

- [ ] **Step 5: Final commit**

```bash
git add .
git commit -m "fix: production readiness polish - API URLs, error handling, layout cleanup"
git push origin main
```

---

## Execution Order

| Chunk | Task | Priority | Independent? |
|-------|------|----------|-------------|
| 1 | Campaign Funding UI | Critical | Yes |
| 2 | Backend Deployment | Critical | Yes |
| 3 | Verification Worker Deploy | High | Depends on Chunk 2 |
| 4 | E2E Integration Test | High | Depends on Chunks 1-3 |
| 5 | CI/CD Pipeline | Medium | Yes |
| 6 | Platform API Improvements | Medium | Yes |
| 7 | Production Polish | Medium | Yes |

**Chunks 1, 2, 5, 6, 7 can run in parallel.** Chunk 3 needs the backend URL from Chunk 2. Chunk 4 needs everything deployed.
