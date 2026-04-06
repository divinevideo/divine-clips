// clipcrate-nostr: Nostr event builders and relay publisher
// Provides helpers to construct and sign Nostr events for DiVine Clips
// campaigns (kind 30402), submissions (kind 30403), and payouts (kind 9734),
// and publish them to configured relays via nostr-sdk.

pub mod campaign;
pub mod payout;
pub mod publisher;
pub mod submission;

pub use campaign::build_campaign_event;
pub use payout::build_payout_event;
pub use publisher::Publisher;
pub use submission::build_submission_event;
