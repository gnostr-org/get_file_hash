#[cfg(feature = "nostr")]
use get_file_hash_core::{get_relay_urls, publish_patch, publish_metadata_event, DEFAULT_PICTURE_URL, DEFAULT_BANNER_URL};
#[cfg(feature = "nostr")]
#[tokio::main]
async fn main() {
    use nostr_sdk::Keys;
    use nostr_sdk::EventId;
    use std::str::FromStr;

    let keys = Keys::generate();
    let relay_urls = get_relay_urls();
    let d_tag = "my-gnostr-repository-patch-with-metadata-example"; // Repository identifier
    let commit_id = "f1e2d3c4b5a6f7e8d9c0b1a2f3e4d5c6b7a8f9e0"; // Example commit ID

    // Metadata for NIP-01 event
    let picture_url = DEFAULT_PICTURE_URL;
    let banner_url = DEFAULT_BANNER_URL;
    let metadata_file_path = "./README.md"; // Using README.md content for metadata

    // Dummy EventId for examples that require a build_manifest_event_id
    const DUMMY_BUILD_MANIFEST_ID_STR: &str = "f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0";
    let dummy_build_manifest_id = EventId::from_str(DUMMY_BUILD_MANIFEST_ID_STR).unwrap();

    println!("Publishing NIP-01 Metadata Event...");
    publish_metadata_event(
        &keys,
        &relay_urls,
        picture_url,
        banner_url,
        metadata_file_path
    ).await;
    println!("NIP-01 Metadata Event published.");

    println!("
Publishing NIP-34 Patch Event without build_manifest_event_id...");
    publish_patch!(
        &keys,
        &relay_urls,
        d_tag,
        commit_id,
        "../Cargo.toml" // Use an existing file for the patch content
    );
    println!("NIP-34 Patch Event without build_manifest_event_id published.");

    println!("
Publishing NIP-34 Patch Event with build_manifest_event_id...");
    publish_patch!(
        &keys,
        &relay_urls,
        d_tag,
        commit_id,
        "../Cargo.toml", // Use an existing file for the patch content
        Some(&dummy_build_manifest_id)
    );
    println!("NIP-34 Patch Event with build_manifest_event_id published.");
}

#[cfg(not(feature = "nostr"))]
fn main() {
    println!("This example requires the 'nostr' feature. Please run with: cargo run --example publish_patch_with_metadata --features nostr");
}
