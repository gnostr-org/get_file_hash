use std::process::Command;
use std::path::PathBuf;
#[cfg(feature = "nostr")]
use nostr_sdk::prelude::*;
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

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use sha2::{Digest, Sha256};
    use tempfile;
    use super::get_git_tracked_files;
    use std::process::Command;

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
        Command::new("git")
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
        Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(repo_path)
            .output()
            .expect("Failed to git add files");
        Command::new("git")
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
        let picture_url = "https://example.com/picture.jpg";
        let banner_url = "https://example.com/banner.jpg";
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
}