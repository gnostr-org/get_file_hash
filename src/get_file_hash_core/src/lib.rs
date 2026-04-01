use std::process::Command;
use std::path::PathBuf;
#[cfg(feature = "nostr")]
use nostr_sdk::prelude::{*, EventBuilder, Tag, Kind};
#[cfg(feature = "nostr")]
use serde_json::json;
#[cfg(feature = "nostr")]
use csv::ReaderBuilder;
#[cfg(feature = "nostr")]
use ::url::Url;

#[cfg(feature = "nostr")]
const ONLINE_RELAYS_GPS_CSV: &[u8] = include_bytes!("online_relays_gps.csv");

#[cfg(feature = "nostr")]
pub fn get_relay_urls() -> Vec<String> {
    let content = String::from_utf8_lossy(ONLINE_RELAYS_GPS_CSV);
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(content.as_bytes());

    rdr.records()
        .filter_map(|result| {
            match result {
                Ok(record) => {
                    record.get(0).and_then(|url_str| {
                        let full_url_str = if url_str.contains("://") {
                            url_str.to_string()
                        } else {
                            format!("wss://{}", url_str)
                        };
                        match Url::parse(&full_url_str) {
                            Ok(url) if url.scheme() == "wss" => Some(url.to_string()),
                            _ => {
                                eprintln!("Warning: Invalid or unsupported relay URL scheme: {}", full_url_str);
                                None
                            }
                        }
                    })
                },
                Err(e) => {
                    eprintln!("Error reading CSV record: {}", e);
                    None
                }
            }
        })
        .collect()
}

/// Computes the SHA-256 hash of the specified file at compile time.
///
/// This macro takes a string literal representing a file path, reads the file's bytes
/// at compile time, computes its SHA-256 hash, and returns the hash as a hex-encoded `String`.
///
/// # Examples
///
/// ```rust
/// use get_file_hash_core::get_file_hash;
/// use sha2::{Digest, Sha256};
///
/// let hash = get_file_hash!("lib.rs");
/// println!("Hash: {}", hash);
/// ```

#[macro_export]
macro_rules! get_file_hash {
    ($file_path:expr) => {{
        let bytes = include_bytes!($file_path);
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let result = hasher.finalize();

        // Convert the GenericArray to a hex string
        result
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    }};
}

/// Computes the SHA-256 hash of the specified file at compile time and uses it as a Nostr private key.
///
/// This macro takes a string literal representing a file path, computes its SHA-256 hash,
/// and returns a `nostr::Keys` object derived from this hash.
///
/// # Examples
///
/// ```rust
/// use get_file_hash_core::file_hash_as_nostr_private_key;
/// use sha2::{Digest, Sha256};
/// use nostr_sdk::prelude::ToBech32;
///
/// let keys = file_hash_as_nostr_private_key!("lib.rs");
/// println!("Public Key: {}", keys.public_key().to_bech32().unwrap());
/// ```
#[cfg(feature = "nostr")]
#[macro_export]
macro_rules! file_hash_as_nostr_private_key {
    ($file_path:expr) => {{
        let hash_hex = $crate::get_file_hash!($file_path);
        nostr_sdk::Keys::parse(&hash_hex).expect("Failed to create Nostr Keys from file hash")
    }};
}

/// Publishes a NIP-34 repository announcement event to Nostr relays.
///
/// This macro takes Nostr keys, relay URLs, project details, a clone URL, and a file path.
/// It computes the SHA-256 hash of the file at compile time to use as the "earliest unique commit" (EUC),
/// and then publishes a Kind 30617 event.
///
/// # Examples
///
/// ```no_run
/// use get_file_hash_core::repository_announcement;
/// use get_file_hash_core::get_file_hash;
/// use nostr_sdk::Keys;
/// use sha2::{Digest, Sha256};
///
/// #[tokio::main]
/// async fn main() {
///     let keys = Keys::generate();
///     let relay_urls = vec!["wss://relay.damus.io".to_string()];
///     let project_name = "my-awesome-repo";
///     let description = "A fantastic new project.";
///     let clone_url = "git@github.com:user/my-awesome-repo.git";
///
///     repository_announcement!(
///         &keys,
///         &relay_urls,
///         project_name,
///         description,
///         clone_url,
///         "../Cargo.toml", // Use a known file in your project
///         None
///     );
/// }
#[cfg(feature = "nostr")]
#[macro_export]
macro_rules! repository_announcement {
    ($keys:expr, $relay_urls:expr, $project_name:expr, $description:expr, $clone_url:expr, $file_for_euc:expr) => {{
        let euc_hash = $crate::get_file_hash!($file_for_euc);
        // The 'd' tag value should be unique for the repository. Using the project_name for simplicity.
        let d_tag_value = $project_name;
        $crate::publish_repository_announcement_event(
            $keys,
            $relay_urls,
            $project_name,
            $description,
            $clone_url,
            &euc_hash,
            d_tag_value,
            None,
        ).await;
    }};
    ($keys:expr, $relay_urls:expr, $project_name:expr, $description:expr, $clone_url:expr, $file_for_euc:expr, $build_manifest_event_id:expr) => {{
        let euc_hash = $crate::get_file_hash!($file_for_euc);
        let d_tag_value = $project_name;
        $crate::publish_repository_announcement_event(
            $keys,
            $relay_urls,
            $project_name,
            $description,
            $clone_url,
            &euc_hash,
            d_tag_value,
            $build_manifest_event_id, // Pass directly, macro arg should be Option<&EventId>
        ).await;
    }};
}

