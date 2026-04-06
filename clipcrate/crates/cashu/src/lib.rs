// clipcrate-cashu: Cashu mint integration for payments
// MVP balance-only mode: tracks balances in Postgres without a real Cashu mint.
// Real mint integration can be layered in without changing the public API.

pub mod mint;
pub mod wallet;
