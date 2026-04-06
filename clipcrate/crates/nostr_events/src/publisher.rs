// publisher.rs — signs EventBuilders and publishes to a Nostr relay

use anyhow::{Context, Result};
use nostr::prelude::*;
use nostr_sdk::Client;
use tracing::{debug, info};

/// Signs and publishes Nostr events to a configured relay.
pub struct Publisher {
    client: Client,
    relay_url: String,
}

impl Publisher {
    /// Create a new `Publisher` connected to `relay_url`.
    ///
    /// The publisher takes ownership of `keys` and uses them both for signing
    /// events and as the identity shown to the relay.
    pub async fn new(relay_url: &str, keys: Keys) -> Result<Self> {
        let client = Client::new(keys);

        client
            .add_relay(relay_url)
            .await
            .with_context(|| format!("failed to add relay: {relay_url}"))?;

        client.connect().await;

        info!(relay_url, "publisher connected to relay");

        Ok(Self {
            client,
            relay_url: relay_url.to_string(),
        })
    }

    /// Sign `builder` with the publisher's keys and broadcast to the relay.
    pub async fn publish(&self, builder: EventBuilder) -> Result<()> {
        let output = self
            .client
            .send_event_builder(builder)
            .await
            .with_context(|| format!("failed to publish event to {}", self.relay_url))?;

        debug!(
            relay_url = %self.relay_url,
            event_id = %output.id(),
            "published event",
        );

        Ok(())
    }

    /// Return the relay URL this publisher is connected to.
    pub fn relay_url(&self) -> &str {
        &self.relay_url
    }
}