/// Publishes a NIP-34 patch event to Nostr relays.
///
/// This macro takes Nostr keys, relay URLs, the repository's d-tag value,
/// the commit ID the patch applies to, and the path to the patch file.
/// The content of the patch file is included directly in the event.
///
/// # Examples
///
/// ```no_run
/// use get_file_hash_core::publish_patch;
/// use nostr_sdk::Keys;
///
/// #[tokio::main]
/// async fn main() {
///     let keys = Keys::generate();
///     let relay_urls = vec!["wss://relay.damus.io".to_string()];
///     let d_tag = "my-awesome-repo";
///     let commit_id = "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0"; // Example commit ID
///
///     publish_patch!(
///         &keys,
///         &relay_urls,
///         d_tag,
///         commit_id,
///         "lib.rs" // Use an existing file for the patch content
///     );
/// }
/// ```
#[cfg(feature = "nostr")]
#[macro_export]
macro_rules! publish_patch {
    ($keys:expr, $relay_urls:expr, $d_tag_value:expr, $commit_id:expr, $patch_file_path:expr) => {{
        let patch_content = include_str!($patch_file_path);
        $crate::publish_patch_event(
            $keys,
            $relay_urls,
            $d_tag_value,
            $commit_id,
            patch_content,
            None, // Pass None for build_manifest_event_id
        ).await;
    }};
    ($keys:expr, $relay_urls:expr, $d_tag_value:expr, $commit_id:expr, $patch_file_path:expr, $build_manifest_event_id:expr) => {{
        let patch_content = include_str!($patch_file_path);
        $crate::publish_patch_event(
            $keys,
            $relay_urls,
            $d_tag_value,
            $commit_id,
            patch_content,
            $build_manifest_event_id, // Pass directly, macro arg should be Option<&EventId>
        ).await;
    }};
}

/// Publishes a NIP-34 pull request event to Nostr relays.
///
/// This macro takes Nostr keys, relay URLs, the repository's d-tag value,
/// the commit ID of the pull request, a clone URL where the work can be fetched,
/// and an optional title for the pull request.
///
/// # Examples
///
/// ```no_run
/// use get_file_hash_core::publish_pull_request;
/// use nostr_sdk::Keys;
///
/// #[tokio::main]
/// async fn main() {
///     let keys = Keys::generate();
///     let relay_urls = vec!["wss://relay.damus.io".to_string()];
///     let d_tag = "my-awesome-repo";
///     let commit_id = "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0";
///     let clone_url = "git@github.com:user/my-feature-branch.git";
///     let title = Some("Feat: Add new awesome feature");
///
///     publish_pull_request!(
///         &keys,
///         &relay_urls,
///         d_tag,
///         commit_id,
///         clone_url,
///         title.unwrap()
///     );
/// }
/// ```
#[cfg(feature = "nostr")]
#[macro_export]
macro_rules! publish_pull_request {
    ($keys:expr, $relay_urls:expr, $d_tag_value:expr, $commit_id:expr, $clone_url:expr) => {{
        $crate::publish_pull_request_event(
            $keys,
            $relay_urls,
            $d_tag_value,
            $commit_id,
            $clone_url,
            None,
        ).await;
    }};
    ($keys:expr, $relay_urls:expr, $d_tag_value:expr, $commit_id:expr, $clone_url:expr, $title:expr) => {{
        $crate::publish_pull_request_event(
            $keys,
            $relay_urls,
            $d_tag_value,
            $commit_id,
            $clone_url,
            Some($title),
        ).await;
    }};
}

