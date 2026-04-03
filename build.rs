/// deterministic nostr event build example
// deterministic nostr event build example
use get_file_hash_core::get_file_hash;
#[cfg(all(not(debug_assertions), feature = "nostr"))]
use get_file_hash_core::{get_git_tracked_files, DEFAULT_GNOSTR_KEY, DEFAULT_PICTURE_URL, DEFAULT_BANNER_URL};
#[cfg(all(not(debug_assertions), feature = "nostr"))]
use nostr_sdk::{EventBuilder, Keys, EventId, Tag, SecretKey, JsonUtil, Kind, Event};

#[cfg(all(not(debug_assertions), feature = "nostr"))]
use std::fs;
use std::path::PathBuf;
use sha2::{Digest, Sha256};
#[cfg(all(not(debug_assertions), feature = "nostr"))]
use ::hex;
#[cfg(all(not(debug_assertions), feature = "nostr"))]
use std::io::Write;

#[cfg(all(not(debug_assertions), feature = "nostr"))]
fn should_remove_relay(error_msg: &str) -> bool {
    error_msg.contains("relay not connected") ||
    error_msg.contains("not in web of trust") ||
    error_msg.contains("blocked: not authorized") ||
    error_msg.contains("timeout") ||
    error_msg.contains("blocked: spam not permitted") ||
    error_msg.contains("relay experienced an error trying to publish the latest event") ||
    error_msg.contains("duplicate: event already broadcast")
}

#[cfg(all(not(debug_assertions), feature = "nostr"))]
fn write_event_json_to_file(
    output_dir: &PathBuf,
    filename: &str,
    event: &Event,
) -> Option<()> {
    let file_path = output_dir.join(filename);
    if let Some(parent) = file_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            println!("cargo:warning=Failed to create parent directories for {}: {}", file_path.display(), e);
            return None;
        }
    }
    if let Err(e) = fs::File::create(&file_path).and_then(|mut file| write!(file, "{}", event.as_json())) {
        println!("cargo:warning=Failed to write event JSON to file {}: {}", file_path.display(), e);
        None
    } else {
        println!("cargo:warning=Successfully wrote event JSON to {}", file_path.display());
        Some(())
    }
}

#[cfg(all(not(debug_assertions), feature = "nostr"))]
async fn publish_nostr_event_if_release(
    hash: String,
    keys: Keys,
    event_builder: EventBuilder,
    mut relay_urls: Vec<String>,
    file_path_str: &str,
    output_dir: &PathBuf,
) -> Option<EventId> {
    let client = nostr_sdk::Client::new(keys.clone());
        let public_key = keys.public_key().to_string();

    for i in (0..relay_urls.len()).rev() {
        let relay_url = &relay_urls[i];
        if let Err(e) = client.add_relay(relay_url).await {
            println!("cargo:warning=Failed to add relay {}: {}", relay_url, e);
        }
    }
    println!("cargo:warning=Added {} relays", relay_urls.len());

    client.connect().await;
    println!("cargo:warning=Connected to {} relays", relay_urls.len());

    let event = client.sign_event_builder(event_builder).await.unwrap();

    match client.send_event(&event).await {        Ok(event_output) => {
            println!("cargo:warning=Published Nostr event for {}: {}", file_path_str, event_output.val);

            // Print successful relays
            for relay_url in event_output.success.iter() {
                println!("cargo:warning=Successfully published to relay: {}", relay_url);
            }
            // Print failed relays and remove "unfriendly" relays from the list
            let mut relays_to_remove: Vec<String> = Vec::new();
            for (relay_url, error_msg) in event_output.failed.iter() {
                if should_remove_relay(error_msg) {
                    relays_to_remove.push(relay_url.to_string());
                }
            }
            // Remove failed relays from the list
            relay_urls.retain(|url| !relays_to_remove.contains(url));
            if !relays_to_remove.is_empty() {
                println!("cargo:warning=Removed {} unresponsive relays from the list.", relays_to_remove.len());
            }

            let filename = format!("{}/{}/{}/{}.json", file_path_str, hash, public_key.clone(), event_output.val.to_string());
            write_event_json_to_file(output_dir, &filename, &event);
            Some(event_output.val)
        },
        Err(e) => {
            println!("cargo:warning=Failed to publish Nostr event for {}: {}", file_path_str, e);
            None
        },
    }
}

