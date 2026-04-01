#[tokio::main]
#[cfg(feature = "nostr")]
async fn main() {
    use get_file_hash_core::publish_repository_state;
    use nostr_sdk::Keys;

    let keys = Keys::generate();
    let relay_urls = get_file_hash_core::get_relay_urls();
    let d_tag = "my-awesome-repo-example";
    let branch_name = "main";
    let commit_id = "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0";

    println!("Publishing repository state...");
    publish_repository_state!(
        &keys,
        &relay_urls,
        d_tag,
        branch_name,
        commit_id
    );
    println!("Repository state published.");
}

#[cfg(not(feature = "nostr"))]
fn main() {
    println!("This example requires the 'nostr' feature. Please run with: cargo run --example publish_repository_state --features nostr");
}
