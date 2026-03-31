/// deterministic nostr event build example
// deterministic nostr event build example
use get_file_hash_core::get_file_hash;
use nostr_sdk::prelude::*;
use std::fs;
use sha2::{Digest, Sha256};
use hex;
use std::path::PathBuf;
use std::io::Write;

async fn publish_nostr_event_if_release(
	hash: String,
    keys: Keys,
    event: Event,
    relay_url: &str,
    file_path_str: &str,
) {
    let client = nostr_sdk::Client::new(&keys);
	let public_key = keys.public_key().to_string();

    if let Err(e) = client.add_relay(relay_url).await {
        println!("cargo:warning=Failed to add relay {}: {}", relay_url, e);
        return;
    }
    println!("cargo:warning=Added relay {}", relay_url);

    client.connect().await;
    println!("cargo:warning=Connected to relay {}", relay_url);

    let output_dir = PathBuf::from(".gnostr/build");
    if let Err(e) = fs::create_dir_all(&output_dir) {
        println!("cargo:warning=Failed to create output directory {}: {}", output_dir.display(), e);
        return;
    }

    match client.send_event(event.clone()).await {
        Ok(event_id) => {
            println!("cargo:warning=Published Nostr event for {}: {}", file_path_str, event_id);
            let filename = format!("{}/{}/{}.json", hash, public_key.clone(), event_id);
            let file_path = output_dir.join(&filename);
            if let Some(parent) = file_path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    println!("cargo:warning=Failed to create parent directories for {}: {}", file_path.display(), e);
                    return;
                }
            }
            if let Err(e) = fs::File::create(&file_path).and_then(|mut file| write!(file, "{}", event.as_json())) {
                println!("cargo:warning=Failed to write event JSON to file {}: {}", file_path.display(), e);
            } else {
                println!("cargo:warning=Successfully wrote event JSON to {}", file_path.display());
            }
        },
        Err(e) => {
            println!("cargo:warning=Failed to publish Nostr event for {}: {}", file_path_str, e);
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
    println!("cargo:rustc-env=BUILD_HASH={}", core_hash);

    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/get_file_hash_core/src/lib.rs");
    println!("cargo:rerun-if-changed=build.rs");

    if cfg!(not(debug_assertions)) {
        // This code only runs in release builds
        let relay_url = ["wss://relay.damus.io", "wss://nos.lol"];

        let files_to_publish = [
            "Cargo.toml",
            "src/lib.rs",
            "build.rs",
            "src/get_file_hash_core/src/lib.rs",
        ];

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
                            let event = EventBuilder::text_note(content, vec![]).to_event(&keys).unwrap();

                            publish_nostr_event_if_release(file_hash_hex, keys, event, relay_url[1], file_path_str).await;
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
    }
}
// deterministic nostr event build example
