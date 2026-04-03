use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;
use frost_secp256k1_tr as frost;
use frost::{Identifier, keys::IdentifierList};

#[cfg(feature = "nostr")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. We need the dealer setup first to get a real SigningShare
    let dealer_seed = [0u8; 32]; 
    let mut dealer_rng = ChaCha20Rng::from_seed(dealer_seed);
    
    let (shares, _pubkey_package) = frost::keys::generate_with_dealer(
        3, 2, IdentifierList::Default, &mut dealer_rng
    )?;

    // 2. Setup nonce RNG
    let nonce_seed = [1u8; 32]; 
    let mut rng = ChaCha20Rng::from_seed(nonce_seed);

    // 3. Get Participant 1's share
    let p1_id = Identifier::try_from(1u16)?;
    let p1_share = shares.get(&p1_id).ok_or("Share not found")?;

    ////////////////////////////////////////////////////////////////////////////
    // Round 1: Commitments & Nonces
    ////////////////////////////////////////////////////////////////////////////
    
    // In RC.0, commit() requires the secret share reference
    let (p1_nonces, p1_commitments) = frost::round1::commit(p1_share.signing_share(), &mut rng);

    println!("--- BIP-64MOD Round 1: Nonce Generation ---");
    println!("Participant Identifier: {:?}", p1_id);
    
    // 4. Handle Results for serialization
    println!("\nPublic Signing Commitments (To be shared):");
    println!("  Hiding:  {}", hex::encode(p1_commitments.hiding().serialize()?));
    println!("  Binding: {}", hex::encode(p1_commitments.binding().serialize()?));

    // Keep nonces in memory for the next step
    let _p1_secret_nonces = p1_nonces; 
    
    println!("\n✅ Nonces generated and tied to Participant 1's share.");

    Ok(())
}

#[cfg(not(feature = "nostr"))]
fn main() {
    println!("Run with --features nostr to enable this example.");
}
