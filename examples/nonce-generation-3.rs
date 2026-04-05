use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;
use frost_secp256k1_tr as frost;
use frost::{Identifier, keys::IdentifierList, round1};
use std::collections::BTreeMap;

#[cfg(feature = "nostr")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup deterministic dealer (Genesis State)
    let mut dealer_rng = ChaCha20Rng::from_seed([0u8; 32]);
    let (shares, _pubkey_package) = frost::keys::generate_with_dealer(
        3, 2, IdentifierList::Default, &mut dealer_rng
    )?;

    // 2. Setup Participant 1
    let p1_id = Identifier::try_from(1u16)?;
    let p1_share = &shares[&p1_id];
    
    // 3. Setup Nonce RNG
    let mut nonce_rng = ChaCha20Rng::from_seed([1u8; 32]);

    println!("--- BIP-64MOD Round 1: Batch Nonce Generation ---");
    println!("Participant: {:?}", p1_id);
    println!("Generating 10 Nonce Pairs...\n");

    let mut batch_commitments = BTreeMap::new();
    let mut batch_secrets = Vec::new();

    for i in 0..10 {
        // Generate a single pair
        let (nonces, commitments) = round1::commit(p1_share.signing_share(), &mut nonce_rng);
        
        // Store the secret nonces locally (index i)
        batch_secrets.push(nonces);
        
        // Store the public commitments in a map to share with the Coordinator
        batch_commitments.insert(i, commitments);

        println!("Nonce Pair [{}]:", i);
        println!("  Hiding:  {}", hex::encode(commitments.hiding().serialize()?));
        println!("  Binding: {}", hex::encode(commitments.binding().serialize()?));
    }

    // 4. Persistence Simulation
    // In a real GCC app, you would save `batch_secrets` to an encrypted file 
    // and send `batch_commitments` to a Nostr Relay (Kind 1351).
    println!("\n✅ Batch generation complete.");
    println!("Ready to sign up to 10 independent Git commits.");

    Ok(())
}

#[cfg(not(feature = "nostr"))]
fn main() { println!("Run with --features nostr"); }