/// Publishes a NIP-34 PR update event to Nostr relays.
///
/// This macro takes Nostr keys, relay URLs, the repository's d-tag value,
/// the event ID of the original pull request, the new commit ID,
/// and the new clone URL.
///
/// # Examples
///
/// ```no_run
/// use get_file_hash_core::publish_pr_update;
/// use nostr_sdk::Keys;
/// use nostr_sdk::EventId;
/// use std::str::FromStr;
///
/// #[tokio::main]
/// async fn main() {
///     let keys = Keys::generate();
///     let relay_urls = vec!["wss://relay.damus.io".to_string()];
///     let d_tag = "my-awesome-repo";
///     let pr_event_id = EventId::from_str("f6e4d6a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9").unwrap(); // Example PR Event ID
///     let updated_commit_id = "z9y8x7w6v5u4t3s2r1q0p9o8n7m6l5k4j3i2h1g0";
///     let updated_clone_url = "git@github.com:user/my-feature-branch-v2.git";
///
///     publish_pr_update!(
///         &keys,
///         &relay_urls,
///         d_tag,
///         &pr_event_id,
///         updated_commit_id,
///         updated_clone_url
///     );
/// }
/// ```
#[cfg(feature = "nostr")]
#[macro_export]
macro_rules! publish_pr_update {
    ($keys:expr, $relay_urls:expr, $d_tag_value:expr, $pr_event_id:expr, $updated_commit_id:expr, $updated_clone_url:expr) => {{
        $crate::publish_pr_update_event(
            $keys,
            $relay_urls,
            $d_tag_value,
            $pr_event_id,
            $updated_commit_id,
            $updated_clone_url,
        ).await;
    }};
}

/// Publishes a NIP-34 repository state event to Nostr relays.
///
/// This macro takes Nostr keys, relay URLs, the repository's d-tag value,
/// the branch name, and the commit ID for that branch.
///
/// # Examples
///
/// ```no_run
/// use get_file_hash_core::publish_repository_state;
/// use nostr_sdk::Keys;
///
/// #[tokio::main]
/// async fn main() {
///     let keys = Keys::generate();
///     let relay_urls = vec!["wss://relay.damus.io".to_string()];
///     let d_tag = "my-awesome-repo";
///     let branch_name = "main";
///     let commit_id = "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0";
///
///     publish_repository_state!(
///         &keys,
///         &relay_urls,
///         d_tag,
///         branch_name,
///         commit_id
///     );
/// }
/// ```
#[cfg(feature = "nostr")]
#[macro_export]
macro_rules! publish_repository_state {
    ($keys:expr, $relay_urls:expr, $d_tag_value:expr, $branch_name:expr, $commit_id:expr) => {{
        $crate::publish_repository_state_event(
            $keys,
            $relay_urls,
            $d_tag_value,
            $branch_name,
            $commit_id,
        ).await;
    }};
}

/// Publishes a NIP-34 issue event to Nostr relays.
///
/// This macro takes Nostr keys, relay URLs, the repository's d-tag value,
/// a unique issue ID, the issue's title, and its content (markdown).
///
/// # Examples
///
/// ```no_run
/// use get_file_hash_core::publish_issue;
/// use nostr_sdk::Keys;
///
/// #[tokio::main]
/// async fn main() {
///     let keys = Keys::generate();
///     let relay_urls = vec!["wss://relay.damus.io".to_string()];
///     let d_tag = "my-awesome-repo";
///     let issue_id = "123";
///     let title = "Bug: Fix authentication flow";
///     let content = "The authentication flow is currently broken when users try to log in with invalid credentials. It crashes instead of showing an error message.";
///
///     publish_issue!(
///         &keys,
///         &relay_urls,
///         d_tag,
///         issue_id,
///         title,
///         content
///     );
/// }
/// ```
/// ```
#[cfg(feature = "nostr")]
#[macro_export]
macro_rules! publish_issue {
    ($keys:expr, $relay_urls:expr, $d_tag_value:expr, $issue_id:expr, $title:expr, $content:expr) => {{
        $crate::publish_issue_event(
            $keys,
            $relay_urls,
            $d_tag_value,
            $issue_id,
            $title,
            $content,
        ).await;
    }};
}

