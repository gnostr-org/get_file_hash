//! A crate providing the `get_file_hash!` procedural macro.
//!
//! This macro allows you to compute the SHA-256 hash of a file at compile time,
//! embedding the resulting hash string directly into your Rust executable.

pub use get_file_hash_core::get_file_hash;

#[cfg(feature = "nostr")]
use chrono::Utc;
#[cfg(feature = "nostr")]
use get_file_hash_core::get_relay_urls;
#[cfg(feature = "nostr")]
use nostr::nips::nip96;
#[cfg(feature = "nostr")]
use nostr::prelude::*;
#[cfg(feature = "nostr")]
use sha2::Digest;
#[cfg(feature = "nostr")]
use std::env;
#[cfg(feature = "nostr")]
use std::error::Error;
#[cfg(feature = "nostr")]
use std::fs;
#[cfg(feature = "nostr")]
use std::path::{Path, PathBuf};
#[cfg(feature = "nostr")]
use std::process::Command;
#[cfg(feature = "nostr")]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(feature = "nostr")]
const DEFAULT_MAX_POW_ATTEMPTS: u128 = 100_000_000;

#[cfg(feature = "nostr")]
pub const EMBEDDED_ICON_SVG: &[u8] = include_bytes!("get_file_hash_core/src/icon.svg");

#[cfg(feature = "nostr")]
pub const EMBEDDED_ICON_PNG: &[u8] = include_bytes!("get_file_hash_core/src/icon.png");

#[cfg(feature = "nostr")]
pub const EMBEDDED_PLACEHOLDER_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
    0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
    0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00,
    0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x60, 0x00, 0x00, 0x00,
    0x02, 0x00, 0x01, 0xE5, 0x27, 0xD4, 0xA6, 0x00, 0x00, 0x00, 0x00, 0x49,
    0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

#[cfg(feature = "nostr")]
static ICON_OUTPUT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[cfg(feature = "nostr")]
#[derive(Debug, Clone, Default)]
pub struct ScreenshotOptions {
    pub path: Option<PathBuf>,
    pub server: Option<String>,
    pub nostr_sec: Option<String>,
    pub relays: Vec<String>,
}

#[cfg(feature = "nostr")]
pub fn content_type_for_path(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .as_deref()
    {
        Some("png") => "image/png",
        Some("svg") => "image/svg+xml",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        _ => "application/octet-stream",
    }
}

#[cfg(feature = "nostr")]
pub fn timestamped_icon_output_path_in_dir(dir: &Path, original_name: &str) -> PathBuf {
    let timestamp = Utc::now().timestamp();
    let sequence = ICON_OUTPUT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let original = Path::new(original_name);
    let stem = original
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("icon");
    let ext = original.extension().and_then(|value| value.to_str()).unwrap_or("");
    let file_name = if ext.is_empty() {
        format!("{stem}-{timestamp}-{sequence}")
    } else {
        format!("{stem}-{timestamp}-{sequence}.{ext}")
    };
    dir.join(file_name)
}

#[cfg(feature = "nostr")]
pub fn timestamped_icon_output_path_for_content_type(dir: &Path, content_type: &str) -> PathBuf {
    match content_type {
        "image/png" => timestamped_icon_output_path_in_dir(dir, "icon.png"),
        "image/svg+xml" => timestamped_icon_output_path_in_dir(dir, "icon.svg"),
        _ => timestamped_icon_output_path_in_dir(dir, "icon.bin"),
    }
}

#[cfg(feature = "nostr")]
pub fn timestamped_icon_output_path_for_download_url(dir: &Path, download_url: &Url) -> PathBuf {
    let ext = download_url
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|filename| Path::new(filename).extension())
        .and_then(|value| value.to_str())
        .unwrap_or("");
    let timestamp = Utc::now().timestamp();
    let sequence = ICON_OUTPUT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let file_name = if ext.is_empty() {
        format!("icon-{timestamp}-{sequence}")
    } else {
        format!("icon-{timestamp}-{sequence}.{ext}")
    };
    dir.join(file_name)
}

