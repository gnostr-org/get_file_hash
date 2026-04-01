#[tokio::main]
#[cfg(feature = "nostr")]
async fn main() {
    use get_file_hash_core::publish_pull_request;
    use nostr_sdk::Keys;
    use nostr_sdk::EventId;
    use std::str::FromStr;

    let keys = Keys::generate();
    let relay_urls = get_file_hash_core::get_relay_urls();
    let d_tag = "my-awesome-repo-example";
    let commit_id = "0123456789abcdef0123456789abcdef01234567";
    let clone_url = "git@github.com:user/my-feature-branch.git";
    let title = Some("Feat: Add new awesome feature example");

    // Dummy EventId for examples that require a build_manifest_event_id
    const DUMMY_BUILD_MANIFEST_ID_STR: &str = "f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0";
    let dummy_build_manifest_id = EventId::from_str(DUMMY_BUILD_MANIFEST_ID_STR).unwrap();

    // Example 1: Without title and build_manifest_event_id
    println!("Publishing pull request without title and build_manifest_event_id...");
    publish_pull_request!(
        &keys,
        &relay_urls,
        d_tag,
        commit_id,
        clone_url
    );
    println!("Pull request without title and build_manifest_event_id published.");

    // Example 2: With title but without build_manifest_event_id
    println!("Publishing pull request with title but without build_manifest_event_id...");
    publish_pull_request!(
        &keys,
        &relay_urls,
        d_tag,
        commit_id,
        clone_url,
        title
    );
    println!("Pull request with title but without build_manifest_event_id published.");

    // Example 3: With build_manifest_event_id but without title
    println!("Publishing pull request with build_manifest_event_id but without title...");
    publish_pull_request!(
        &keys,
        &relay_urls,
        d_tag,
        commit_id,
        clone_url,
        Some(&dummy_build_manifest_id)
    );
    println!("Pull request with build_manifest_event_id but without title published.");

    // Example 4: With title and build_manifest_event_id
    println!("Publishing pull request with title and build_manifest_event_id...");
    publish_pull_request!(
        &keys,
        &relay_urls,
        d_tag,
        commit_id,
        clone_url,
        title,
        Some(&dummy_build_manifest_id)
    );
    println!("Pull request with title and build_manifest_event_id published.");
}

#[cfg(not(feature = "nostr"))]
fn main() {
    println!("This example requires the 'nostr' feature. Please run with: cargo run --example publish_pull_request --features nostr");
}
