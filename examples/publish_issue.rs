#[cfg(feature = "nostr")]
use get_file_hash_core::{get_relay_urls, publish_issue, DEFAULT_GNOSTR_KEY, DEFAULT_PICTURE_URL, DEFAULT_BANNER_URL, publish_nostr_event_if_release, get_repo_announcement_event, publish_patch_event};
#[cfg(feature = "nostr")]
#[tokio::main]
async fn main() {
    use nostr_sdk::Keys;
    use nostr_sdk::EventId;
    use std::str::FromStr;

    let keys = Keys::generate();
    let relay_urls = get_relay_urls();
    let d_tag = "my-gnostr-repository-issue-example"; // Repository identifier
    let issue_id_1 = "issue-001"; // Unique identifier for the first issue
    let issue_id_2 = "issue-002"; // Unique identifier for the second issue
    let title_1 = "Bug: Application crashes on startup";
    let content_1 = "The application fails to launch on macOS Ventura. It throws a 'Segmentation Fault' error immediately after execution. This was observed on version `v1.2.3`.

Steps to reproduce:
1. Download `app-v1.2.3-macos.tar.gz`
2. Extract the archive
3. Run `./app`

Expected behavior: Application launches successfully.
Actual behavior: Application crashes with 'Segmentation Fault'.";

    let title_2 = "Feature Request: Dark Mode";
    let content_2 = "Users have requested a dark mode option to improve readability and reduce eye strain during prolonged use. This should be toggleable in the settings menu.

Considerations:
- Adherence to system dark mode settings.
- Consistent styling across all UI components.";

    // Dummy EventId for examples that require a build_manifest_event_id
    const DUMMY_BUILD_MANIFEST_ID_STR: &str = "f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0";
    let dummy_build_manifest_id = EventId::from_str(DUMMY_BUILD_MANIFEST_ID_STR).unwrap();

    // Example 1: Publish an issue without build_manifest_event_id
    println!("Publishing issue '{}' without build_manifest_event_id...", title_1);
    publish_issue!(
        &keys,
        &relay_urls,
        d_tag,
        issue_id_1,
        title_1,
        content_1
    );
    println!("Issue '{}' published.", title_1);

    // Example 2: Publish an issue with build_manifest_event_id
    println!("Publishing issue '{}' with build_manifest_event_id...", title_2);
    publish_issue!(
        &keys,
        &relay_urls,
        d_tag,
        issue_id_2,
        title_2,
        content_2,
        Some(&dummy_build_manifest_id)
    );
    println!("Issue '{}' published.", title_2);
}

#[cfg(not(feature = "nostr"))]
fn main() {
    println!("This example requires the 'nostr' feature. Please run with: cargo run --example publish_issue --features nostr");
}
