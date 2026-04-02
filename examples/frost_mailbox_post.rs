#[cfg(feature = "nostr")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    get_file_hash_core::frost_mailbox_logic::simulate_frost_mailbox_post_signer()
}
#[cfg(not(feature = "nostr"))]
fn main() {
    println!("This example requires the 'nostr' feature. Please run with: cargo run --example frost_mailbox_post --features nostr");
}
