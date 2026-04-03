use frost_secp256k1_tr as frost;
use frost::{Identifier, round1, round2};
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;
use std::fs;
use std::collections::BTreeMap;

#[cfg(feature = "nostr")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. SETUP: Reload the KeyPackage we saved in the last example
    let p1_json = fs::read_to_string("p1_key.json")
        .map_err(|_| "Run example 6 first to generate p1_key.json")?;
    let p1_key_pkg: frost::keys::KeyPackage = serde_json::from_str(&p1_json)?;
    let p1_id = *p1_key_pkg.identifier();

    println!("--- BIP-64MOD: Distributed Handshake Simulation ---");

    // 2. SIGNER: Round 1 (Generate and Vault)
    // In a real app, the Signer does this and sends the Commitment to a Nostr Relay.
    let mut rng = ChaCha20Rng::from_seed([42u8; 32]);
    let (p1_nonces, p1_commitments) = round1::commit(p1_key_pkg.signing_share(), &mut rng);

    // Securely "vault" the secret nonces (Simulating a local DB or protected file)
    let nonce_json = serde_json::to_string(&p1_nonces)?;
    fs::write("p1_nonce_vault.json", nonce_json)?;
    println!("✅ Signer: Generated Nonce and saved to p1_nonce_vault.json");
    println!("✅ Signer: Shared Public Commitment: {}", hex::encode(p1_commitments.serialize()?));

    // 3. COORDINATOR: Create Signing Request
    // The Coordinator sees the commitment and asks the group to sign a Git Commit.
    let message = b"gnostr-gcc-distributed-commit-xyz123";
    let mut commitments_map = BTreeMap::new();
    commitments_map.insert(p1_id, p1_commitments);
    
    // We mock P2's commitment here to satisfy the 2-of-3 threshold
    let mock_p2_id = Identifier::try_from(2u16)?;
    let mut rng2 = ChaCha20Rng::from_seed([7u8; 32]);
    let (_, p2_commitments) = round1::commit(p1_key_pkg.signing_share(), &mut rng2); // Mocking
    commitments_map.insert(mock_p2_id, p2_commitments);

    let signing_package = frost::SigningPackage::new(commitments_map, message);
    println!("\n🚀 Coordinator: Created Signing Request for message: {:?}", 
        String::from_utf8_lossy(message));

    // 4. SIGNER: Round 2 (Fulfill Request)
    // Signer receives the SigningPackage, reloads their secret nonce, and signs.
    let vaulted_nonce_json = fs::read_to_string("p1_nonce_vault.json")?;
    let p1_reloaded_nonces: round1::SigningNonces = serde_json::from_str(&vaulted_nonce_json)?;

    let p1_share = round2::sign(&signing_package, &p1_reloaded_nonces, &p1_key_pkg)?;
    
    println!("✅ Signer: Fulfilled request with Signature Share: {}", 
        hex::encode(p1_share.serialize()));

    // IMPORTANT: Delete the secret nonce after use to prevent reuse attacks!
    fs::remove_file("p1_nonce_vault.json")?;
    println!("🛡️  Signer: Secret nonce deleted from vault (Reuse Protection).");

    Ok(())
}

#[cfg(not(feature = "nostr"))]
fn main() { println!("Enable nostr feature."); }
