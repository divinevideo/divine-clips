# Divine Clips

Divine Clips is a clipping marketplace for [Divine](https://divine.video), the decentralized short-form video app reviving Vine's 6-second format on Nostr. Creators fund campaigns to promote their loops; clippers repost that content to TikTok, Instagram Reels, YouTube Shorts, and X, then submit the links back. View counts are verified through platform APIs, and payouts settle as Bitcoin Lightning zaps based on verified views. Campaigns, submissions, and payouts are published as Nostr events, so a clipper's identity and reputation stay portable across the network.

## Features

- **Campaign marketplace** — creators fund a sats budget and set a per-view payout rate; clippers browse a live feed of active campaigns.
- **Submission flow** — clippers repost a loop to an external platform and submit the URL back, tagged with the platform.
- **View verification** — a scheduled worker polls each submission's view count through platform APIs, with a Phyllo fallback for platforms without direct access.
- **Lightning payouts** — verified views trigger automated payouts; campaign budgets are held as Cashu ecash backed by a Lightning mint.
- **Fraud signals** — velocity spikes, young accounts, and other suspicious patterns are flagged before payout.
- **Analytics & leaderboards** — per-submission view charts, an overview dashboard, a clipper leaderboard, and social-proof stats.
- **Web push notifications** — clippers can subscribe to updates on their submissions.
- **Auto-campaigns** — new Divine loops arriving over a Nostr relay subscription can spawn campaigns automatically.

## Repository layout

This is a monorepo with three independently built and deployed subprojects, plus local orchestration at the root.

- **`clipcrate/`** — the Rust backend. A Cargo workspace exposing an Axum HTTP API, backed by Postgres (relational state) and ClickHouse (analytics). Its members are:
  - `clipcrate` — the binary; wires up the database, runs migrations, starts the auto-campaign listener, and serves the API.
  - `crates/api` — Axum router and handlers for campaigns, funding, submissions, wallet, dashboard, leaderboard, analytics, notifications, and internal verifier endpoints.
  - `crates/db` — Postgres and ClickHouse clients and models.
  - `crates/nostr_events` — builders and a relay publisher for campaign (kind 30402), submission (kind 30403), and payout (kind 9734) events.
  - `crates/cashu` — a Cashu ecash wallet for holding campaign budgets and paying out.
  - `crates/trust` — fraud detection (velocity, account age, suspicious patterns).
  - `crates/platform-apis` — platform clients (Instagram, TikTok, Twitter/X, YouTube).
- **`clips-web/`** — the web frontend. A SvelteKit app (Svelte 5, Tailwind CSS 4) for creators and clippers, with sign-in via `@divinevideo/login`, campaign and submission pages, a wallet, and LayerCake analytics charts. Builds for Cloudflare via `@sveltejs/adapter-cloudflare`.
- **`clips-verifier/`** — the verification service. A Cloudflare Worker that runs on a cron schedule (every 6 hours), pulls pending submissions from the backend, resolves view counts per platform, scores them for fraud, and posts results back.

## Architecture

The three services form a loop around the campaign lifecycle:

1. A creator **funds a campaign** in `clips-web`. The backend issues a Lightning invoice through the Cashu mint (`crates/cashu`); once paid, the budget is escrowed as ecash.
2. The campaign is stored in Postgres and published as a Nostr event (kind 30402) by `crates/nostr_events`.
3. A clipper **submits a repost URL** through `clips-web`. The submission is stored and published as a Nostr event (kind 30403).
4. `clips-verifier` wakes on its cron trigger, calls the backend's internal endpoint for pending submissions, and **resolves view counts** — YouTube via the Data API, TikTok / Instagram / X via their clients, and Phyllo as a fallback. It runs a fraud score and posts verification results back to the internal API.
5. When verified views cross a threshold, the backend **triggers a payout** from the Cashu wallet as a Lightning zap and records it as a Nostr event (kind 9734).

Analytics roll up into ClickHouse to power the dashboard, leaderboard, and per-submission charts. Because campaigns, submissions, and payouts are Nostr events keyed to each participant's npub, clipper reputation lives on the network rather than inside the marketplace.

**Platform fit.** Divine Clips is grassroots distribution for Divine loops. The 6-second looping format is distinctive on its own, so clips need no watermark or overlay — reposting the content is enough to drive curiosity back to Divine. Payments and identity reuse the same Nostr and Lightning infrastructure the rest of Divine is built on.

## Getting started

Clone the repo, then work within each subproject. To bring up the backing services (Postgres, ClickHouse, the clipcrate API, a Lightning node, and a Cashu mint) together:

```sh
docker compose up
```

### Backend (`clipcrate/`)

Requires a Rust toolchain, plus Postgres and ClickHouse (the `docker compose` stack provides both). Migrations run automatically on startup.

```sh
cd clipcrate
cargo check --all-targets
cargo test --workspace --lib
cargo run -p clipcrate      # serves on PORT (default 8080)
```

### Frontend (`clips-web/`)

```sh
cd clips-web
npm ci
npm run dev                 # local dev server on http://localhost:5173
npm run build               # production build
npm run check               # svelte-check / type checking
```

### Verifier (`clips-verifier/`)

```sh
cd clips-verifier
npm ci
npx vitest run              # run tests
npm run dev                 # run the worker locally via wrangler
```

## Configuration

Services are configured through environment variables.

**Backend (`clipcrate`)**

- `DATABASE_URL` — Postgres connection string (default `postgres://localhost:5432/clipcrate`).
- `CLICKHOUSE_URL` — ClickHouse HTTP endpoint (default `http://localhost:8123`).
- `PORT` — HTTP port to listen on (default `8080`).
- `CDK_MINT_URL` — Cashu mint endpoint used by the wallet.
- `RUST_LOG` — tracing filter.

**Frontend (`clips-web`)**

- `VITE_API_URL` — backend base URL (production: `https://api.clips.divine.video`).

**Verifier (`clips-verifier`)**

- `CLIPCRATE_API_URL` — backend base URL (set in `wrangler.toml`, production: `https://api.clips.divine.video`).
- `CLIPCRATE_API_TOKEN` — token for the backend's internal endpoints.
- `YOUTUBE_API_KEY` — YouTube Data API key for view counts.

## Deployment

- **Backend** builds from `clipcrate/Dockerfile` and runs on Cloud Run, served at `api.clips.divine.video`.
- **Frontend** builds with the Cloudflare adapter and is served at `clips.divine.video`.
- **Verifier** deploys as a Cloudflare Worker (`wrangler deploy`) and runs on its cron schedule.

CI (`.github/workflows/ci.yml`) checks all three on every pull request: `cargo check` and `cargo test` for the backend, `npm run build` for the frontend, and `vitest` for the verifier. A semantic PR title check enforces Conventional Commit titles.

See [AGENTS.md](AGENTS.md) for build, test, and contribution guidelines.

---

Part of [Divine](https://divine.video) — your playground for human creativity · [Brand guidelines](https://github.com/divinevideo/brand-guidelines)