#[cfg(feature = "nostr")]
pub fn extract_multipart_file_bytes(bytes: &[u8]) -> &[u8] {
    let first_line_end = match bytes.windows(2).position(|window| window == b"\r\n") {
        Some(pos) => pos,
        None => return bytes,
    };

    if !bytes.starts_with(b"--") {
        return bytes;
    }

    let boundary = &bytes[2..first_line_end];
    let header_end = match bytes.windows(4).position(|window| window == b"\r\n\r\n") {
        Some(pos) => pos + 4,
        None => return bytes,
    };

    let mut terminator = Vec::with_capacity(boundary.len() + 6);
    terminator.extend_from_slice(b"\r\n--");
    terminator.extend_from_slice(boundary);
    terminator.extend_from_slice(b"--");

    match bytes.windows(terminator.len()).rposition(|window| window == terminator.as_slice()) {
        Some(end) if end >= header_end => &bytes[header_end..end],
        _ => bytes,
    }
}

#[cfg(feature = "nostr")]
pub async fn run_screenshot_upload(options: ScreenshotOptions) -> Result<(), Box<dyn Error>> {
    let file_content_type = options
        .path
        .as_deref()
        .map(content_type_for_path)
        .unwrap_or("image/svg+xml");

    let (file_path, file_data) = match options.path {
        Some(path) => {
            println!("Reading file: {}", path.display());
            (path.display().to_string(), fs::read(&path)?)
        }
        None => {
            let path = env::current_dir()?.join("icon.svg");
            fs::write(&path, EMBEDDED_ICON_SVG)?;
            println!("No file path provided; wrote embedded icon.svg to: {}", path.display());
            (path.display().to_string(), EMBEDDED_ICON_SVG.to_vec())
        }
    };
    println!("File size: {} bytes", file_data.len());

    let nostr_sec = options.nostr_sec.or_else(|| env::var("NOSTR_SEC").ok());
    let keys = match nostr_sec.as_deref() {
        Some(nsec) => {
            println!("Using keys from NOSTR_SEC / --nostr-sec");
            Keys::parse(nsec)?
        }
        None => {
            let secret_key_hex = hex::encode(sha2::Sha256::digest(&file_data));
            println!("NOSTR_SEC not set – deriving deterministic keys from file sha256");
            Keys::parse(&secret_key_hex)?
        }
    };

    let server_url = Url::parse(
        &options
            .server
            .or_else(|| env::var("NIP96_SERVER").ok())
            .unwrap_or_else(|| "https://nostr.build".to_string()),
    )?;
    println!("NIP-96 server: {}", server_url);

    let config_url = nip96::get_server_config_url(&server_url)?;
    println!("Fetching config from: {}", config_url);

    let config_output = Command::new("curl")
        .args(["--silent", "--fail", "--location", config_url.as_str()])
        .output();

    let config = match config_output {
        Ok(out) if out.status.success() => nip96::ServerConfig::from_json(&out.stdout)?,
        Ok(out) => {
            eprintln!(
                "Failed to fetch server config (curl exit status {}): {}",
                out.status,
                String::from_utf8_lossy(&out.stderr)
            );
            return Ok(());
        }
        Err(e) => {
            eprintln!("curl not available or failed: {e}");
            return Ok(());
        }
    };

    println!("Upload endpoint: {}", config.api_url);

    let upload_request = nip96::UploadRequest::new(&keys, &config, &file_data).await?;
    println!("Authorization header: {}", upload_request.authorization());
    println!("Upload URL:           {}", upload_request.url());

    println!();
    println!("Equivalent curl command:");
    println!(
        "  curl -X POST '{}' \\\n    \
         -H 'Authorization: {}' \\\n    \
         -F 'file=@{};type={}'",
        upload_request.url(),
        upload_request.authorization(),
        file_path,
        file_content_type
    );

    println!("\nUploading…");
    let upload_output = Command::new("curl")
        .args([
            "--silent",
            "--fail",
            "--location",
            "-X",
            "POST",
            upload_request.url().as_str(),
            "-H",
            &format!("Authorization: {}", upload_request.authorization()),
            "-F",
            &format!("file=@{};type={}", file_path, file_content_type),
        ])
        .output()?;

    let upload_summary = if upload_output.status.success() {
        let response = nip96::UploadResponse::from_json(&upload_output.stdout)?;
        match response.download_url() {
            Ok(url) => {
                println!("Upload successful!\nFile URL: {url}");
                let download_output = Command::new("curl")
                    .args(["--silent", "--fail", "--location", url.as_str()])
                    .output()?;
                if download_output.status.success() {
                    let output_dir = env::current_dir()?;
                    let saved_path = timestamped_icon_output_path_for_download_url(&output_dir, &url);
                    let image_bytes = extract_multipart_file_bytes(&download_output.stdout);
                    fs::write(&saved_path, image_bytes)?;
                    println!("Saved returned image to: {}", saved_path.display());
                } else {
                    eprintln!(
                        "Failed to download returned image (curl exit status {}): {}",
                        download_output.status,
                        String::from_utf8_lossy(&download_output.stderr)
                    );
                }
                format!("Upload successful: {url}")
            }
            Err(e) => {
                eprintln!("Upload response error: {e}");
                format!("Upload response error: {e}")
            }
        }
    } else {
        let stderr = String::from_utf8_lossy(&upload_output.stderr);
        eprintln!(
            "Upload failed (curl exit status {}): {}",
            upload_output.status, stderr
        );
        format!("Upload failed (curl exit status {}): {}", upload_output.status, stderr)
    };

    let relay_urls = if options.relays.is_empty() {
        get_relay_urls()
    } else {
        options.relays
    };
    if !relay_urls.is_empty() {
        let client = nostr_sdk::Client::new(keys.clone());
        for relay_url in &relay_urls {
            if let Err(e) = client.add_relay(relay_url).await {
                eprintln!("Failed to add relay {}: {}", relay_url, e);
            }
        }
        client.connect().await;

        let note = EventBuilder::text_note(format!(
            "Screenshot upload result for {}: {}",
            file_path, upload_summary
        ))
        .sign_with_keys(&keys)?;

        if let Err(e) = client.send_event(&note).await {
            eprintln!("Failed to syndicate screenshot upload to relays: {}", e);
        } else {
            println!("Syndicated upload announcement to {} relays", relay_urls.len());
        }
    }

    Ok(())
}

