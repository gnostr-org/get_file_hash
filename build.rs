/// deterministic nostr event build example
// deterministic nostr event build example
use get_file_hash_core::get_file_hash;
use nostr_sdk::prelude::*;
use std::fs;
use sha2::{Digest, Sha256};
use hex;
use std::path::PathBuf;
use std::io::Write;
use std::process::Command;

async fn publish_nostr_event_if_release(
	hash: String,
    keys: Keys,
    event: Event,
    relay_url: &str,
    file_path_str: &str,
) -> Option<EventId> {
    let client = nostr_sdk::Client::new(&keys);
	let public_key = keys.public_key().to_string();

    if let Err(e) = client.add_relay(relay_url).await {
        println!("cargo:warning=Failed to add relay {}: {}", relay_url, e);
        return None;
    }
    println!("cargo:warning=Added relay {}", relay_url);

    client.connect().await;
    println!("cargo:warning=Connected to relay {}", relay_url);

    let package_version = std::env::var("CARGO_PKG_VERSION").unwrap();
    let output_dir = PathBuf::from(format!(".gnostr/build/{}", package_version));
    if let Err(e) = fs::create_dir_all(&output_dir) {
        println!("cargo:warning=Failed to create output directory {}: {}", output_dir.display(), e);
        return None;
    }

    match client.send_event(event.clone()).await {
        Ok(event_id) => {
            println!("cargo:warning=Published Nostr event for {}: {}", file_path_str, event_id);
            let filename = format!("{}/{}/{}.json", hash, public_key.clone(), event_id);
            let file_path = output_dir.join(&filename);
            if let Some(parent) = file_path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    println!("cargo:warning=Failed to create parent directories for {}: {}", file_path.display(), e);
                    return None;
                }
            }
            if let Err(e) = fs::File::create(&file_path).and_then(|mut file| write!(file, "{}", event.as_json())) {
                println!("cargo:warning=Failed to write event JSON to file {}: {}", file_path.display(), e);
            } else {
                println!("cargo:warning=Successfully wrote event JSON to {}", file_path.display());
            }
            Some(event_id)
        },
        Err(e) => {
            println!("cargo:warning=Failed to publish Nostr event for {}: {}", file_path_str, e);
            None
        },
    }
}

#[tokio::main]
async fn main() {
    //println!("cargo:rerun-if-changed=ALWAYS_RUN_NONEXISTENT_FILE");

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let is_git_repo = std::path::Path::new(&manifest_dir).join(".git").exists();

    if !is_git_repo {
        println!("cargo:rustc-cfg=is_published_source");
    } else {
        println!("cargo:rerun-if-changed=ALWAYS_RUN_NONEXISTENT_FILE");
    }

    let cargo_toml_hash = get_file_hash!("Cargo.toml");
    println!("cargo:rustc-env=CARGO_TOML_HASH={}", cargo_toml_hash);

    let lib_hash = get_file_hash!("src/lib.rs");
    println!("cargo:rustc-env=LIB_HASH={}", lib_hash);

    let build_hash = get_file_hash!("build.rs");
    println!("cargo:rustc-env=BUILD_HASH={}", build_hash);
                                //prepend get_file_hash version to path
    let core_hash = get_file_hash!(/*get_file_hash-version*/"src/get_file_hash_core/src/lib.rs");
    println!("cargo:rustc-env=CORE_HASH={}", core_hash);

    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/get_file_hash_core/src/lib.rs");
    println!("cargo:rerun-if-changed=build.rs");

    if cfg!(not(debug_assertions)) {
        // This code only runs in release builds
        let relay_url = ["wss://relay.damus.io", "wss://nos.lol"];
        let package_version = std::env::var("CARGO_PKG_VERSION").unwrap();

        let files_to_publish: Vec<String> = String::from_utf8_lossy(
            &Command::new("git")
                .arg("ls-files")
                .current_dir(&manifest_dir)
                .output()
                .expect("Failed to execute git ls-files")
                .stdout
        )
        .lines()
        .filter_map(|line| Some(String::from(line)))
        .collect();
        
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
                            let event = EventBuilder::text_note(content, tags).to_event(&keys).unwrap();

                            if let Some(event_id) = publish_nostr_event_if_release(file_hash_hex, keys, event, relay_url[1], file_path_str).await {
                                published_event_ids.push(Tag::event(event_id));
                            }
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
            let keys = Keys::generate(); // Generate new keys for the linking event
            let content = format!("Build manifest for get_file_hash v{}", package_version);
            let mut tags = vec![
                Tag::parse(["build_manifest", &package_version].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
            ];
            tags.extend(published_event_ids);

            let event = EventBuilder::text_note(content, tags).to_event(&keys).unwrap();
            
            // Use a dummy hash and file_path_str for the linking event, as it's not tied to a single file
            publish_nostr_event_if_release(
                hex::encode(Sha256::digest(event.as_json().as_bytes())),
                keys,
                event,
                relay_url[1],
                "build_manifest.json",
            ).await;
        }
    }
}
// deterministic nostr event build example
