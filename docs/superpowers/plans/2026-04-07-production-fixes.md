# Production Fixes Plan

> **For agentic workers:** Execute in parallel where possible.

**Goal:** Fix all 11 gaps preventing DiVine Clips from working end-to-end in production.

---

## Task A: Fix auto-migrations in main.rs + hostname consistency

1. Add `sqlx::migrate!("../migrations").run(&db).await?;` to main.rs after pool creation
2. Fix clips-verifier wrangler.toml: change CLIPCRATE_API_URL to `https://api.clips.divine.video`
3. Fix clips-web inconsistent API_BASE: make ALL defaults `https://api.clips.divine.video`
4. Create `.env.production` in clips-web with `VITE_API_URL=https://api.clips.divine.video`
5. Add sqlx migrate feature to clipcrate binary Cargo.toml

## Task B: Generate VAPID keys + fix notifications

1. Generate VAPID keypair using web-push tooling
2. Store as GCP secrets: `clipcrate-vapid-private-key-production`, `clipcrate-vapid-public-key-production`
3. Update notifications.rs to read from env vars (not placeholder)
4. Add VAPID env vars to k8s deployment

## Task C: Rebuild Docker + push + redeploy

1. Rebuild Docker image with all fixes
2. Push to Artifact Registry with new digest
3. Update kustomization.yaml with new digest
4. Push to divine-iac-coreconfig
5. Sync via GitHub Actions

## Task D: Run Postgres migrations

1. Connect to Cloud SQL and run migrations 001-005
2. Verify tables exist

## Task E: Configure CF Worker secrets

1. Set real YOUTUBE_API_KEY (or get one)
2. Update CLIPCRATE_API_URL to correct hostname
3. Redeploy worker

## Task F: Redeploy frontend with correct env

1. Build clips-web with VITE_API_URL set
2. Deploy to CF Pages
