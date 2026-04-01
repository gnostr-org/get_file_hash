#[tokio::main]
#[cfg(feature = "nostr")]
async fn main() {
    use get_file_hash_core::publish_pr_update;
    use nostr_sdk::Keys;
    use nostr_sdk::EventId;
    use std::str::FromStr;

    let keys = Keys::generate();
    let relay_urls = get_file_hash_core::get_relay_urls();
    let d_tag = "my-awesome-repo-example";
    let pr_event_id = EventId::from_str("f6e4d6a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9").unwrap(); // Example PR Event ID
    let updated_commit_id = "z9y8x7w6v5u4t3s2r1q0p9o8n7m6l5k4j3i2h1g0";
    let updated_clone_url = "git@github.com:user/my-feature-branch-v2.git";

    // Dummy EventId for examples that require a build_manifest_event_id
    const DUMMY_BUILD_MANIFEST_ID_STR: &str = "f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0";
    let dummy_build_manifest_id = EventId::from_str(DUMMY_BUILD_MANIFEST_ID_STR).unwrap();

    // Example 1: Without build_manifest_event_id
    println!("Publishing PR update without build_manifest_event_id...");
    publish_pr_update!(
        &keys,
        &relay_urls,
        d_tag,
        &pr_event_id,
        updated_commit_id,
        updated_clone_url
    );
    println!("PR update without build_manifest_event_id published.");

    // Example 2: With build_manifest_event_id
    println!("Publishing PR update with build_manifest_event_id...");
    publish_pr_update!(
        &keys,
        &relay_urls,
        d_tag,
        &pr_event_id,
        updated_commit_id,
        updated_clone_url,
        Some(&dummy_build_manifest_id)
    );
    println!("PR update with build_manifest_event_id published.");
}

#[cfg(not(feature = "nostr"))]
fn main() {
    println!("This example requires the 'nostr' feature. Please run with: cargo run --example publish_pr_update --features nostr");
}
