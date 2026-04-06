use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use cdk::amount::SplitTarget;
use cdk::nuts::nut00::KnownMethod;
use cdk::nuts::{CurrencyUnit, MintQuoteState, PaymentMethod};
use cdk::wallet::Wallet;
use cdk::Amount;
use cdk_sqlite::WalletSqliteDatabase;

/// Real Cashu wallet backed by CDK, connecting to a configurable mint.
///
/// The server holds a single custodial wallet. Campaign deposits mint ecash,
/// clipper withdrawals melt ecash to pay Lightning invoices.
#[derive(Clone)]
pub struct CashuWallet {
    wallet: Arc<Wallet>,
}

impl CashuWallet {
    /// Create a new wallet connected to the given mint URL.
    pub async fn new(mint_url: &str, seed: [u8; 64], db_path: &str) -> Result<Self> {
        let localstore = WalletSqliteDatabase::new(db_path)
            .await
            .context("Failed to open wallet SQLite database")?;

        let wallet = Wallet::new(
            mint_url,
            CurrencyUnit::Sat,
            Arc::new(localstore),
            seed,
            None,
        )
        .context("Failed to create CDK wallet")?;

        // Recover any incomplete operations from previous runs
        if let Err(e) = wallet.recover_incomplete_sagas().await {
            tracing::warn!("Wallet recovery had issues (non-fatal): {e}");
        }

        tracing::info!(mint_url, "Cashu wallet initialized");

        Ok(Self {
            wallet: Arc::new(wallet),
        })
    }

    /// Create a Lightning invoice for funding a campaign.
    /// Returns (bolt11_invoice, quote_id).
    pub async fn create_funding_invoice(&self, amount_sats: u64) -> Result<(String, String)> {
        let quote = self
            .wallet
            .mint_quote(
                PaymentMethod::Known(KnownMethod::Bolt11),
                Some(Amount::from(amount_sats)),
                None,
                None,
            )
            .await
            .context("Failed to create mint quote")?;

        let invoice = quote.request.clone();
        let quote_id = quote.id.clone();

        tracing::info!(
            amount_sats,
            quote_id = %quote_id,
            "Created funding invoice"
        );

        Ok((invoice, quote_id))
    }

    /// Check whether a funding invoice has been paid.
    pub async fn check_funding_paid(&self, quote_id: &str) -> Result<bool> {
        let quote = self
            .wallet
            .check_mint_quote_status(quote_id)
            .await
            .context("Failed to check mint quote status")?;

        Ok(quote.state == MintQuoteState::Paid)
    }

    /// Complete funding by minting ecash tokens after the invoice is paid.
    /// Returns the amount minted in sats.
    pub async fn complete_funding(&self, quote_id: &str) -> Result<u64> {
        let _proofs = self
            .wallet
            .mint(quote_id, SplitTarget::default(), None)
            .await
            .context("Failed to mint tokens after payment")?;

        // Use wallet balance to determine the amount (proofs are stored internally)
        let balance = self.total_balance().await?;

        tracing::info!(
            quote_id,
            balance_sats = balance,
            "Minted ecash tokens for campaign funding"
        );

        Ok(balance)
    }

    /// Pay a Lightning invoice by melting ecash tokens.
    /// Returns the payment preimage on success.
    pub async fn withdraw_to_invoice(&self, invoice: &str) -> Result<String> {
        let melt_quote = self
            .wallet
            .melt_quote(
                PaymentMethod::Known(KnownMethod::Bolt11),
                invoice,
                None,
                None,
            )
            .await
            .context("Failed to create melt quote")?;

        tracing::info!(
            quote_id = %melt_quote.id,
            amount = %melt_quote.amount,
            fee_reserve = %melt_quote.fee_reserve,
            "Created melt quote for withdrawal"
        );

        let prepared = self
            .wallet
            .prepare_melt(&melt_quote.id, HashMap::new())
            .await
            .context("Failed to prepare melt")?;

        let finalized = prepared
            .confirm()
            .await
            .context("Failed to confirm melt (pay Lightning invoice)")?;

        let preimage = finalized
            .payment_proof()
            .unwrap_or("")
            .to_string();

        tracing::info!(
            amount = %finalized.amount(),
            fee_paid = %finalized.fee_paid(),
            "Withdrawal complete"
        );

        Ok(preimage)
    }

    /// Get total ecash balance held by the wallet.
    pub async fn total_balance(&self) -> Result<u64> {
        let balance = self
            .wallet
            .total_balance()
            .await
            .context("Failed to get wallet balance")?;

        Ok(u64::from(balance))
    }
}

/// Create a CashuWallet from environment variables.
///
/// Reads:
/// - `CASHU_MINT_URL` (default: "https://testnut.cashu.space")
/// - `CASHU_WALLET_DB_PATH` (default: "./data/cashu-wallet.db")
/// - `CASHU_WALLET_SEED` (hex-encoded 64-byte seed; generates random if not set)
pub async fn from_env() -> Result<CashuWallet> {
    let mint_url = std::env::var("CASHU_MINT_URL")
        .unwrap_or_else(|_| "https://testnut.cashu.space".to_string());

    let db_path = std::env::var("CASHU_WALLET_DB_PATH")
        .unwrap_or_else(|_| "./data/cashu-wallet.db".to_string());

    let seed: [u8; 64] = match std::env::var("CASHU_WALLET_SEED") {
        Ok(hex_seed) => {
            let bytes = hex::decode(&hex_seed)
                .context("CASHU_WALLET_SEED must be valid hex")?;
            let mut seed = [0u8; 64];
            if bytes.len() != 64 {
                anyhow::bail!("CASHU_WALLET_SEED must be exactly 64 bytes (128 hex chars)");
            }
            seed.copy_from_slice(&bytes);
            seed
        }
        Err(_) => {
            tracing::warn!("No CASHU_WALLET_SEED set — generating random seed (NOT suitable for production)");
            let mut seed = [0u8; 64];
            getrandom::fill(&mut seed).context("Failed to generate random seed")?;
            seed
        }
    };

    // Ensure the db directory exists
    if let Some(parent) = std::path::Path::new(&db_path).parent() {
        std::fs::create_dir_all(parent)
            .context("Failed to create wallet database directory")?;
    }

    CashuWallet::new(&mint_url, seed, &db_path).await
}
