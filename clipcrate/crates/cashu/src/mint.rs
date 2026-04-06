use anyhow::Result;
use uuid::Uuid;

/// MVP Cashu mint abstraction.
///
/// All operations run in balance-only mode — no real Lightning or Cashu mint
/// is contacted. Payouts and withdrawals are tracked entirely in Postgres.
/// Swap out the inner methods for real mint calls once an actual mint is available.
pub struct CashuMint;

impl CashuMint {
    pub fn new() -> Self {
        Self
    }

    /// Generate a Lightning invoice so a campaign sponsor can deposit sats.
    ///
    /// MVP: returns a placeholder invoice string and payment hash. No real
    /// Lightning node is contacted.
    pub async fn create_deposit_invoice(&self, amount_sats: u64) -> Result<(String, String)> {
        let payment_hash = format!("mvp_hash_{}", Uuid::new_v4().simple());
        let invoice = format!("lnbcrt{}n1mvp_placeholder_{}", amount_sats, payment_hash);
        tracing::info!(
            amount_sats,
            payment_hash = %payment_hash,
            "MVP: created placeholder deposit invoice"
        );
        Ok((invoice, payment_hash))
    }

    /// Mark a deposit as confirmed and credit the campaign budget.
    ///
    /// MVP: no-op — the actual budget credit happens in the campaign service
    /// when a deposit is confirmed out-of-band (e.g. via admin API or test helper).
    pub async fn confirm_deposit(&self, payment_hash: &str, campaign_id: Uuid) -> Result<()> {
        tracing::warn!(
            payment_hash,
            %campaign_id,
            "MVP: confirm_deposit is a no-op; update campaign budget manually"
        );
        Ok(())
    }

    /// Issue an ecash payout token to a clipper.
    ///
    /// MVP: generates a token ID (UUID) and returns it. The caller is responsible
    /// for persisting a row in the `payouts` table with this token ID.
    pub async fn issue_payout(
        &self,
        campaign_id: Uuid,
        clipper_pubkey: &str,
        amount_sats: u64,
    ) -> Result<String> {
        let token_id = format!("mvp_token_{}", Uuid::new_v4().simple());
        tracing::info!(
            %campaign_id,
            clipper_pubkey,
            amount_sats,
            token_id = %token_id,
            "MVP: issued placeholder payout token"
        );
        Ok(token_id)
    }

    /// Melt ecash tokens by paying a Lightning invoice (withdrawal).
    ///
    /// MVP: returns a placeholder preimage. No real payment is sent.
    /// The caller should update the `withdrawals` table row to `completed`
    /// with this preimage once confirmed out-of-band.
    pub async fn withdraw(
        &self,
        clipper_pubkey: &str,
        invoice: &str,
        amount_sats: u64,
    ) -> Result<String> {
        let preimage = format!("mvp_preimage_{}", Uuid::new_v4().simple());
        tracing::warn!(
            clipper_pubkey,
            invoice,
            amount_sats,
            preimage = %preimage,
            "MVP: withdraw is a no-op; no real Lightning payment sent"
        );
        Ok(preimage)
    }
}

impl Default for CashuMint {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_deposit_invoice_does_not_panic() {
        let mint = CashuMint::new();
        let result = mint.create_deposit_invoice(1000).await;
        assert!(result.is_ok());
        let (invoice, hash) = result.unwrap();
        assert!(!invoice.is_empty());
        assert!(!hash.is_empty());
    }

    #[tokio::test]
    async fn test_confirm_deposit_does_not_panic() {
        let mint = CashuMint::new();
        let result = mint.confirm_deposit("some_hash", Uuid::new_v4()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_issue_payout_returns_token_id() {
        let mint = CashuMint::new();
        let result = mint
            .issue_payout(Uuid::new_v4(), "npub1test", 500)
            .await;
        assert!(result.is_ok());
        let token_id = result.unwrap();
        assert!(token_id.starts_with("mvp_token_"));
    }

    #[tokio::test]
    async fn test_withdraw_does_not_panic() {
        let mint = CashuMint::new();
        let result = mint
            .withdraw("npub1test", "lnbcrt1placeholder", 500)
            .await;
        assert!(result.is_ok());
        let preimage = result.unwrap();
        assert!(preimage.starts_with("mvp_preimage_"));
    }
}
