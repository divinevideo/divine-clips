# clips.divine.video SvelteKit Frontend Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development to implement this plan.

**Goal:** Build the clipper-facing web UI at clips.divine.video — campaign browsing, clip submission, wallet, dashboard, and live feed.

**Architecture:** SvelteKit with Keycast login, REST API calls to clipcrate, WebSocket to funnelcake relay for live events.

**Tech Stack:** SvelteKit, TypeScript, TailwindCSS, nostr-tools, Keycast auth.

**Spec:** `docs/superpowers/specs/2026-04-06-divine-clips-design.md`

---

## File Structure

```
clips-web/
├── package.json
├── svelte.config.js
├── vite.config.ts
├── tailwind.config.js
├── tsconfig.json
├── src/
│   ├── app.html
│   ├── app.css                     # Tailwind imports
│   ├── lib/
│   │   ├── api.ts                  # clipcrate REST client
│   │   ├── auth.ts                 # Keycast auth helpers
│   │   ├── nostr.ts                # Nostr relay subscription helpers
│   │   ├── stores/
│   │   │   ├── auth.ts             # Auth state store
│   │   │   ├── campaigns.ts        # Campaign list store
│   │   │   └── feed.ts             # Live feed store
│   │   ├── types.ts                # Shared TypeScript types
│   │   └── utils.ts                # Formatting helpers (sats, dates)
│   └── routes/
│       ├── +layout.svelte          # App shell: nav, auth state
│       ├── +layout.ts              # Load auth state
│       ├── +page.svelte            # / — Live activity feed
│       ├── campaigns/
│       │   ├── +page.svelte        # /campaigns — Browse campaigns
│       │   ├── +page.ts            # Load campaigns
│       │   └── [id]/
│       │       ├── +page.svelte    # /campaigns/[id] — Campaign detail
│       │       └── +page.ts        # Load campaign by ID
│       ├── submit/
│       │   └── +page.svelte        # /submit — Submit clip link
│       ├── dashboard/
│       │   └── +page.svelte        # /dashboard — Clipper stats
│       ├── wallet/
│       │   └── +page.svelte        # /wallet — Balance + withdraw
│       ├── profile/
│       │   └── +page.svelte        # /profile — Connected accounts
│       └── create/
│           └── +page.svelte        # /create — Campaign creation
└── static/
    └── favicon.png
```

---

## Task 1: SvelteKit project scaffold

**Files:** All config files + app.html + app.css + types.ts + utils.ts

Create SvelteKit project with TailwindCSS. Install dependencies:
- `@sveltejs/kit`, `svelte`, `vite`
- `tailwindcss`, `@tailwindcss/vite`
- `nostr-tools` (for relay subscriptions and event parsing)

Create `src/lib/types.ts` with TypeScript interfaces matching clipcrate API:
```typescript
export interface Campaign {
  id: string; creator_pubkey: string; title: string;
  budget_total_sats: number; budget_remaining_sats: number;
  cpm_sats: number; status: string; target_platforms: string[];
  content_refs: string[]; guidelines: string | null;
  expires_at: string | null; created_at: string;
}
export interface Submission {
  id: string; campaign_id: string; clipper_pubkey: string;
  external_url: string; platform: string; status: string;
  total_verified_views: number; total_paid_sats: number;
  submitted_at: string; last_verified_at: string | null;
}
export interface DashboardData {
  trust_level: number; total_verified_views: number;
  total_earned_sats: number; balance_sats: number;
  active_submissions: number; weekly_views_used: number;
  weekly_views_limit: number;
}
export interface TransactionRecord {
  amount_sats: number; transaction_type: string;
  created_at: string;
}
```

Create `src/lib/utils.ts`:
```typescript
export function formatSats(sats: number): string {
  return sats.toLocaleString() + ' sats';
}
export function formatViews(views: number): string {
  if (views >= 1_000_000) return (views / 1_000_000).toFixed(1) + 'M';
  if (views >= 1_000) return (views / 1_000).toFixed(1) + 'K';
  return views.toString();
}
export function timeAgo(date: string): string {
  const seconds = Math.floor((Date.now() - new Date(date).getTime()) / 1000);
  if (seconds < 60) return 'just now';
  if (seconds < 3600) return Math.floor(seconds / 60) + 'm ago';
  if (seconds < 86400) return Math.floor(seconds / 3600) + 'h ago';
  return Math.floor(seconds / 86400) + 'd ago';
}
export const PLATFORMS = ['tiktok', 'instagram', 'youtube', 'x'] as const;
```

Verify: `npm run dev` starts without errors.

---

## Task 2: API client + Auth

**Files:** `src/lib/api.ts`, `src/lib/auth.ts`, `src/lib/stores/auth.ts`

