#[cfg(feature = "nostr")]
use get_file_hash_core::{get_git_tracked_files, DEFAULT_GNOSTR_KEY, DEFAULT_PICTURE_URL, DEFAULT_BANNER_URL, publish_nostr_event_if_release, get_repo_announcement_event, publish_patch_event};
#[cfg(feature = "nostr")]
#[tokio::main]
async fn main() {
    use get_file_hash_core::publish_patch;
    use nostr_sdk::Keys;
    use nostr_sdk::EventId;
    use std::str::FromStr;

    let keys = Keys::generate();
    let relay_urls = get_file_hash_core::get_relay_urls();
    let d_tag = "my-awesome-repo-example";
    let commit_id = "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0"; // Example commit ID

    // Dummy EventId for examples that require a build_manifest_event_id
    const DUMMY_BUILD_MANIFEST_ID_STR: &str = "f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0";
    let dummy_build_manifest_id = EventId::from_str(DUMMY_BUILD_MANIFEST_ID_STR).unwrap();

    // Example 1: Without build_manifest_event_id
    println!("Publishing patch without build_manifest_event_id...");
    publish_patch!(
        &keys,
        &relay_urls,
        d_tag,
        commit_id,
        "../Cargo.toml" // Use an existing file for the patch content
    );
    println!("Patch without build_manifest_event_id published.");

    // Example 2: With build_manifest_event_id
    println!("Publishing patch with build_manifest_event_id...");
    publish_patch!(
        &keys,
        &relay_urls,
        d_tag,
        commit_id,
        "../Cargo.toml", // Use an existing file for the patch content
        Some(&dummy_build_manifest_id)
    );
    println!("Patch with build_manifest_event_id published.");
}

#[cfg(not(feature = "nostr"))]
fn main() {
    println!("This example requires the 'nostr' feature. Please run with: cargo run --example publish_patch --features nostr");
}