pub fn get_git_tracked_files(dir: &PathBuf) -> Vec<String> {
    String::from_utf8_lossy(
        &Command::new("git")
            .arg("ls-files")
            .current_dir(dir)
            .output()
            .expect("Failed to execute git ls-files")
            .stdout
    )
    .lines()
    .filter_map(|line| Some(String::from(line)))
    .collect()
}

#[cfg(feature = "nostr")]
pub async fn publish_metadata_event(
    keys: &Keys,
    relay_urls: &[String],
    picture_url: &str,
    banner_url: &str,
    file_path_str: &str,
) {
    let client = nostr_sdk::Client::new(keys.clone());

    for relay_url in relay_urls {
        if let Err(e) = client.add_relay(relay_url).await {
            println!("cargo:warning=Failed to add relay for metadata {}: {}", relay_url, e);
        }
    }
    client.connect().await;

    let metadata_json = json!({
        "picture": picture_url,
        "banner": banner_url,
        "name": file_path_str,
        "about": format!("Metadata for file event: {}", file_path_str),
    });

    let metadata = serde_json::from_str::<nostr_sdk::Metadata>(&metadata_json.to_string())
        .expect("Failed to parse metadata JSON");

    match client.send_event_builder(EventBuilder::metadata(&metadata)).await {
        Ok(event_id) => {
            println!("cargo:warning=Published Nostr metadata event for {}: {:?}", file_path_str, event_id);
        }
        Err(e) => {
            println!("cargo:warning=Failed to publish Nostr metadata event for {}: {}", file_path_str, e);
        }
    }
}

#[cfg(feature = "nostr")]
pub async fn publish_repository_announcement_event(
    keys: &Keys,
    relay_urls: &[String],
    project_name: &str,
    description: &str,
    clone_url: &str,
    euc: &str, // Earliest Unique Commit hash
    d_tag_value: &str, // d-tag value
    build_manifest_event_id: Option<&EventId>,
) {
    let client = nostr_sdk::Client::new(keys.clone());

    for relay_url in relay_urls {
        if let Err(e) = client.add_relay(relay_url).await {
            println!("cargo:warning=Failed to add relay for repository announcement {}: {}", relay_url, e);
        }
    }
    client.connect().await;

    let mut tags = vec![
        Tag::parse(["name", project_name]).expect("Failed to create name tag"),
        Tag::parse(["description", description]).expect("Failed to create description tag"),
        Tag::parse(["clone", clone_url]).expect("Failed to create clone tag"),
        Tag::custom("euc".into(), vec![euc.to_string()]),
        Tag::custom("d".into(), vec![d_tag_value.to_string()]), // NIP-33 d-tag
    ];

    if let Some(event_id) = build_manifest_event_id {
        tags.push(Tag::event(*event_id));
    }

    let event_builder = EventBuilder::new(
        Kind::Custom(30617), // NIP-34 Repository Announcement kind
        "", // Content is empty for repository announcement
    ).tags(tags);

    match client.send_event_builder(event_builder).await {
        Ok(event_id) => {
            println!("cargo:warning=Published NIP-34 Repository Announcement for {}. Event ID (raw): {:?}, Event ID (bech32): {}", project_name, event_id, event_id.to_bech32().unwrap());
        }
        Err(e) => {
            println!("cargo:warning=Failed to publish NIP-34 Repository Announcement for {}: {}", project_name, e);
        }
    }
}

#[cfg(feature = "nostr")]
pub async fn publish_patch_event(
    keys: &Keys,
    relay_urls: &[String],
    d_tag_value: &str,
    commit_id: &str,
    patch_content: &str,
    build_manifest_event_id: Option<&EventId>,
) {
    let client = nostr_sdk::Client::new(keys.clone());

    for relay_url in relay_urls {
        if let Err(e) = client.add_relay(relay_url).await {
            println!("cargo:warning=Failed to add relay for patch {}: {}", relay_url, e);
        }
    }
    client.connect().await;

    let mut tags = vec![
        Tag::custom("d".into(), vec![d_tag_value.to_string()]), // Repository d-tag
        Tag::parse(["commit", commit_id]).expect("Failed to create commit tag"),
    ];

    if let Some(event_id) = build_manifest_event_id {
        tags.push(Tag::event(*event_id));
    }

    let event_builder = EventBuilder::new(
        Kind::Custom(1617), // NIP-34 Patch kind
        patch_content,
    ).tags(tags);

    match client.send_event_builder(event_builder).await {
        Ok(event_id) => {
            println!("cargo:warning=Published NIP-34 Patch event for commit {}. Event ID (raw): {:?}, Event ID (bech32): {}", commit_id, event_id, event_id.to_bech32().unwrap());
        }
        Err(e) => {
            println!("cargo:warning=Failed to publish NIP-34 Patch event for commit {}: {}", commit_id, e);
        }
    }
}

