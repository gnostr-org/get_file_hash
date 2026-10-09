//! Mine a Nostr event whose ID starts with a chosen hex prefix.
//!
//! This example uses [`get_file_hash::mine_event_with_prefix`] from the crate's
//! public API. The SDK's [`EventBuilder::pow`](nostr_sdk::EventBuilder::pow)
//! only supports NIP-13 leading-zero-bits difficulty; custom-prefix mining is
//! done by brute-forcing a `["nonce", "<nonce>", "0"]` tag.
//!
//! Run with:
//!     cargo run --example pow_miner --release --features nostr -- <hex-prefix> <content>
//!
//! Example:
//!     cargo run --example pow_miner --release --features nostr -- 0000 "hello world"

use std::env;
use std::time::Instant;

use get_file_hash::mine_event_with_prefix_or_default;
use nostr_sdk::nostr::util::JsonUtil;
use nostr_sdk::{Client, EventBuilder, Keys, Tag};

const DEFAULT_RELAY: &str = "wss://relay.damus.io";
const MAX_ATTEMPTS: u128 = 100_000_000;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let prefix = args.get(1).map(|s| s.as_str()).unwrap_or("0000");
    let content = args
        .get(2)
        .map(|s| s.as_str())
        .unwrap_or("hello from pow_miner");

    if prefix.is_empty() {
        eprintln!("prefix must not be empty");
        return;
    }

    let keys = Keys::generate(); //TODO padded commit hash?
    println!(
        "mining event with prefix '{}' for pubkey {}",
        prefix,
        keys.public_key()
    );

    let builder = EventBuilder::text_note(content)
        .tag(Tag::parse(["example", "pow_miner"]).unwrap());

    let start = Instant::now();
    match mine_event_with_prefix_or_default(&keys, builder, prefix) {
        Some(event) => {
            println!(
                "mined event id: {} elapsed={:?}",
                event.id,
                start.elapsed()
            );
            println!("{}", event.as_json());

            // Optional: send the mined event to a relay.
            let client = Client::new(keys);
            if let Err(e) = client.add_relay(DEFAULT_RELAY).await {
                println!("failed to add relay {}: {}", DEFAULT_RELAY, e);
                return;
            }
            client.connect().await;
            match client.send_event(&event).await {
                Ok(output) => println!(
                    "published to {} relay(s): id={}",
                    output.success.len(),
                    output.val
                ),
                Err(e) => println!("failed to publish: {}", e),
            }
        }
        None => eprintln!(
            "failed to mine event with prefix '{}' after {} attempts",
            prefix, MAX_ATTEMPTS
        ),
    }
}
