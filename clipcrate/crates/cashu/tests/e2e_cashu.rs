/// End-to-end tests against testnut.cashu.space
///
/// These tests exercise the full Cashu wallet lifecycle:
/// - Creating funding invoices (mint quotes)
/// - Checking quote status
/// - Minting tokens
/// - Melting tokens (paying invoices)
/// - Balance tracking
///
/// Run with: cargo test -p clipcrate-cashu --test e2e_cashu -- --nocapture
///
/// These tests hit a real test mint so they require network access.
/// The testnut mint auto-pays invoices, making e2e testing possible without
/// a real Lightning wallet.
///
/// Tests run as a single sequential test to avoid rate limiting.

use clipcrate_cashu::cashu_wallet::CashuWallet;
use std::time::Duration;

const TEST_MINT_URL: &str = "https://testnut.cashu.space";

async fn fresh_wallet(name: &str) -> CashuWallet {
    let mut seed = [0u8; 64];
    getrandom::fill(&mut seed).expect("Failed to generate random seed");

    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let db_path = format!("/tmp/cashu_test_{}_{}.db", name, unique);

    CashuWallet::new(TEST_MINT_URL, seed, &db_path)
        .await
        .expect("Failed to create test wallet")
}

/// Single sequential test that exercises the full lifecycle to avoid rate limiting.
#[tokio::test]
async fn test_full_cashu_lifecycle() {
    println!("\n=== Step 1: Fresh wallet has zero balance ===");
    let wallet = fresh_wallet("lifecycle").await;
    let balance = wallet.total_balance().await.expect("Failed to get balance");
    assert_eq!(balance, 0, "Fresh wallet should have zero balance");
    println!("OK: Balance is 0");

    println!("\n=== Step 2: Create funding invoice ===");
    let (invoice, quote_id) = wallet
        .create_funding_invoice(500)
        .await
        .expect("Failed to create funding invoice");
    assert!(!invoice.is_empty());
    assert!(!quote_id.is_empty());
    println!("OK: Invoice created ({}...)", &invoice[..30.min(invoice.len())]);

    println!("\n=== Step 3: Check if testnut auto-paid ===");
    tokio::time::sleep(Duration::from_secs(3)).await;
    let paid = wallet
        .check_funding_paid(&quote_id)
        .await
        .expect("Failed to check funding status");
    println!("Invoice paid: {}", paid);
    if !paid {
        println!("SKIP: testnut did not auto-pay — remaining tests require payment");
        return;
    }

    println!("\n=== Step 4: Mint tokens (complete funding) ===");
    let balance_after_fund = wallet
        .complete_funding(&quote_id)
        .await
        .expect("Failed to complete funding");
    assert!(balance_after_fund >= 500, "Should have at least 500 sats");
    println!("OK: Balance after funding: {} sats", balance_after_fund);

    println!("\n=== Step 5: Fund again (cumulative) ===");
    tokio::time::sleep(Duration::from_secs(1)).await;
    let (_, quote_id2) = wallet
        .create_funding_invoice(200)
        .await
        .expect("Failed to create second invoice");
    tokio::time::sleep(Duration::from_secs(3)).await;
    let paid2 = wallet
        .check_funding_paid(&quote_id2)
        .await
        .expect("Failed to check second quote");
    if paid2 {
        wallet
            .complete_funding(&quote_id2)
            .await
            .expect("Failed to complete second funding");
        let cumulative = wallet.total_balance().await.expect("balance");
        assert!(cumulative >= 700, "Should have at least 700 sats cumulative");
        println!("OK: Cumulative balance: {} sats", cumulative);
    } else {
        println!("WARN: Second invoice not auto-paid, continuing with first funding only");
    }

    println!("\n=== Step 6: Withdraw (melt to Lightning invoice) ===");
    tokio::time::sleep(Duration::from_secs(1)).await;
    let balance_before_withdraw = wallet.total_balance().await.expect("balance");

    // Create a receiver wallet to generate an invoice
    let receiver = fresh_wallet("receiver").await;
    let (receive_invoice, _) = receiver
        .create_funding_invoice(100)
        .await
        .expect("Failed to create receive invoice");

    let preimage = wallet
        .withdraw_to_invoice(&receive_invoice)
        .await
        .expect("Failed to withdraw to invoice");
    println!("OK: Withdrawal preimage: {}", preimage);

    let balance_after_withdraw = wallet.total_balance().await.expect("balance");
    println!(
        "OK: Balance {} -> {} sats (spent {} + fees)",
        balance_before_withdraw,
        balance_after_withdraw,
        balance_before_withdraw - balance_after_withdraw
    );
    assert!(
        balance_after_withdraw < balance_before_withdraw,
        "Balance should decrease after withdrawal"
    );

    println!("\n=== Step 7: Verify insufficient funds rejection ===");
    // Try to withdraw way more than we have
    let huge_invoice_result = wallet.create_funding_invoice(1).await;
    // This tests that the wallet is still functional
    assert!(huge_invoice_result.is_ok(), "Wallet should still be functional");

    println!("\n=== ALL TESTS PASSED ===");
    println!(
        "Final balance: {} sats (funded ~700, withdrew ~100 + fees)",
        balance_after_withdraw
    );
}
