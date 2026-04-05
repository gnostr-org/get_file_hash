use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::SeedableRng;
use frost_secp256k1_tr as frost;
use frost::{Identifier, keys::IdentifierList, round1, round2};
use std::collections::BTreeMap;

#[cfg(feature = "nostr")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Dealer Setup
    let mut dealer_rng = ChaCha20Rng::from_seed([0u8; 32]);
    let min_signers = 2;
    let (shares, pubkey_package) = frost::keys::generate_with_dealer(
        3, min_signers, IdentifierList::Default, &mut dealer_rng
    )?;

    // 2. Setup Participant Identifiers
    let p1_id = Identifier::try_from(1u16)?;
    let p2_id = Identifier::try_from(2u16)?;

    // 3. Construct KeyPackages manually for RC.0
    let p1_verifying_share = frost::keys::VerifyingShare::from(*shares[&p1_id].signing_share());
    let p1_key_package = frost::keys::KeyPackage::new(
        p1_id,
        *shares[&p1_id].signing_share(),
        p1_verifying_share,
        *pubkey_package.verifying_key(),
        min_signers,
    );

    let p2_verifying_share = frost::keys::VerifyingShare::from(*shares[&p2_id].signing_share());
    let p2_key_package = frost::keys::KeyPackage::new(
        p2_id,
        *shares[&p2_id].signing_share(),
        p2_verifying_share,
        *pubkey_package.verifying_key(),
        min_signers,
    );

    // 4. Round 1: Commitments
    let mut rng1 = ChaCha20Rng::from_seed([1u8; 32]);
    let (p1_nonces, p1_commitments) = round1::commit(p1_key_package.signing_share(), &mut rng1);

    let mut rng2 = ChaCha20Rng::from_seed([2u8; 32]);
    let (p2_nonces, p2_commitments) = round1::commit(p2_key_package.signing_share(), &mut rng2);

    // 5. Coordinator: Signing Package
    let message = b"gnostr-commit-7445bd727dbce5bac004861a45c35ccd4f4a195bfb1cc39f2a7c9fd3aa3b6547";
    let mut commitments_map = BTreeMap::new();
    commitments_map.insert(p1_id, p1_commitments);
    commitments_map.insert(p2_id, p2_commitments);

    let signing_package = frost::SigningPackage::new(commitments_map, message);

    // 6. Round 2: Partial Signatures
    let p1_signature_share = round2::sign(&signing_package, &p1_nonces, &p1_key_package)?;
    let p2_signature_share = round2::sign(&signing_package, &p2_nonces, &p2_key_package)?;

    // 7. Aggregation
    let mut signature_shares = BTreeMap::new();
    signature_shares.insert(p1_id, p1_signature_share);
    signature_shares.insert(p2_id, p2_signature_share);

    let group_signature = frost::aggregate(&signing_package, &signature_shares, &pubkey_package)?;

    println!("--- BIP-64MOD Aggregated Signature ---");
    println!("Final Signature (Hex): {}", hex::encode(group_signature.serialize()?));

    // Final Verification
    pubkey_package.verifying_key().verify(message, &group_signature)?;
    println!("🛡️  Signature is valid for the 2nd generation group!");

    Ok(())
}

#[cfg(not(feature = "nostr"))]
fn main() { println!("Run with --features nostr"); }
