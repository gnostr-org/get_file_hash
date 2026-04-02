use frost_secp256k1 as frost;
use rand::thread_rng;
use std::collections::BTreeMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = thread_rng();
    let max_signers = 3;
    let min_signers = 2;

    ////////////////////////////////////////////////////////////////////////////
    // Round 0: Key Generation (Trusted Dealer)
    ////////////////////////////////////////////////////////////////////////////
    
    // In a real P2P setup, you'd use Distributed Key Generation (DKG).
    // For local testing/simulations, the trusted dealer is faster.
    let (shares, pubkey_package) = frost::keys::generate_with_dealer(
        max_signers,
        min_signers,
        frost::keys::IdentifierList::Default,
        &mut rng,
    )?;

    // Verifying the public key exists
    let group_public_key = pubkey_package.verifying_key();
    println!("Group Public Key: {:?}", group_public_key);

    ////////////////////////////////////////////////////////////////////////////
    // Round 1: Commitment
    ////////////////////////////////////////////////////////////////////////////
    
    let message = b"BIP-64MOD Consensus Proposal";
    let mut signing_commitments = BTreeMap::new();
    let mut participant_nonces = BTreeMap::new();

    // Participants 1 and 2 decide to sign
    for i in 1..=min_signers {
        let identifier = frost::Identifier::try_from(i as u16)?;
        
        // Generate nonces and commitments
        let (nonces, commitments) = frost::round1::commit(
            shares[&identifier].signing_share(),
            &mut rng,
        );
        
        signing_commitments.insert(identifier, commitments);
        participant_nonces.insert(identifier, nonces);
    }

    ////////////////////////////////////////////////////////////////////////////
    // Round 2: Signing
    ////////////////////////////////////////////////////////////////////////////
    
    let mut signature_shares = BTreeMap::new();
    let signing_package = frost::SigningPackage::new(signing_commitments, message);

    for i in 1..=min_signers {
        let identifier = frost::Identifier::try_from(i as u16)?;
        let nonces = &participant_nonces[&identifier];
        
        // Each participant produces a signature share
        let key_package: frost::keys::KeyPackage = shares[&identifier].clone().try_into()?;
        let share = frost::round2::sign(&signing_package, nonces, &key_package)?;
        signature_shares.insert(identifier, share);
    }

    ////////////////////////////////////////////////////////////////////////////
    // Finalization: Aggregation
    ////////////////////////////////////////////////////////////////////////////
    
    let group_signature = frost::aggregate(
        &signing_package,
        &signature_shares,
        &pubkey_package,
    )?;

    // Verification
    group_public_key.verify(message, &group_signature)?;
    
    println!("Threshold signature verified successfully!");
    Ok(())
}