/// Mine a Nostr event whose ID starts with the given lowercase-hex prefix.
///
/// The SDK's [`EventBuilder::pow`](nostr_sdk::EventBuilder::pow) only supports
/// NIP-13 leading-zero-bits difficulty. This helper does custom-prefix mining
/// by brute-forcing a `["nonce", "<nonce>", "0"]` tag until the computed
/// event ID matches the desired prefix.
///
/// # Arguments
///
/// * `keys` - The keypair used to sign the event.
/// * `builder` - A pre-configured event builder (content, tags, kind, ...).
/// * `prefix` - The desired hex prefix (case-insensitive). Each extra hex
///   character increases the expected search time by a factor of 16.
/// * `max_attempts` - Safety cap. Returns `None` if no matching nonce is found.
///
/// # Example
///
/// ```no_run
/// use get_file_hash::mine_event_with_prefix;
/// use nostr_sdk::prelude::{EventBuilder, Keys, Tag};
///
/// let keys = Keys::generate(); ## padded commit hash?
/// let builder = EventBuilder::text_note("hello")
///     .tag(Tag::parse(["example", "pow"]).unwrap());
///
/// if let Some(event) = mine_event_with_prefix(&keys, builder, "0000", 10_000_000) {
///     println!("mined event id: {}", event.id);
/// }
/// ```
#[cfg(feature = "nostr")]
pub fn mine_event_with_prefix(
    keys: &Keys,
    builder: EventBuilder,
    prefix: &str,
    max_attempts: u128,
) -> Option<Event> {
    let pubkey = keys.public_key();
    let prefix = prefix.to_ascii_lowercase();
    let mut nonce: u128 = 0;

    loop {
        let mut attempt = builder.clone();
        attempt = attempt.tag(Tag::pow(nonce, 0));

        let mut unsigned: UnsignedEvent = attempt.build(pubkey);
        let id: EventId = unsigned.id();

        if id.to_hex().starts_with(&prefix) {
            return unsigned.sign_with_keys(keys).ok();
        }

        nonce += 1;
        if nonce > max_attempts {
            return None;
        }
    }
}

