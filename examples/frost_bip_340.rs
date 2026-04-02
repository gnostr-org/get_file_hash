#[cfg(feature = "nostr")]
use frost_secp256k1_tr as frost; // MUST use the -tr variant for BIP-340/Nostr
#[cfg(feature = "nostr")]
use rand::thread_rng;
#[cfg(feature = "nostr")]
use serde_json::json;
#[cfg(feature = "nostr")]
use sha2::{Digest, Sha256};
#[cfg(feature = "nostr")]
use std::collections::BTreeMap;
#[cfg(feature = "nostr")]
use hex;
#[cfg(feature = "nostr")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = thread_rng();
    let (max_signers, min_signers) = (3, 2);

    // 1. Setup Nostr Event Metadata
    let pubkey_hex = "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"; // Example
    let created_at = 1712050000;
    let kind = 1;
    let content = "Hello from ROAST threshold signatures!";
    
    // 2. Serialize for Nostr ID (per NIP-01)
    let event_json = json!([
        0,
        pubkey_hex,
        created_at,
        kind,
        [],
        content
    ]).to_string();
    
    let mut hasher = Sha256::new();
    hasher.update(event_json.as_bytes());
    let event_id = hasher.finalize(); // This 32-byte hash is our signing message

    // 3. FROST/ROAST Key Generation
    let (shares, pubkey_package) = frost::keys::generate_with_dealer(
        max_signers,
        min_signers,
        frost::keys::IdentifierList::Default,
        &mut rng,
    )?;

    // 4. ROAST Coordination Simulation (Round 1: Commitments)
    // In ROAST, the coordinator keeps a "session" open and collects commitments
    let mut session_commitments = BTreeMap::new();
    let mut signer_nonces = BTreeMap::new();

    // Signers 1 and 3 respond first (Signer 2 is offline/slow)
    for &id_val in &[1, 3] {
        let id = frost::Identifier::try_from(id_val as u16)?;
        let (nonces, comms) = frost::round1::commit(shares[&id].signing_share(), &mut rng);
        session_commitments.insert(id, comms);
        signer_nonces.insert(id, nonces);
    }

    // 5. Round 2: Signing the Nostr ID
    let signing_package = frost::SigningPackage::new(session_commitments, &event_id);
    let mut signature_shares = BTreeMap::new();

    for (id, nonces) in signer_nonces {
        let key_package: frost::keys::KeyPackage = shares[&id].clone().try_into()?;
        let share = frost::round2::sign(&signing_package, &nonces, &key_package)?;
        signature_shares.insert(id, share);
    }

    // 6. Aggregate into a BIP-340 Signature
    let group_signature = frost::aggregate(
        &signing_package,
        &signature_shares,
        &pubkey_package,
    )?;

    // 7. Verification (using BIP-340 logic)
    pubkey_package.verifying_key().verify(&event_id, &group_signature)?;

    println!("Nostr Event ID: {}", hex::encode(event_id));
    println!("Threshold Signature (BIP-340): {}", hex::encode(group_signature.serialize()?));
    println!("Successfully signed Nostr event using ROAST/FROST!");

    Ok(())
}

#[cfg(not(feature = "nostr"))]
fn main() {
    println!("This example requires the 'nostr' feature. Please run with: cargo run --example frost_bip_340 --features nostr");
}