#[cfg(feature = "nostr")]
pub async fn publish_pull_request_event(
    keys: &Keys,
    relay_urls: &[String],
    d_tag_value: &str,
    commit_id: &str,
    clone_url: &str,
    title: Option<&str>,
) {
    let client = nostr_sdk::Client::new(keys.clone());

    for relay_url in relay_urls {
        if let Err(e) = client.add_relay(relay_url).await {
            println!("cargo:warning=Failed to add relay for pull request {}: {}", relay_url, e);
        }
    }
    client.connect().await;

    let mut tags = vec![
        Tag::custom("d".into(), vec![d_tag_value.to_string()]), // Repository d-tag
        Tag::parse(["commit", commit_id]).expect("Failed to create commit tag"),
        Tag::parse(["clone", clone_url]).expect("Failed to create clone tag"),
    ];

    if let Some(t) = title {
        tags.push(Tag::parse(["title", t]).expect("Failed to create title tag"));
    }

    let event_builder = EventBuilder::new(
        Kind::Custom(1618), // NIP-34 Pull Request kind
        "", // Content can be empty or a description for the PR
    ).tags(tags);

    match client.send_event_builder(event_builder).await {
        Ok(event_id) => {
            println!("cargo:warning=Published NIP-34 Pull Request event for commit {}. Event ID (raw): {:?}, Event ID (bech32): {}", commit_id, event_id, event_id.to_bech32().unwrap());
        }
        Err(e) => {
            println!("cargo:warning=Failed to publish NIP-34 Pull Request event for commit {}: {}", commit_id, e);
        }
    }
}

#[cfg(feature = "nostr")]
pub async fn publish_pr_update_event(
    keys: &Keys,
    relay_urls: &[String],
    d_tag_value: &str,
    pr_event_id: &EventId,
    updated_commit_id: &str,
    updated_clone_url: &str,
) {
    let client = nostr_sdk::Client::new(keys.clone());

    for relay_url in relay_urls {
        if let Err(e) = client.add_relay(relay_url).await {
            println!("cargo:warning=Failed to add relay for PR update {}: {}", relay_url, e);
        }
    }
    client.connect().await;

    let event_builder = EventBuilder::new(
        Kind::Custom(1619), // NIP-34 PR Update kind
        "", // Content is empty for PR update
    ).tags(vec![
        Tag::custom("d".into(), vec![d_tag_value.to_string()]), // Repository d-tag
        Tag::parse(["p", pr_event_id.to_string().as_str()]).expect("Failed to create PR event ID tag"),
        Tag::parse(["commit", updated_commit_id]).expect("Failed to create updated commit ID tag"),
        Tag::parse(["clone", updated_clone_url]).expect("Failed to create updated clone URL tag"),
    ]);

    match client.send_event_builder(event_builder).await {
        Ok(event_id) => {
            println!("cargo:warning=Published NIP-34 PR Update event for PR {} (raw: {:?}). Event ID (raw): {:?}, Event ID (bech32): {}", pr_event_id.to_bech32().unwrap(), pr_event_id, event_id, event_id.to_bech32().unwrap());
        }
        Err(e) => {
            println!("cargo:warning=Failed to publish NIP-34 PR Update event for PR {}: {}", pr_event_id.to_string(), e);
        }
    }
}

