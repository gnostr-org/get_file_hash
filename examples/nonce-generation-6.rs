use frost_secp256k1_tr as frost;
use frost::{Identifier, keys::IdentifierList, round1, round2};
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;
use std::fs;

#[cfg(feature = "nostr")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. SETUP: Initial Key Generation (The "Genesis" event)
    let mut dealer_rng = ChaCha20Rng::from_seed([0u8; 32]);
    let min_signers = 2;
    let (shares, pubkey_package) = frost::keys::generate_with_dealer(
        3, min_signers, IdentifierList::Default, &mut dealer_rng
    )?;

    // 2. PERSISTENCE: Save Participant 1's KeyPackage to a file
    let p1_id = Identifier::try_from(1u16)?;
    let p1_key_pkg = frost::keys::KeyPackage::new(
        p1_id,
        *shares[&p1_id].signing_share(),
        frost::keys::VerifyingShare::from(*shares[&p1_id].signing_share()),
        *pubkey_package.verifying_key(),
        min_signers,
    );

    // Serialize to JSON (standard for many Nostr/Git tools)
    let p1_json = serde_json::to_string_pretty(&p1_key_pkg)?;
    fs::write("p1_key.json", p1_json)?;
    
    let pub_json = serde_json::to_string_pretty(&pubkey_package)?;
    fs::write("group_public.json", pub_json)?;

    println!("--- BIP-64MOD: Key Persistence ---");
    println!("✅ Saved p1_key.json and group_public.json to disk.");

    // 3. RELOAD: Simulate a Signer waking up later
    let p1_loaded_json = fs::read_to_string("p1_key.json")?;
    let p1_reloaded_pkg: frost::keys::KeyPackage = serde_json::from_str(&p1_loaded_json)?;

    println!("✅ Reloaded KeyPackage for Participant: {:?}", p1_reloaded_pkg.identifier());

    // 4. SIGN: Use the reloaded key to sign a new Git Commit Hash
    let mut rng = ChaCha20Rng::from_seed([100u8; 32]); // Fresh seed for this specific signing session
    let (nonces, commitments) = round1::commit(p1_reloaded_pkg.signing_share(), &mut rng);

    println!("\nGenerated Nonce for new session:");
    println!("  Commitment: {}", hex::encode(commitments.serialize()?));

    // Cleanup files for the example
    // fs::remove_file("p1_key.json")?;
    // fs::remove_file("group_public.json")?;

    Ok(())
}

#[cfg(not(feature = "nostr"))]
fn main() { println!("Enable nostr feature."); }
