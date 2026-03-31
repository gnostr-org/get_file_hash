use std::process::Command;
use std::path::PathBuf;
#[cfg(feature = "nostr")]
use nostr_sdk::prelude::*;
#[cfg(feature = "nostr")]
use serde_json::json;

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
///
/// let keys = file_hash_as_nostr_private_key!("lib.rs");
/// println!("Public Key: {}", keys.public_key().to_bech32().unwrap());
/// ```
#[cfg(feature = "nostr")]
#[macro_export]
macro_rules! file_hash_as_nostr_private_key {
    ($file_path:expr) => {{
        let hash_hex = $crate::get_file_hash!($file_path);
        nostr::Keys::from_hex_secret_key(hash_hex).expect("Failed to create Nostr Keys from file hash")
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
    relay_url: &str,
    picture_url: &str,
    banner_url: &str,
    file_path_str: &str,
) {
    let client = nostr_sdk::Client::new(keys);

    if let Err(e) = client.add_relay(relay_url).await {
        println!("cargo:warning=Failed to add relay for metadata {}: {}", relay_url, e);
        return;
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

    let event = EventBuilder::metadata(&metadata)
        .to_event(keys)
        .unwrap();

    match client.send_event(event).await {
        Ok(event_id) => {
            println!("cargo:warning=Published Nostr metadata event for {}: {}", file_path_str, event_id);
        }
        Err(e) => {
            println!("cargo:warning=Failed to publish Nostr metadata event for {}: {}", file_path_str, e);
        }
    }
}