#[cfg(feature = "nostr")]
pub async fn publish_repository_state_event(
    keys: &Keys,
    relay_urls: &[String],
    d_tag_value: &str,
    branch_name: &str,
    commit_id: &str,
) {
    let client = nostr_sdk::Client::new(keys.clone());

    for relay_url in relay_urls {
        if let Err(e) = client.add_relay(relay_url).await {
            println!("cargo:warning=Failed to add relay for repository state {}: {}", relay_url, e);
        }
    }
    client.connect().await;

    let event_builder = EventBuilder::new(
        Kind::Custom(30618), // NIP-34 Repository State kind
        "", // Content is empty for repository state
    ).tags(vec![
        Tag::custom("d".into(), vec![d_tag_value.to_string()]), // Repository d-tag
        Tag::parse(["name", branch_name]).expect("Failed to create branch name tag"),
        Tag::parse(["commit", commit_id]).expect("Failed to create commit ID tag"),
    ]);

    match client.send_event_builder(event_builder).await {
        Ok(event_id) => {
            println!("cargo:warning=Published NIP-34 Repository State event for branch {} (commit {}). Event ID (raw): {:?}, Event ID (bech32): {}", branch_name, commit_id, event_id, event_id.to_bech32().unwrap());
        }
        Err(e) => {
            println!("cargo:warning=Failed to publish NIP-34 Repository State event for branch {} (commit {}): {}", branch_name, commit_id, e);
        }
    }
}

#[cfg(feature = "nostr")]
pub async fn publish_issue_event(
    keys: &Keys,
    relay_urls: &[String],
    d_tag_value: &str,
    issue_id: &str, // Unique identifier for the issue
    title: &str,
    content: &str,
) {
    let client = nostr_sdk::Client::new(keys.clone());

    for relay_url in relay_urls {
        if let Err(e) = client.add_relay(relay_url).await {
            println!("cargo:warning=Failed to add relay for issue {}: {}", relay_url, e);
        }
    }
    client.connect().await;

    let event_builder = EventBuilder::new(
        Kind::Custom(1621), // NIP-34 Issue kind
        content,
    ).tags(vec![
        Tag::custom("d".into(), vec![d_tag_value.to_string()]), // Repository d-tag
        Tag::parse(["i", issue_id]).expect("Failed to create issue ID tag"),
        Tag::parse(["title", title]).expect("Failed to create title tag"),
    ]);

    match client.send_event_builder(event_builder).await {
        Ok(event_id) => {
            println!("cargo:warning=Published NIP-34 Issue event for issue {} ({}). Event ID (raw): {:?}, Event ID (bech32): {}", issue_id, title, event_id, event_id.to_bech32().unwrap());
        }
        Err(e) => {
            println!("cargo:warning=Failed to publish NIP-34 Issue event for issue {} ({}): {}", issue_id, title, e);
        }
    }
}


