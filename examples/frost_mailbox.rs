
#[cfg(feature = "nostr")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    get_file_hash_core::frost_mailbox_logic::simulate_frost_mailbox_coordinator()
}

#[cfg(not(feature = "nostr"))]
fn main() {
    println!("This example requires the 'nostr' feature. Please run with: cargo run --example frost_mailbox --features nostr");
}
