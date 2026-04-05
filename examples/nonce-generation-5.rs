use frost_secp256k1_tr as frost;
use frost::{Identifier, keys::IdentifierList, round1, round2};
use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;
use std::collections::BTreeMap;

#[cfg(feature = "nostr")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. RECREATE CONTEXT (Same as Signer)
    let mut dealer_rng = ChaCha20Rng::from_seed([0u8; 32]);
    let min_signers = 2;
    let (shares, pubkey_package) = frost::keys::generate_with_dealer(
        3, min_signers, IdentifierList::Default, &mut dealer_rng
    )?;

    let p1_id = Identifier::try_from(1u16)?;
    let p2_id = Identifier::try_from(2u16)?;

    // 2. SIMULATE SIGNING (Round 1 & 2)
    let mut rng1 = ChaCha20Rng::from_seed([1u8; 32]);
    let (p1_nonces, p1_commitments) = round1::commit(shares[&p1_id].signing_share(), &mut rng1);
    let mut rng2 = ChaCha20Rng::from_seed([2u8; 32]);
    let (p2_nonces, p2_commitments) = round1::commit(shares[&p2_id].signing_share(), &mut rng2);

    let message = b"gnostr-gcc-verification-test";
    let mut commitments_map = BTreeMap::new();
    commitments_map.insert(p1_id, p1_commitments);
    commitments_map.insert(p2_id, p2_commitments);
    let signing_package = frost::SigningPackage::new(commitments_map, message);

    // Generate shares (using the KeyPackage method we perfected)
    let p1_key_pkg = frost::keys::KeyPackage::new(p1_id, *shares[&p1_id].signing_share(), 
        frost::keys::VerifyingShare::from(*shares[&p1_id].signing_share()), 
        *pubkey_package.verifying_key(), min_signers);
    let p2_key_pkg = frost::keys::KeyPackage::new(p2_id, *shares[&p2_id].signing_share(), 
        frost::keys::VerifyingShare::from(*shares[&p2_id].signing_share()), 
        *pubkey_package.verifying_key(), min_signers);

    let p1_sig_share = round2::sign(&signing_package, &p1_nonces, &p1_key_pkg)?;
    let p2_sig_share = round2::sign(&signing_package, &p2_nonces, &p2_key_pkg)?;

    // 3. COORDINATOR: AGGREGATION
    println!("--- BIP-64MOD: Coordinator Aggregation ---");
    
    let mut shares_map = BTreeMap::new();
    shares_map.insert(p1_id, p1_sig_share);
    shares_map.insert(p2_id, p2_sig_share);

    let final_signature = frost::aggregate(
        &signing_package, 
        &shares_map, 
        &pubkey_package
    )?;

    let sig_bytes = final_signature.serialize()?;
    println!("✅ Aggregation Successful!");
    println!("Final Signature (Hex): {}", hex::encode(&sig_bytes));

    // 4. VERIFICATION (The moment of truth)
    pubkey_package.verifying_key().verify(message, &final_signature)?;
    println!("🛡️  Signature Verified against Group Public Key!");

    Ok(())
}

#[cfg(not(feature = "nostr"))]
fn main() { println!("Enable nostr feature."); }