Create `src/lib/api.ts` — REST client for clipcrate:
```typescript
const API_BASE = import.meta.env.VITE_API_URL || 'http://localhost:3100';

async function apiFetch<T>(path: string, opts: RequestInit = {}): Promise<T> {
  const token = getAuthToken();
  const headers: Record<string, string> = { 'Content-Type': 'application/json' };
  if (token) headers['Authorization'] = `Bearer ${token}`;
  const res = await fetch(`${API_BASE}${path}`, { ...opts, headers: { ...headers, ...opts.headers } });
  if (!res.ok) throw new Error(`API error: ${res.status}`);
  return res.json();
}

export const api = {
  campaigns: {
    list: (limit = 20, offset = 0) => apiFetch<Campaign[]>(`/api/campaigns?limit=${limit}&offset=${offset}`),
    get: (id: string) => apiFetch<Campaign>(`/api/campaigns/${id}`),
    create: (data: CreateCampaignRequest) => apiFetch<Campaign>('/api/campaigns', { method: 'POST', body: JSON.stringify(data) }),
    update: (id: string, data: UpdateCampaignRequest) => apiFetch<Campaign>(`/api/campaigns/${id}`, { method: 'PATCH', body: JSON.stringify(data) }),
  },
  submissions: {
    list: (limit = 20, offset = 0) => apiFetch<Submission[]>(`/api/submissions?limit=${limit}&offset=${offset}`),
    get: (id: string) => apiFetch<Submission>(`/api/submissions/${id}`),
    create: (data: CreateSubmissionRequest) => apiFetch<Submission>('/api/submissions', { method: 'POST', body: JSON.stringify(data) }),
  },
  dashboard: () => apiFetch<DashboardData>('/api/dashboard'),
  wallet: {
    balance: () => apiFetch<{ balance_sats: number }>('/api/wallet/balance'),
    withdraw: (invoice: string, amount_sats: number) => apiFetch<{ status: string }>('/api/wallet/withdraw', { method: 'POST', body: JSON.stringify({ invoice, amount_sats }) }),
    history: () => apiFetch<{ transactions: TransactionRecord[] }>('/api/wallet/history'),
  },
};
```

Create `src/lib/stores/auth.ts` — Svelte writable store for auth state.
Create `src/lib/auth.ts` — Keycast login stub (MVP: prompt for npub, store as token).

---

## Task 3: Layout + Navigation

**Files:** `src/routes/+layout.svelte`, `src/routes/+layout.ts`

App shell with:
- Top nav: "DiVine Clips" logo/text, links (Campaigns, Dashboard, Wallet), auth button
- Auth button: "Sign in with DiVine" → shows login modal → stores token
- Responsive: mobile-friendly nav
- TailwindCSS styling: dark theme matching DiVine brand (blacks, purples)

---

## Task 4: Campaign browsing pages

**Files:** `src/routes/campaigns/+page.svelte`, `src/routes/campaigns/+page.ts`, `src/routes/campaigns/[id]/+page.svelte`, `src/routes/campaigns/[id]/+page.ts`

**/campaigns** — Grid of campaign cards showing:
- Title, CPM rate, budget remaining (progress bar), target platforms (icons), time remaining
- Click → campaign detail page

**/campaigns/[id]** — Campaign detail:
- Full details, content preview (Blossom video URLs), guidelines
- "Claim & Clip" button (requires auth) → links to /submit with campaign pre-selected
- Download button for source video

---

## Task 5: Submit clip page

**Files:** `src/routes/submit/+page.svelte`

Form:
- Campaign selector (dropdown or pre-filled from campaign detail)
- Platform selector (TikTok, Instagram, YouTube, X)
- External URL input (validated: must match platform domain)
- Submit button → POST to /api/submissions
- Success/error feedback

---

## Task 6: Dashboard page

**Files:** `src/routes/dashboard/+page.svelte`

Shows:
- Trust level badge (Level 1/2/3)
- Stats cards: total views, total earned, balance, active submissions
- Weekly views progress bar (used / limit)
- List of active submissions with status, views, earned per submission

---

## Task 7: Wallet page

**Files:** `src/routes/wallet/+page.svelte`

Shows:
- Balance prominently
- Withdraw form: Lightning invoice input + amount + submit
- Transaction history list (payouts + withdrawals with timestamps)

---

## Task 8: Live feed + home page

**Files:** `src/routes/+page.svelte`, `src/lib/nostr.ts`, `src/lib/stores/feed.ts`

Home page shows real-time activity:
- SSE connection to `/api/feed/live`
- New campaigns appear with animation
- "New campaign: [title] — [cpm] sats/1K views" cards
- Link to browse all campaigns

`src/lib/nostr.ts` — helper for future WebSocket relay subscriptions (stub for MVP, SSE is used initially).

---

## Task 9: Campaign creation page

**Files:** `src/routes/create/+page.svelte`

Creator flow:
- Title input
- Budget (sats) input
- CPM rate input
- Target platforms multi-select
- Content refs (DiVine video URLs/IDs)
- Guidelines textarea (optional)
- Expiry date picker (optional)
- Submit → POST to /api/campaigns

---

## Task 10: Profile page + final polish

**Files:** `src/routes/profile/+page.svelte`

Shows:
- Connected Nostr identity (npub display)
- Placeholder for Phyllo social account linking
- Stats summary

Final polish:
- Loading states on all pages
- Error boundaries
- Mobile responsive check
- `npm run build` must succeed
