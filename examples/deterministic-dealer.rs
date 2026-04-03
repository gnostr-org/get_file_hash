use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;
use hex;

// Adjust this import based on your specific FROST crate (e.g., frost_rerandomized, frost_secp256k1)
use frost_secp256k1_tr as frost; 
use frost::keys::IdentifierList;

#[cfg(feature = "nostr")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create a deterministic seed (e.g., 32 bytes of zeros or a Git Hash)
    let seed_hex = "473a0f4c3be8a93681a267e3b1e9a7dcda1185436fe141f7749120a303721813";
    let seed_bytes = hex::decode(seed_hex)?;
    let mut rng = ChaCha20Rng::from_seed(seed_bytes.try_into().map_err(|_| "Invalid seed length")?);

    let max_signers = 3;
    let min_signers = 2;

    ////////////////////////////////////////////////////////////////////////////
    // Round 0: Key Generation (Trusted Dealer)
    ////////////////////////////////////////////////////////////////////////////

    // Using IdentifierList::Default creates identifiers 1, 2, 3...
    let (shares, pubkey_package) = frost::keys::generate_with_dealer(
        max_signers,
        min_signers,
        IdentifierList::Default,
        &mut rng,
    )?;

    println!("--- Deterministic FROST Dealer ---");
    println!("Threshold: {} of {}", min_signers, max_signers);
    println!("Number of shares generated: {}", shares.len()); 

    for (identifier, _share) in &shares {
        println!("Participant Identifier: {:?}", identifier);
    }

    let pubkey_bytes = pubkey_package.verifying_key().serialize()?;
    println!("Group Public Key (Hex): {}", hex::encode(pubkey_bytes));

    Ok(())
}

#[cfg(not(feature = "nostr"))]
fn main() {
    println!("Run with --features nostr to enable this example.");
}
