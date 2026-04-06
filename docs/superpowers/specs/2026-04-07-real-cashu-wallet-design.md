# Real Cashu Wallet Integration

**Date:** 2026-04-07
**Status:** Approved

## Overview

Replace the stub `CashuMint` with a real CDK-based Cashu wallet that connects to a configurable mint (defaulting to `https://testnut.cashu.space` for beta). The server holds a single custodial wallet for all campaign funds and clipper payouts.

## Architecture

- **Single server wallet** connected to the configured mint via CDK (`cdk` crate v0.16 with `wallet` feature)
- **Wallet state** (ecash proofs) stored in SQLite via `cdk-sqlite`
- **Postgres** continues to track per-user balances, campaigns, and transactions
- Cashu wallet = source of truth for *total funds held*; Postgres = source of truth for *who owns what*

### Fund Flow

1. Campaign creator funds a campaign → Server creates a mint quote (Lightning invoice) → Creator pays → Mint issues ecash proofs → Campaign budget credited in Postgres
2. Clipper earns from verified views → Earnings tracked in Postgres
3. Clipper withdraws → Server creates a melt quote → Pays clipper's Lightning invoice with ecash → Withdrawal recorded in Postgres

## API Changes

### New Endpoints

- `POST /api/campaigns/:id/fund` — Returns Lightning invoice + quote ID
- `GET /api/campaigns/:id/fund/:quote_id` — Poll whether invoice has been paid

### Modified Endpoints

- `POST /api/wallet/withdraw` — Now actually melts ecash to pay the Lightning invoice, records as "completed" or "failed"

### Unchanged

- `GET /api/wallet/balance` — Reads from Postgres
- `GET /api/wallet/history` — Reads from Postgres
- `GET /api/dashboard` — Reads from Postgres

## Cashu Crate Rewrite

Replace `CashuMint` stub with `CashuWallet`:

```rust
pub struct CashuWallet {
    wallet: cdk::wallet::Wallet,
}

impl CashuWallet {
    pub async fn new(mint_url: &str, seed: &[u8], db_path: &str) -> Result<Self>;
    pub async fn create_funding_invoice(&self, amount_sats: u64) -> Result<(String, String)>;
    pub async fn check_funding_paid(&self, quote_id: &str) -> Result<bool>;
    pub async fn complete_funding(&self, quote_id: &str) -> Result<u64>;
    pub async fn withdraw_to_invoice(&self, invoice: &str) -> Result<String>;
    pub async fn total_balance(&self) -> Result<u64>;
}
```

### Dependencies

```toml
cdk = { version = "0.16", features = ["wallet"] }
cdk-sqlite = "0.16"
```

## Configuration

- `CASHU_MINT_URL` env var (default: `https://testnut.cashu.space`)
- `CASHU_WALLET_DB_PATH` env var (default: `./data/cashu-wallet.db`)
- `CASHU_WALLET_SEED` env var (hex-encoded 64-byte seed, required in production)

## End-to-End Tests

Tests run against testnut.cashu.space:

1. **Fund campaign** — Create campaign → get invoice → verify budget credited after payment
2. **Withdraw** — Fund → earn payout → request withdrawal → verify melt succeeds
3. **Balance consistency** — Fund, earn, withdraw → verify Postgres balance matches
4. **Insufficient funds** — Try to withdraw more than earned → verify rejection

## Risks

- CDK is alpha (v0.16) — API may change
- testnut.cashu.space availability for tests
- Custodial model means server holds all funds
