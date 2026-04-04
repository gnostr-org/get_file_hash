#[cfg(feature = "nostr")]
use rand_chacha::ChaCha20Rng;
#[cfg(feature = "nostr")]
use rand_chacha::rand_core::SeedableRng;
#[cfg(feature = "nostr")]
use hex;

#[cfg(feature = "nostr")]
use frost_secp256k1_tr as frost; 
#[cfg(feature = "nostr")]
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

    println!("\n--- Verifying Shares Against Commitments ---");
    for (identifier, share) in &shares {

        // The Deterministic Values (Scalar Hex)
        // Because your seed is fixed to the EMPTY_BLOB_SHA256,
        // the "redacted" values in your output are always the same.
        // Here are the Secret Signing Shares (the private scalars) for your 2-of-3 setup:
        //
        // Participant,Identifier (x),Signing Share (f(x)) in Hex
        // Participant 1,...0001,757f49553754988450d995c65a0459a0f5a703d7c585f95f468202d09a365f57
        // Participant 2,...0002,a3c4835e32308cb11b43968962290bc9171f1f1ca90c21741890e4f326f9879b
        // Participant 3,...0003,d209bd672d0c80dd65ad974c6a4dc1f138973a618c924988eaaa0715b3bcafdf
        //
        // println!("Participant Identifier: {:?} {:?}", identifier, _share);
        //

        // In FROST, the 'verify' method checks the share against the VSS commitment
        match share.verify() {
            Ok(_) => {
                println!("Participant {:?}: Valid  ✅", identifier);
            }
            Err(e) => {
                println!("Participant {:?}: INVALID! ❌ Error: {:?}", identifier, e);
            }
        }
    }

    let pubkey_bytes = pubkey_package.verifying_key().serialize()?;
    println!("Group Public Key (Hex Compressed): {}", hex::encode(&pubkey_bytes));
    let x_only_hex = hex::encode(&pubkey_bytes[1..]);
    println!("Group Public Key (Hex X-Only):       {}", x_only_hex);

    Ok(())
}

#[cfg(not(feature = "nostr"))]
fn main() {
    println!("Run with --features nostr to enable this example.");
}
