//! Mine a Nostr event whose ID starts with a chosen hex prefix.
//!
//! Run with:
//!     cargo run --example pow_miner --release --features nostr -- <hex-prefix> <content>
//!
//! Example:
//!     cargo run --example pow_miner --release --features nostr -- ff "hello world"

use std::env;
use std::time::Instant;

use nostr_sdk::nostr::util::JsonUtil;
use nostr_sdk::{Client, Event, EventBuilder, EventId, Keys, Tag, UnsignedEvent};

const DEFAULT_RELAY: &str = "wss://relay.damus.io";
const MAX_ATTEMPTS: u128 = 100_000_000;

/// Mine an event whose ID starts with `prefix` (lowercase hex).
///
/// The SDK's `EventBuilder::pow(n)` only supports NIP-13 leading-zero-bits
/// difficulty. This function does custom-prefix mining by brute-forcing a
/// `nonce` tag until the computed event ID matches the desired prefix.
fn mine_event_with_prefix(
    keys: &Keys,
    builder: EventBuilder,
    prefix: &str,
) -> Option<Event> {
    let pubkey = keys.public_key();
    let mut nonce: u128 = 0;
    let start = Instant::now();

    loop {
        let mut attempt = builder.clone();
        attempt = attempt.tag(Tag::pow(nonce, 0));

        let mut unsigned: UnsignedEvent = attempt.build(pubkey);
        let id: EventId = unsigned.id();

        if id.to_hex().starts_with(prefix) {
            let elapsed = start.elapsed();
            println!(
                "mined nonce={} attempts={} elapsed={:?}",
                nonce,
                nonce + 1,
                elapsed
            );
            return unsigned.sign_with_keys(keys).ok();
        }

        nonce += 1;
        if nonce > MAX_ATTEMPTS {
            println!("gave up after {} attempts", MAX_ATTEMPTS);
            return None;
        }
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let prefix = args.get(1).map(|s| s.as_str()).unwrap_or("ff");
    let content = args
        .get(2)
        .map(|s| s.as_str())
        .unwrap_or("hello from pow_miner");

    if prefix.is_empty() {
        eprintln!("prefix must not be empty");
        return;
    }

    // Accept uppercase input but event IDs are lowercase hex.
    let prefix = prefix.to_ascii_lowercase();

    let keys = Keys::generate();
    println!(
        "mining event with prefix '{}' for pubkey {}",
        prefix,
        keys.public_key()
    );

    let builder = EventBuilder::text_note(content)
        .tag(Tag::parse(["example", "pow_miner"]).unwrap());

    match mine_event_with_prefix(&keys, builder, &prefix) {
        Some(event) => {
            println!("mined event id: {}", event.id);
            println!("{}", event.as_json());

            // Optional: send the mined event to a relay.
            let client = Client::new(keys);
            if let Err(e) = client.add_relay(DEFAULT_RELAY).await {
                println!("failed to add relay {}: {}", DEFAULT_RELAY, e);
                return;
            }
            client.connect().await;
            match client.send_event(&event).await {
                Ok(output) => println!("published to {} relay(s): id={}", output.success.len(), output.val),
                Err(e) => println!("failed to publish: {}", e),
            }
        }
        None => eprintln!("failed to mine event with prefix '{}'", prefix),
    }
}