/// Convenience wrapper around [`mine_event_with_prefix`] with a default attempt
/// cap of 100,000,000.
#[cfg(feature = "nostr")]
pub fn mine_event_with_prefix_or_default(
    keys: &Keys,
    builder: EventBuilder,
    prefix: &str,
) -> Option<Event> {
    mine_event_with_prefix(keys, builder, prefix, DEFAULT_MAX_POW_ATTEMPTS)
}

/// The SHA-256 hash of this crate's `build.rs` at the time of compilation.
pub const BUILD_HASH: &str = env!("BUILD_HASH");

/// The SHA-256 hash of this crate's `Cargo.toml` at the time of compilation.
pub const CARGO_TOML_HASH: &str = env!("CARGO_TOML_HASH");

/// The SHA-256 hash of this crate's `src/lib.rs` at the time of compilation.
pub const LIB_HASH: &str = env!("LIB_HASH");

/// The name of the package as specified in Cargo.toml.
pub const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");

/// The version of the package as specified in Cargo.toml.
pub const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(feature = "nostr")]
/// The git commit hash of the repository at the time of compilation.
pub const GIT_COMMIT_HASH: &str = env!("GIT_COMMIT_HASH");

#[cfg(feature = "nostr")]
/// The git branch of the repository at the time of compilation.
pub const GIT_BRANCH: &str = env!("GIT_BRANCH");

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};

    use super::*;

    /// Verifies that the exported CARGO_TOML_HASH is not empty.
    #[test]
    fn test_injected_hash_exists() {
        assert!(!BUILD_HASH.is_empty());
        println!("Verified build.rs Hash:
{}", BUILD_HASH);

        assert!(!CARGO_TOML_HASH.is_empty());
        println!("Verified Cargo.toml Hash:
{}", CARGO_TOML_HASH);

        assert!(!LIB_HASH.is_empty());
        println!("Verified src/lib.rs Hash:\n{}", LIB_HASH);

        assert!(!CARGO_PKG_NAME.is_empty());
        println!("Verified Package Name:\n{}", CARGO_PKG_NAME);

        assert!(!CARGO_PKG_VERSION.is_empty());
        println!("Verified Package Version:\n{}", CARGO_PKG_VERSION);

        #[cfg(feature = "nostr")]
        {
            assert!(!GIT_COMMIT_HASH.is_empty());
            println!("Verified Git Commit Hash:\n{}", GIT_COMMIT_HASH);

            assert!(!GIT_BRANCH.is_empty());
            println!("Verified Git Branch:\n{}", GIT_BRANCH);
        }
    }

    /// Tests that the `get_file_hash!` macro correctly computes the SHA-256
    /// hash of `lib.rs` and that it matches a manually computed hash of the
    /// same file.
    #[test]
    fn test_get_lib_hash() {
        let file_content = include_bytes!("lib.rs");

        let mut hasher = Sha256::new();
        hasher.update(file_content);
        let expected_hash = hasher
            .finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        let actual_hash = get_file_hash!("lib.rs");
        assert_eq!(actual_hash, expected_hash);
    }
}