#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use sha2::{Digest, Sha256};
    use tempfile;
    use super::get_git_tracked_files;
    use std::process::Command;
    use nostr_sdk::EventId;
    use std::str::FromStr;

    // Dummy EventId for tests that require a build_manifest_event_id
    const DUMMY_BUILD_MANIFEST_ID_STR: &str = "f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0";


    // Test for get_file_hash! macro
    #[test]
    fn test_get_file_hash() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test_file.txt");
        let content = "Hello, world!";
        File::create(&file_path).unwrap().write_all(content.as_bytes()).unwrap();

        // The macro expects a string literal, so we need to construct the path at compile time.
        // This is a limitation for testing, normally you'd use it with a known file.
        // For testing, we'll manually verify a file known to be in the project.
        // Let's test `lib.rs` itself for a more realistic scenario.
        let macro_hash = get_file_hash!("lib.rs");

        // We will assert on a known file within the crate.
        let bytes = include_bytes!("lib.rs");
        let mut hasher_manual = Sha256::new();
        hasher_manual.update(bytes);
        let expected_hash_lib_rs = hasher_manual.finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        assert_eq!(macro_hash, expected_hash_lib_rs);

        // Test with another known file, e.g., Cargo.toml of the core crate
        let cargo_toml_hash = get_file_hash!("../Cargo.toml");
        let cargo_toml_bytes = include_bytes!("../Cargo.toml");
        let mut cargo_toml_hasher = Sha256::new();
        cargo_toml_hasher.update(cargo_toml_bytes);
        let expected_cargo_toml_hash = cargo_toml_hasher.finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        assert_eq!(cargo_toml_hash, expected_cargo_toml_hash);
    }

    #[test]
    fn test_get_git_tracked_files() {
        let dir = tempfile::tempdir().unwrap();
        let repo_path = dir.path();

        // Initialize a git repository
        let _ = Command::new("git")
            .arg("init")
            .current_dir(repo_path)
            .output()
            .expect("Failed to initialize git repo");

        // Create some files
        let file1_path = repo_path.join("file1.txt");
        File::create(&file1_path).unwrap().write_all(b"content1").unwrap();
        let file2_path = repo_path.join("file2.txt");
        File::create(&file2_path).unwrap().write_all(b"content2").unwrap();

        // Add and commit files
        let _ = Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(repo_path)
            .output()
            .expect("Failed to git add files");
        let _ = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("Initial commit")
            .current_dir(repo_path)
            .output()
            .expect("Failed to git commit");

        let tracked_files = get_git_tracked_files(&repo_path.to_path_buf());
        assert_eq!(tracked_files.len(), 2);
        assert!(tracked_files.contains(&"file1.txt".to_string()));
        assert!(tracked_files.contains(&"file2.txt".to_string()));
    }

    // #[cfg(feature = "nostr")]
    // #[test]
    // fn test_file_hash_as_nostr_private_key() {
    //     use super::file_hash_as_nostr_private_key;
    //     // use std::fs::{File, remove_file};
    //     // use std::io::Write;
    //     // use tempfile::tempdir; // Not needed as we're using a literal path
    //     use nostr_sdk::prelude::ToBech32;

    //     let file_path = PathBuf::from("test_nostr_file_for_macro.txt");
    //     let content = "Nostr test content!";
    //     File::create(&file_path).unwrap().write_all(content.as_bytes()).unwrap();

    //     let keys = file_hash_as_nostr_private_key!("test_nostr_file_for_macro.txt");

    //     assert!(!keys.public_key().to_bech32().unwrap().is_empty());

    //     remove_file(&file_path).unwrap();
    // }

    #[cfg(feature = "nostr")]
    #[tokio::test]
    async fn test_publish_metadata_event() {
        use super::publish_metadata_event;
        use nostr_sdk::Keys;

        let keys = Keys::generate();
        let picture_url = "https://avatars.githubusercontent.com/u/135379339?s=400&u=11cb72cccbc2b13252867099546074c50caef1ae&v=4";
        let banner_url = "https://raw.githubusercontent.com/gnostr-org/gnostr-icons/refs/heads/master/banner/1024x341.png";
        let file_path_str = "test_file.txt";

        // This test primarily checks that the function doesn't panic
        // and goes through its execution path.
        // Actual publishing success depends on external network conditions.
        let relay_urls = super::get_relay_urls();
        publish_metadata_event(
            &keys,
            &relay_urls,
            picture_url,
            banner_url,
            file_path_str,
        ).await;
    }

    #[cfg(feature = "nostr")]
    #[tokio::test]
    async fn test_repository_announcement_event() {
        use super::get_relay_urls;
        use nostr_sdk::{Keys, EventId};
        use std::str::FromStr;

        let keys = Keys::generate();
        let relay_urls = get_relay_urls();
        let project_name = "test-nip34-repo";
        let description = "A test repository for NIP-34 announcements.";
        let clone_url = "git@example.com:test/test-nip34-repo.git";
        let dummy_build_manifest_id = EventId::from_str(DUMMY_BUILD_MANIFEST_ID_STR).unwrap();
        let _file_for_euc = "Cargo.toml"; // Use a known file in the project, as required by include_bytes!

        // This test primarily checks that the macro and function compile and execute without panicking.
        // Actual publishing success depends on external network conditions.
        super::publish_metadata_event(
            &keys,
            &relay_urls,
            "https://example.com/test_repo_announcement_picture.jpg",
            "https://example.com/test_repo_announcement_banner.jpg",
            "test_repository_announcement_event_metadata",
        ).await;

        let dummy_build_manifest_id = EventId::from_str(DUMMY_BUILD_MANIFEST_ID_STR).unwrap();

        repository_announcement!(
            &keys,
            &relay_urls,
            project_name,
            description,
            clone_url,
            "../Cargo.toml", // Pass the string literal directly, correcting path for include_bytes!
            Some(&dummy_build_manifest_id)
        );
    }

    #[cfg(feature = "nostr")]
    #[tokio::test]
    async fn test_publish_patch_event() {
        use super::get_relay_urls;
        use nostr_sdk::Keys;

        let keys = Keys::generate();
        let relay_urls = get_relay_urls();
        let d_tag = "test-repo-for-patch";
        let commit_id = "fedcba9876543210fedcba9876543210fedcba";

        // This test primarily checks that the macro and function compile and execute without panicking.
        // Actual publishing success depends on external network conditions.
        super::publish_metadata_event(
            &keys,
            &relay_urls,
            "https://example.com/test_patch_picture.jpg",
            "https://example.com/test_patch_banner.jpg",
            "test_publish_patch_event_metadata",
        ).await;

        let dummy_build_manifest_id = EventId::from_str(DUMMY_BUILD_MANIFEST_ID_STR).unwrap();
        publish_patch!(
            &keys,
            &relay_urls,
            d_tag,
            commit_id,
            "lib.rs", // Use an existing file for the patch content
            Some(&dummy_build_manifest_id)
        );    }

    #[cfg(feature = "nostr")]
    #[tokio::test]
    async fn test_publish_pull_request_event() {
        use super::get_relay_urls;
        use nostr_sdk::Keys;

        let keys = Keys::generate();
        let relay_urls = get_relay_urls();
        let d_tag = "test-repo-for-pr";
        let commit_id = "0123456789abcdef0123456789abcdef01234567";
        let clone_url = "git@example.com:test/pr-branch.git";
        let title = Some("Feat: Implement NIP-34 PR");

        super::publish_metadata_event(
            &keys,
            &relay_urls,
            "https://example.com/test_pr_picture.jpg",
            "https://example.com/test_pr_banner.jpg",
            "test_publish_pull_request_event_metadata",
        ).await;

        // Test with a title
        publish_pull_request!(
            &keys,
            &relay_urls,
            d_tag,
            commit_id,
            clone_url,
            title.unwrap()
        );
        // Test without a title
        publish_pull_request!(
            &keys,
            &relay_urls,
            d_tag,
            commit_id,
            clone_url
        );
    }

    #[cfg(feature = "nostr")]
    #[tokio::test]
    async fn test_publish_pr_update_event() {
        use super::get_relay_urls;
        use nostr_sdk::{Keys, EventId};
        use std::str::FromStr;

        let keys = Keys::generate();
        let relay_urls = get_relay_urls();
        let d_tag = "test-repo-for-pr-update";
        let pr_event_id = EventId::from_str("f6e4d6a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9").unwrap(); // Placeholder EventId
        let updated_commit_id = "z9y8x7w6v5u4t3s2r1q0p9o8n7m6l5k4j3i2h1g0";
        let updated_clone_url = "git@example.com:test/pr-branch-updated.git";

        // This test primarily checks that the macro and function compile and execute without panicking.
        // Actual publishing success depends on external network conditions.
        super::publish_metadata_event(
            &keys,
            &relay_urls,
            "https://example.com/test_pr_update_picture.jpg",
            "https://example.com/test_pr_update_banner.jpg",
            "test_publish_pr_update_event_metadata",
        ).await;

        publish_pr_update!(
            &keys,
            &relay_urls,
            d_tag,
            &pr_event_id, // Pass a reference to pr_event_id
            updated_commit_id,
            updated_clone_url
        );    }

    #[cfg(feature = "nostr")]
    #[tokio::test]
    async fn test_publish_repository_state_event() {
        use super::get_relay_urls;
        use nostr_sdk::Keys;

        let keys = Keys::generate();
        let relay_urls = get_relay_urls();
        let d_tag = "test-repo-for-state";
        let branch_name = "main";
        let commit_id = "abcde12345abcde12345abcde12345abcde12345";

        // This test primarily checks that the macro and function compile and execute without panicking.
        // Actual publishing success depends on external network conditions.
        super::publish_metadata_event(
            &keys,
            &relay_urls,
            "https://example.com/test_repo_state_picture.jpg",
            "https://example.com/test_repo_state_banner.jpg",
            "test_publish_repository_state_event_metadata",
        ).await;

        publish_repository_state!(
            &keys,
            &relay_urls,
            d_tag,
            branch_name,
            commit_id
        );    }

    #[cfg(feature = "nostr")]
    #[tokio::test]
    async fn test_publish_issue_event() {
        use super::get_relay_urls;
        use nostr_sdk::Keys;

        let keys = Keys::generate();
        let relay_urls = get_relay_urls();
        let d_tag = "test-repo-for-issue";
        let issue_id = "456";
        let title = "Feature: Implement NIP-34 Issues";
        let content = "This is a test issue to verify the NIP-34 issue macro implementation.";

        // This test primarily checks that the macro and function compile and execute without panicking.
        // Actual publishing success depends on external network conditions.
        super::publish_metadata_event(
            &keys,
            &relay_urls,
            "https://example.com/test_issue_picture.jpg",
            "https://example.com/test_issue_banner.jpg",
            "test_publish_issue_event_metadata",
        ).await;

        publish_issue!(
            &keys,
            &relay_urls,
            d_tag,
            issue_id,
            title,
            content
        );    }
}
