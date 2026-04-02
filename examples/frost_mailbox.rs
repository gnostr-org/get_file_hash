
#[cfg(feature = "nostr")]
use frost_secp256k1_tr as frost;
#[cfg(feature = "nostr")]
use frost::keys::PublicKeyPackage;
#[cfg(feature = "nostr")]
use frost::round2::SignatureShare;
#[cfg(feature = "nostr")]
use frost::SigningPackage;
#[cfg(feature = "nostr")]
use hex;

#[cfg(feature = "nostr")]
fn process_relay_share(
    relay_payload_hex: &str,
    signer_id_u16: u16,
    _signing_package: &SigningPackage,
    _pubkey_package: &PublicKeyPackage,
) -> Result<(), Box<dyn std::error::Error>> {
    // In a real scenario, this function would deserialize the share, perform
    // individual verification, and store it for aggregation.
    // For this example, we'll just acknowledge receipt.
    let _share_bytes = hex::decode(relay_payload_hex)?;
    let _share = SignatureShare::deserialize(&_share_bytes)?;
    let _identifier = frost::Identifier::try_from(signer_id_u16)?;

    println!("✅ Share from Signer {} processed (simplified).", signer_id_u16);
    Ok(())
}


#[cfg(feature = "nostr")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use rand::thread_rng;
    use std::collections::BTreeMap;

    // This example simulates a coordinator listening for and processing FROST signature shares
    // posted to a Nostr "mailbox" relay.
    // The general workflow is:
    // 1. Coordinator sends a request for signatures (e.g., on a BIP-64MOD proposal).
    // 2. Signers receive the proposal, perform local verification.
    // 3. Each signer generates their signature share and posts it (encrypted) to a
    //    Nostr relay, targeting the coordinator's mailbox.
    // 4. The coordinator collects enough shares to aggregate the final signature.

    let mut rng = thread_rng();
    // For this example, we simulate a 2-of-2 threshold for simplicity.
    let (max_signers, min_signers) = (2, 2);

    ////////////////////////////////////////////////////////////////////////////
    // 1. Key Generation (Simulated Trusted Dealer for Coordinator's context)
    ////////////////////////////////////////////////////////////////////////////
    // In a real distributed setup, this would be DKG. Here, a "trusted dealer"
    // generates the shares and public key package, which the coordinator needs.
    let (shares, pubkey_package) = frost::keys::generate_with_dealer(
        max_signers,
        min_signers,
        frost::keys::IdentifierList::Default,
        &mut rng,
    )?;

    // Get key packages for our simulated signers
    let signer1_id = frost::Identifier::try_from(1 as u16)?;
    let key_package1: frost::keys::KeyPackage = shares[&signer1_id].clone().try_into()?;
    let signer2_id = frost::Identifier::try_from(2 as u16)?;
    let key_package2: frost::keys::KeyPackage = shares[&signer2_id].clone().try_into()?;

    // The message that is to be signed (e.g., a hash of a Git commit or a Nostr event ID).
    let message = b"BIP-64MOD: Anchor Data Proposal v1";

    ////////////////////////////////////////////////////////////////////////////
    // 2. Simulated Round 1: Commitment Phase (Coordinator receives commitments)
    ////////////////////////////////////////////////////////////////////////////
    // In a real system, the coordinator would receive commitments from signers.
    // Here, we simulate by generating them directly.
    let (nonces1, comms1) = frost::round1::commit(key_package1.signing_share(), &mut rng);
    let (nonces2, comms2) = frost::round1::commit(key_package2.signing_share(), &mut rng);

    let mut session_commitments = BTreeMap::new();
    session_commitments.insert(signer1_id, comms1);
    session_commitments.insert(signer2_id, comms2);

    ////////////////////////////////////////////////////////////////////////////
    // 3. Signing Package Creation (Coordinator's role)
    ////////////////////////////////////////////////////////////////////////////
    // The coordinator creates the signing package based on the message and received commitments.
    let signing_package = frost::SigningPackage::new(session_commitments.clone(), message);

    ////////////////////////////////////////////////////////////////////////////
    // 4. Simulated Signer Actions (Signers generate and post shares)
    ////////////////////////////////////////////////////////////////////////////
    // In a real system, signers would individually generate these shares and post
    // them to the Nostr relay, potentially encrypted.

    // Signer 1 generates share
    let share1 = frost::round2::sign(&signing_package, &nonces1, &key_package1)?;
    let share1_hex = hex::encode(share1.serialize());

    // Signer 2 generates share
    let share2 = frost::round2::sign(&signing_package, &nonces2, &key_package2)?;
    let share2_hex = hex::encode(share2.serialize());

    ////////////////////////////////////////////////////////////////////////////
    // 5. Coordinator Processes Shares from Mailbox
    ////////////////////////////////////////////////////////////////////////////
    println!("Coordinator listening for Nostr events (simulated)...");

    // Simulate receiving share from Signer 1 via the mailbox
    process_relay_share(&share1_hex, 1_u16, &signing_package, &pubkey_package)?;
    // Simulate receiving share from Signer 2 via the mailbox
    process_relay_share(&share2_hex, 2_u16, &signing_package, &pubkey_package)?;
    println!("All required shares processed. Coordinator would now aggregate.");

    Ok(())
}

#[cfg(not(feature = "nostr"))]
fn main() {
    println!("This example requires the 'nostr' feature. Please run with: cargo run --example frost_mailbox --features nostr");
}
