/// deterministic nostr event build example
// deterministic nostr event build example
use get_file_hash_core::get_file_hash;
#[cfg(all(not(debug_assertions), feature = "nostr"))]
use get_file_hash_core::{get_git_tracked_files, DEFAULT_GNOSTR_KEY, DEFAULT_PICTURE_URL, DEFAULT_BANNER_URL, should_remove_relay, write_event_json_to_file};
#[cfg(all(not(debug_assertions), feature = "nostr"))]
use nostr_sdk::{EventBuilder, Keys, EventId, Tag, SecretKey, JsonUtil, Kind, Event};
#[cfg(all(not(debug_assertions), feature = "nostr"))]
use serde_json::to_string;
#[cfg(all(not(debug_assertions), feature = "nostr"))]
use std::fs;

use std::path::PathBuf;
use sha2::{Digest, Sha256};
#[cfg(all(not(debug_assertions), feature = "nostr"))]
use ::hex;
#[cfg(all(not(debug_assertions), feature = "nostr"))]
use std::io::Write;



#[cfg(all(not(debug_assertions), feature = "nostr"))]
async fn publish_nostr_event_if_release(
    client: &mut nostr_sdk::Client,
    hash: String,
    keys: Keys,
    event_builder: EventBuilder,
    _relay_urls: &mut Vec<String>,
    file_path_str: &str,
    output_dir: &PathBuf,
    total_bytes_sent: &mut usize,
) -> Option<EventId> {
    let public_key = keys.public_key().to_string();

    let event = client.sign_event_builder(event_builder).await.unwrap();

    match client.send_event(&event).await {        Ok(event_output) => {
            println!("cargo:warning=Published Nostr event for {}: {}", file_path_str, event_output.val);

            let event_json_size = to_string(&event).map(|s| s.as_bytes().len()).unwrap_or(0);
            // Print successful relays
            for relay_url in event_output.success.iter() {
                println!("cargo:warning=Successfully published to relay: {} ({} bytes)", relay_url, event_json_size);
                *total_bytes_sent += event_json_size;
            }
            // Print failed relays and remove "unfriendly" relays from the list
            for (relay_url, error_msg) in event_output.failed.iter() {
                if should_remove_relay(error_msg) {
                    if let Err(e) = client.remove_relay(relay_url).await {
                        println!("cargo:warning=Failed to remove relay {}: {}", relay_url, e);
                    }
                     // println!("cargo:warning=Removed relay {}", relay_url);
                }
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
    client: &mut nostr_sdk::Client,
    _keys: &Keys,
    relay_urls: &Vec<String>,
    repo_url: &str,
    repo_name: &str,
    repo_description: &str,
    git_commit_hash: &str,
    git_branch: &str,
    output_dir: &PathBuf,
    public_key_hex: &str,
) -> Option<EventId> {

    let mut tags = vec![
        Tag::parse(["d", repo_name].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["name", repo_name].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["description", repo_description].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["web", repo_url].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["clone", repo_url].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["r", git_commit_hash, "euc"].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["commit", git_commit_hash].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["branch", git_branch].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["maintainers", "gnostr"].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        //Tag::parse(["t", "personal-fork"].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["t", "gnostr"].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["t", repo_name].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
    ];

    // Append each relay url
    for relay in relay_urls {
        tags.push(Tag::parse(["relays", relay].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap());
    }
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
#[cfg(all(not(debug_assertions), feature = "nostr"))]
pub async fn get_repo_patch_event(
    client: &mut nostr_sdk::Client,
    _keys: &Keys,
    _relay_urls: &Vec<String>,
    repo_url: &str,
    repo_name: &str,
    repo_description: &str,
    git_commit_hash: &str,
    git_branch: &str,
    output_dir: &PathBuf,
    public_key_hex: &str,
) -> Option<EventId> {

    let tags = vec![
        Tag::parse(["r", repo_url].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["name", repo_name].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["description", repo_description].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["commit", git_commit_hash].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["branch", git_branch].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
    ];

    let event_builder = EventBuilder::new(Kind::Custom(1617), repo_description).tags(tags);
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
    let mut git_commit_hash_str = String::new();
    let mut git_branch_str = String::new();



    println!("cargo:rustc-env=CARGO_PKG_NAME={}", env!("CARGO_PKG_NAME"));
    println!("cargo:rustc-env=CARGO_PKG_VERSION={}", env!("CARGO_PKG_VERSION"));

    if is_git_repo {
        let git_commit_hash_output = std::process::Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .expect("Failed to execute git command for commit hash");

        git_commit_hash_str = if git_commit_hash_output.status.success() && !git_commit_hash_output.stdout.is_empty() {
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

        git_branch_str = if git_branch_output.status.success() && !git_branch_output.stdout.is_empty() {
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

    //#[cfg(all(not(debug_assertions), feature = "nostr"))]
    //let relay_urls = get_file_hash_core::get_relay_urls();

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

        // Initialize client and keys once
        let initial_keys = Keys::new(SecretKey::from_hex(&hex::encode(Sha256::digest("initial_seed".as_bytes()))).expect("Failed to create initial Nostr keys"));
        let mut client = nostr_sdk::Client::new(initial_keys.clone());
        let mut relay_urls = get_file_hash_core::get_relay_urls();

        // Add relays to the client
        for relay_url in relay_urls.iter() {
            if let Err(e) = client.add_relay(relay_url).await {
                println!("cargo:warning=Failed to add relay {}: {}", relay_url, e);
            }
        }
        client.connect().await;
        println!("cargo:warning=Added and connected to {} relays.", relay_urls.len());

        let mut published_event_ids: Vec<Tag> = Vec::new();
        let mut total_bytes_sent: usize = 0;
    
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

                            if let Some(event_id) = publish_nostr_event_if_release(&mut client, file_hash_hex, keys.clone(), event_builder, &mut relay_urls, file_path_str, &output_dir, &mut total_bytes_sent).await {
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

        // Create and publish the build_manifest
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
                &mut client,
                hex::encode(Sha256::digest(content.as_bytes())),
                keys,
                event_builder,
                &mut relay_urls,
                "build_manifest.json",
                &output_dir,
                &mut total_bytes_sent,
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
            let git_commit_hash = &git_commit_hash_str;
            let git_branch = &git_branch_str;
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
                &mut client,
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
        println!("cargo:warning=Total bytes sent to Nostr relays: {} bytes ({} MB)", total_bytes_sent, total_bytes_sent as f64 / 1024.0 / 1024.0);
    }
}
// deterministic nostr event build example
