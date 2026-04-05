use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;
use frost_secp256k1_tr as frost;
use frost::{Identifier, keys::IdentifierList, round1, round2};
use std::collections::BTreeMap;

#[cfg(feature = "nostr")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut dealer_rng = ChaCha20Rng::from_seed([0u8; 32]);
    let min_signers = 2;
    let (shares, pubkey_package) = frost::keys::generate_with_dealer(
        3, min_signers, IdentifierList::Default, &mut dealer_rng
    )?;

    // 1. Setup Signer (P1) and peer (P2)
    let p1_id = Identifier::try_from(1u16)?;
    let p2_id = Identifier::try_from(2u16)?;
    
    // 2. Round 1: Both signers generate nonces (Simulating P2's contribution)
    let mut rng1 = ChaCha20Rng::from_seed([1u8; 32]);
    let (p1_nonces, p1_commitments) = round1::commit(shares[&p1_id].signing_share(), &mut rng1);

    let mut rng2 = ChaCha20Rng::from_seed([2u8; 32]);
    let (_p2_nonces, p2_commitments) = round1::commit(shares[&p2_id].signing_share(), &mut rng2);

    // 3. Coordinator: Create a valid SigningPackage with 2 signers
    let message = b"gnostr-gcc-verification-test";
    let mut commitments_map = BTreeMap::new();
    commitments_map.insert(p1_id, p1_commitments);
    commitments_map.insert(p2_id, p2_commitments); // Added P2 to satisfy threshold
    
    let signing_package = frost::SigningPackage::new(commitments_map, message);

    println!("--- BIP-64MOD Round 2: Signer Validation ---");

    // 4. SIGNER-SIDE CHECK (Manual)
    if !signing_package.signing_commitments().contains_key(&p1_id) {
        return Err("Validation Failed: My commitment is missing!".into());
    }

    let commitment_count = signing_package.signing_commitments().len() as u16;
    if commitment_count < min_signers {
         return Err(format!("Validation Failed: Only {} commitments provided, need {}.", commitment_count, min_signers).into());
    }

    println!("✅ Signing Package validated ({} signers).", commitment_count);
    println!("Proceeding to sign message: {:?}", String::from_utf8_lossy(message));

    // 5. Generate the Share
    let p1_verifying_share = frost::keys::VerifyingShare::from(*shares[&p1_id].signing_share());
    let p1_key_package = frost::keys::KeyPackage::new(
        p1_id,
        *shares[&p1_id].signing_share(),
        p1_verifying_share,
        *pubkey_package.verifying_key(),
        min_signers,
    );

    let p1_signature_share = round2::sign(&signing_package, &p1_nonces, &p1_key_package)?;

    println!("\nPartial Signature Share for P1:");
    println!("{}", hex::encode(p1_signature_share.serialize()));

    Ok(())
}

#[cfg(not(feature = "nostr"))]
fn main() { println!("Enable nostr feature."); }