#[cfg(all(not(debug_assertions), feature = "nostr"))]
pub async fn get_repo_announcement_event(
    keys: &Keys,
    relay_urls: &Vec<String>,
    repo_url: &str,
    repo_name: &str,
    repo_description: &str,
    git_commit_hash: &str,
    git_branch: &str,
    output_dir: &PathBuf,
    public_key_hex: &str,
) -> Option<EventId> {
    let client = nostr_sdk::Client::new(keys.clone());

    for relay_url in relay_urls {
        if let Err(e) = client.add_relay(relay_url).await {
            println!("cargo:warning=Failed to add relay {}: {}", relay_url, e);
        }
    }
    client.connect().await;

    let tags = vec![
        Tag::parse(["r", repo_url].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["name", repo_name].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["description", repo_description].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["commit", git_commit_hash].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["branch", git_branch].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
    ];

    let event_builder = EventBuilder::new(Kind::Custom(30617), repo_description).tags(tags);
    let event = client.sign_event_builder(event_builder).await.unwrap();

    match client.send_event(&event).await {
        Ok(event_output) => {
            println!("cargo:warning=Published Nostr Repository Announcement for {}: {}", repo_name, event_output.val);
            
            let filename = format!("30617/{}/{}/{}.json", repo_name, public_key_hex, event_output.val.to_string());
            write_event_json_to_file(output_dir, &filename, &event);
            Some(event_output.val)
        },
        Err(e) => {
            println!("cargo:warning=Failed to publish Nostr Repository Announcement for {}: {}", repo_name, e);
            None
        },
    }
}

#[tokio::main]
async fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let is_git_repo = std::path::Path::new(&manifest_dir).join(".git").exists();


    println!("cargo:rustc-env=CARGO_PKG_NAME={}", env!("CARGO_PKG_NAME"));
    println!("cargo:rustc-env=CARGO_PKG_VERSION={}", env!("CARGO_PKG_VERSION"));

    if is_git_repo {
        let git_commit_hash_output = std::process::Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .expect("Failed to execute git command for commit hash");

        let git_commit_hash_str = if git_commit_hash_output.status.success() && !git_commit_hash_output.stdout.is_empty() {
            String::from_utf8(git_commit_hash_output.stdout).unwrap().trim().to_string()
        } else {
            println!("cargo:warning=Git commit hash command failed or returned empty. Status: {:?}, Stderr: {}", 
                     git_commit_hash_output.status, String::from_utf8_lossy(&git_commit_hash_output.stderr));
            String::new()
        };
        println!("cargo:rustc-env=GIT_COMMIT_HASH={}", git_commit_hash_str);

        let git_branch_output = std::process::Command::new("git")
            .args(&["rev-parse", "--abbrev-ref", "HEAD"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .expect("Failed to execute git command for branch name");

        let git_branch_str = if git_branch_output.status.success() && !git_branch_output.stdout.is_empty() {
            String::from_utf8(git_branch_output.stdout).unwrap().trim().to_string()
        } else {
            println!("cargo:warning=Git branch command failed or returned empty. Status: {:?}, Stderr: {}", 
                     git_branch_output.status, String::from_utf8_lossy(&git_branch_output.stderr));
            String::new()
        };
        println!("cargo:rustc-env=GIT_BRANCH={}", git_branch_str);
    } else {
        println!("cargo:rustc-env=GIT_COMMIT_HASH=");
        println!("cargo:rustc-env=GIT_BRANCH=");
    }

    println!("cargo:rerun-if-changed=.git/HEAD");

    #[cfg(all(not(debug_assertions), feature = "nostr"))]
    let relay_urls = get_file_hash_core::get_relay_urls();

    let cargo_toml_hash = get_file_hash!("Cargo.toml");
    println!("cargo:rustc-env=CARGO_TOML_HASH={}", cargo_toml_hash);

    let lib_hash = get_file_hash!("src/lib.rs");
    println!("cargo:rustc-env=LIB_HASH={}", lib_hash);

    let build_hash = get_file_hash!("build.rs");
    println!("cargo:rustc-env=BUILD_HASH={}", build_hash);

    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=build.rs");
    let online_relays_csv_path = PathBuf::from(&manifest_dir).join("src/get_file_hash_core/src/online_relays_gps.csv");
    if online_relays_csv_path.exists() {
        println!("cargo:rerun-if-changed={}", online_relays_csv_path.to_str().unwrap());
    }

#[cfg(all(not(debug_assertions), feature = "nostr"))]
    if cfg!(not(debug_assertions)) {
        println!("cargo:warning=Nostr feature enabled: Build may take longer due to network operations (publishing events to relays).");

        // This code only runs in release builds
        let package_version = std::env::var("CARGO_PKG_VERSION").unwrap();

        let output_dir = PathBuf::from(format!(".gnostr/build/{}", package_version));
        if let Err(e) = fs::create_dir_all(&output_dir) {
            println!("cargo:warning=Failed to create output directory {}: {}", output_dir.display(), e);
        }

        let files_to_publish: Vec<String> = get_git_tracked_files(&PathBuf::from(&manifest_dir));
        
        let mut published_event_ids: Vec<Tag> = Vec::new();

        for file_path_str in &files_to_publish {
            println!("cargo:warning=Processing file: {}", file_path_str);
            match fs::read(file_path_str) {
                Ok(bytes) => {
                    let mut hasher = Sha256::new();
                    hasher.update(&bytes);
                    let result = hasher.finalize();
                    let file_hash_hex = hex::encode(result);

                    match SecretKey::from_hex(&file_hash_hex.clone()) {
                        Ok(secret_key) => {
                            let keys = Keys::new(secret_key);
                            let content = String::from_utf8_lossy(&bytes).into_owned();
                            let tags = vec![
                                Tag::parse(["file", file_path_str].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
                                Tag::parse(["version", &package_version].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
                            ];
                            let event_builder = EventBuilder::text_note(content).tags(tags);

                            if let Some(event_id) = publish_nostr_event_if_release(file_hash_hex, keys.clone(), event_builder, relay_urls.clone(), file_path_str, &output_dir).await {
                                published_event_ids.push(Tag::event(event_id));
                            }

                            // Publish metadata event
                            get_file_hash_core::publish_metadata_event(
                                &keys,
                                &relay_urls,
                                DEFAULT_PICTURE_URL,
                                DEFAULT_BANNER_URL,
                                file_path_str,
                            ).await;
                        }
                        Err(e) => {
                            println!("cargo:warning=Failed to derive Nostr secret key for {}: {}", file_path_str, e);
                        }
                    }
                }
                Err(e) => {
                    println!("cargo:warning=Failed to read file {}: {}", file_path_str, e);
                }
            }
        }

        // Create and publish the linking event
        if !published_event_ids.is_empty() {

            //TODO this will be either the default or detected from env vars PRIVATE_KEY
            let keys = Keys::new(SecretKey::from_hex(DEFAULT_GNOSTR_KEY).expect("Failed to create Nostr keys from DEFAULT_GNOSTR_KEY"));
            let cloned_keys = keys.clone();
            let content = format!("Build manifest for get_file_hash v{}", package_version);
            let mut tags = vec![
                Tag::parse(["build_manifest", &package_version].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
                Tag::parse(["build_manifest", &package_version].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
                Tag::parse(["build_manifest", &package_version].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
                Tag::parse(["build_manifest", &package_version].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
            ];
            tags.extend(published_event_ids);

            let event_builder = EventBuilder::text_note(content.clone()).tags(tags);

            if let Some(event_id) = publish_nostr_event_if_release(
                hex::encode(Sha256::digest(content.as_bytes())),
                keys,
                event_builder,
                relay_urls.clone(),
                "build_manifest.json",
                &output_dir,
            ).await {

                let build_manifest_event_id = Some(event_id);

            // Publish metadata event for the build manifest
            get_file_hash_core::publish_metadata_event(
                &cloned_keys, // Use reference to cloned keys here
                &relay_urls,
                DEFAULT_PICTURE_URL,
                DEFAULT_BANNER_URL,
                &format!("build_manifest:{}", package_version),
            ).await;
            let git_commit_hash = std::env::var("GIT_COMMIT_HASH").unwrap_or_default();
            let git_branch = std::env::var("GIT_BRANCH").unwrap_or_default();
            let repo_url = std::env::var("CARGO_PKG_REPOSITORY").unwrap();
            let repo_name = std::env::var("CARGO_PKG_NAME").unwrap();
            let repo_description = std::env::var("CARGO_PKG_DESCRIPTION").unwrap();

            let output_dir = PathBuf::from(format!(".gnostr/build/{}", package_version));
            if let Err(e) = fs::create_dir_all(&output_dir) {
                println!("cargo:warning=Failed to create output directory {}: {}", output_dir.display(), e);
            }

            let announcement_keys = Keys::new(SecretKey::from_hex(build_manifest_event_id.unwrap().to_hex().as_str()).expect("Failed to create Nostr keys from build_manifest_event_id"));
            let announcement_pubkey_hex = announcement_keys.public_key().to_string();

            // Publish NIP-34 Repository Announcement
            if let Some(_event_id) = get_repo_announcement_event(
                &announcement_keys,
                &relay_urls,
                &repo_url,
                &repo_name,
                &repo_description,
                &git_commit_hash,
                &git_branch,
                &output_dir,
                &announcement_pubkey_hex
            ).await {
                // Successfully published announcement
            }
            }
        }
    }
}
// deterministic nostr event build example
