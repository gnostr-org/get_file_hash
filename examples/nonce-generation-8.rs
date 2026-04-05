use frost_secp256k1_tr as frost;
use frost::{Identifier, round1, round2};
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;
use std::collections::BTreeMap;
use std::fs;

#[cfg(feature = "nostr")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load the persistent KeyPackage
    let p1_json = fs::read_to_string("p1_key.json")?;
    let p1_key_pkg: frost::keys::KeyPackage = serde_json::from_str(&p1_json)?;
    let p1_id = *p1_key_pkg.identifier();

    println!("--- BIP-64MOD: Batch Nonce Management ---");

    // 2. BATCH GENERATION (The "Public Offer")
    let mut rng = ChaCha20Rng::from_seed([88u8; 32]);
    let mut public_commitments = BTreeMap::new();
    let mut secret_nonce_vault = BTreeMap::new();

    for i in 0..5 {
        let (nonces, commitments) = round1::commit(p1_key_pkg.signing_share(), &mut rng);
        public_commitments.insert(i, commitments);
        secret_nonce_vault.insert(i, nonces);
    }

    // Save the vault (Private)
    fs::write("p1_batch_vault.json", serde_json::to_string(&secret_nonce_vault)?)?;
    println!("✅ Signer: Generated 5 nonces and saved to p1_batch_vault.json");

    // 3. COORDINATOR REQUEST (Choosing Index #3)
    let message = b"gnostr-gcc-batch-commit-hash-003";
    let selected_index: u64 = 3;
    
    let mut commitments_map = BTreeMap::new();
    // Coordinator uses P1's commitment at the specific index
    commitments_map.insert(p1_id, public_commitments[&selected_index]);
    
    // Mock P2 to satisfy threshold
    let mock_p2_id = Identifier::try_from(2u16)?;
    let (_, p2_commitments) = round1::commit(p1_key_pkg.signing_share(), &mut rng);
    commitments_map.insert(mock_p2_id, p2_commitments);

    let signing_package = frost::SigningPackage::new(commitments_map, message);
    println!("\n🚀 Coordinator: Requesting signature for Index #{}", selected_index);

    // 4. SIGNER: Selective Fulfillment
    let mut current_vault: BTreeMap<u64, round1::SigningNonces> = 
        serde_json::from_str(&fs::read_to_string("p1_batch_vault.json")?)?;

    // Extract only the requested nonce
    if let Some(p1_nonces) = current_vault.remove(&selected_index) {
        let p1_share = round2::sign(&signing_package, &p1_nonces, &p1_key_pkg)?;
        
        // Save the updated vault (Index 3 is now GONE)
        fs::write("p1_batch_vault.json", serde_json::to_string(&current_vault)?)?;
        
        println!("✅ Signer: Signed message using Index #{}", selected_index);
        println!("✅ Signer: Partial Signature: {}", hex::encode(p1_share.serialize()));
        println!("🛡️  Signer: Index #{} purged from vault. {} nonces remain.", 
            selected_index, current_vault.len());
    } else {
        println!("❌ Error: Nonce index {} has already been used!", selected_index);
    }

    Ok(())
}

#[cfg(not(feature = "nostr"))]
fn main() { println!("Enable nostr feature."); }
