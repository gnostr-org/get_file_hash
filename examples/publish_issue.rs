#[tokio::main]
#[cfg(feature = "nostr")]
async fn main() {
    use get_file_hash_core::publish_issue;
    use nostr_sdk::Keys;
    use nostr_sdk::EventId;
    use std::str::FromStr;

    let keys = Keys::generate();
    let relay_urls = get_file_hash_core::get_relay_urls();
    let d_tag = "my-awesome-repo-example";
    let issue_id = "123";
    let title = "Bug: Fix authentication flow example";
    let content = "The authentication flow is currently broken when users try to log in with invalid credentials. It crashes instead of showing an error message.";

    // Dummy EventId for examples that require a build_manifest_event_id
    const DUMMY_BUILD_MANIFEST_ID_STR: &str = "f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0";
    let dummy_build_manifest_id = EventId::from_str(DUMMY_BUILD_MANIFEST_ID_STR).unwrap();

    // Example 1: Without build_manifest_event_id
    println!("Publishing issue without build_manifest_event_id...");
    publish_issue!(
        &keys,
        &relay_urls,
        d_tag,
        issue_id,
        title,
        content
    );
    println!("Issue without build_manifest_event_id published.");

    // Example 2: With build_manifest_event_id
    println!("Publishing issue with build_manifest_event_id...");
    publish_issue!(
        &keys,
        &relay_urls,
        d_tag,
        issue_id,
        title,
        content,
        Some(&dummy_build_manifest_id)
    );
    println!("Issue with build_manifest_event_id published.");
}

#[cfg(not(feature = "nostr"))]
fn main() {
    println!("This example requires the 'nostr' feature. Please run with: cargo run --example publish_issue --features nostr");
}
