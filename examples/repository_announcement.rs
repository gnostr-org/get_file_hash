#[tokio::main]
#[cfg(feature = "nostr")]
#[allow(unused_imports)]
async fn main() {
    use get_file_hash_core::repository_announcement;
    use get_file_hash_core::get_file_hash;
    use nostr_sdk::Keys;
    use sha2::{Digest, Sha256};
    use nostr_sdk::EventId;
    use std::str::FromStr;

    let keys = Keys::generate();
    let relay_urls = get_file_hash_core::get_relay_urls();
    let project_name = "my-awesome-repo-example";
    let description = "A fantastic new project example.";
    let clone_url = "git@github.com:user/my-awesome-repo-example.git";
    
    // Dummy EventId for examples that require a build_manifest_event_id
    const DUMMY_BUILD_MANIFEST_ID_STR: &str = "f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0";
    let dummy_build_manifest_id = EventId::from_str(DUMMY_BUILD_MANIFEST_ID_STR).unwrap();

    // Example 1: Without build_manifest_event_id
    println!("Publishing repository announcement without build_manifest_event_id...");
    repository_announcement!(
        &keys,
        &relay_urls,
        project_name,
        description,
        clone_url,
        "../Cargo.toml" // Use a known file in your project
    );
    println!("Repository announcement without build_manifest_event_id published.");

    // Example 2: With build_manifest_event_id
    println!("Publishing repository announcement with build_manifest_event_id...");
    repository_announcement!(
        &keys,
        &relay_urls,
        project_name,
        description,
        clone_url,
        "../Cargo.toml", // Use a known file in your project
        Some(&dummy_build_manifest_id)
    );
    println!("Repository announcement with build_manifest_event_id published.");
}

#[cfg(not(feature = "nostr"))]
fn main() {
    println!("This example requires the 'nostr' feature. Please run with: cargo run --example repository_announcement --features nostr");
}
