//! NIP-96 Screenshot Upload Example
//!
//! Demonstrates how to upload a screenshot (PNG) to a NIP-96 compliant file
//! storage server using the `nostr` crate for authentication.
//!
//! The NIP-98 HTTP Auth header is constructed by the `nostr` crate; the
//! actual HTTP multipart POST is performed via `curl`, which is available in
//! most CI environments without adding a Rust HTTP-client dependency.
//!
//! # Usage
//!
//! ```
//! # Upload screenshot.png using NOSTR_SEC for signing
//! NOSTR_SEC=nsec1... cargo run --example screenshot --features nostr -- screenshot.png
//!
//! # Use the screenshot hash as the signing key when NOSTR_SEC is unset
//! cargo run --example screenshot --features nostr
//! ```
//!
//! # CLI arguments and environment variables
//!
//! * `PATH`            – Optional screenshot file path. When omitted, the
//!                       example uses an embedded 1x1 PNG placeholder so the
//!                       command works out of the box.
//! * `--server`        – NIP-96 server base URL. Defaults to `https://nostr.build`.
//! * `--nostr-sec`     – Bech32-encoded secret key (`nsec1...`). When omitted,
//!                       the example falls back to `NOSTR_SEC`, then the file
//!                       SHA-256.
//! * `--relay`         – Extra relay URL to syndicate the upload result to.
//!
//! # Environment variables
//!
//! * `NOSTR_SEC`       – bech32-encoded secret key (`nsec1...`).  When absent
//!                       the screenshot or placeholder PNG SHA-256 is used as
//!                       a deterministic private key.
//! * `NIP96_SERVER`    – NIP-96 server base URL.
//!                       Defaults to `https://nostr.build`.

use clap::{ArgAction, Parser};
use get_file_hash_core::get_relay_urls;
use nostr::nips::nip96;
use nostr::prelude::*;
use nostr_sdk::Client;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use sha2::{Digest, Sha256};

const PLACEHOLDER_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
    0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
    0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00,
    0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x60, 0x00, 0x00, 0x00,
    0x02, 0x00, 0x01, 0xE5, 0x27, 0xD4, 0xA6, 0x00, 0x00, 0x00, 0x00, 0x49,
    0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

#[derive(Debug, Parser)]
#[command(name = "screenshot", version, about = "Upload a PNG screenshot to NIP-96 and syndicate the result")]
struct Args {
    /// Screenshot file path. When omitted, a placeholder PNG is used.
    #[arg(value_name = "PATH")]
    path: Option<PathBuf>,

    /// NIP-96 server base URL.
    #[arg(long, value_name = "URL", default_value = "https://nostr.build")]
    server: String,

    /// Bech32 secret key (`nsec1...`). Overrides NOSTR_SEC when set.
    #[arg(long = "nostr-sec", value_name = "NSEC")]
    nostr_sec: Option<String>,

    /// Additional relay URL to syndicate the upload result to.
    #[arg(long = "relay", value_name = "URL", action = ArgAction::Append)]
    relays: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // ------------------------------------------------------------------
    // 1. Resolve the screenshot file path
    // ------------------------------------------------------------------
    let (file_path, file_data) = match args.path {
        Some(path) => {
            println!("Reading file: {}", path);
            (path.display().to_string(), std::fs::read(&path)?)
        }
        None => {
            let path = PathBuf::from(env::temp_dir())
                .join(format!("get_file_hash-screenshot-{}.png", std::process::id()));
            fs::write(&path, PLACEHOLDER_PNG)?;
            println!("No file path provided; using embedded placeholder PNG");
            (path.display().to_string(), PLACEHOLDER_PNG.to_vec())
        }
    };
    println!("File size: {} bytes", file_data.len());

    // ------------------------------------------------------------------
    // 2. Resolve signing keys
    // ------------------------------------------------------------------
    let keys = match args.nostr_sec.as_deref().or_else(|| env::var("NOSTR_SEC").ok().as_deref()) {
        Some(nsec) => {
            println!("Using keys from NOSTR_SEC / --nostr-sec");
            Keys::parse(nsec)?
        }
        Err(_) => {
            let secret_key_hex = hex::encode(Sha256::digest(&file_data));
            println!("NOSTR_SEC not set – deriving deterministic keys from file sha256");
            Keys::parse(&secret_key_hex)?
        }
    };

    // ------------------------------------------------------------------
    // 3. Resolve the NIP-96 server
    // ------------------------------------------------------------------
    let server_url = Url::parse(&args.server)?;
    println!("NIP-96 server: {}", server_url);

    // ------------------------------------------------------------------
    // 4. Fetch server configuration (/.well-known/nostr/nip96.json)
    // ------------------------------------------------------------------
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

    // ------------------------------------------------------------------
    // 5. Build NIP-96 upload request (NIP-98 Authorization header)
    // ------------------------------------------------------------------
    let upload_request = nip96::UploadRequest::new(&keys, &config, &file_data).await?;
    // Note: the Authorization value is a base64-encoded signed Nostr event
    // (NIP-98), not the private key.  It is printed here only for debugging;
    // do not log it in non-example, production code.
    println!("Authorization header: {}", upload_request.authorization());
    println!("Upload URL:           {}", upload_request.url());

    // Print the equivalent curl command so users can reproduce it manually.
    println!();
    println!("Equivalent curl command:");
    println!(
        "  curl -X POST '{}' \\\n    \
         -H 'Authorization: {}' \\\n    \
         -F 'file=@{};type=image/png'",
        upload_request.url(),
        upload_request.authorization(),
        file_path
    );

    // ------------------------------------------------------------------
    // 6. Upload via curl multipart POST
    // ------------------------------------------------------------------
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
            &format!("file=@{};type=image/png", file_path),
        ])
        .output()?;

    let upload_summary = if upload_output.status.success() {
        // ------------------------------------------------------------------
        // 7. Parse and display the result
        // ------------------------------------------------------------------
        let response = nip96::UploadResponse::from_json(&upload_output.stdout)?;
        match response.download_url() {
            Ok(url) => {
                println!("Upload successful!\nFile URL: {url}");
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
            upload_output.status,
            stderr
        );
        format!("Upload failed (curl exit status {}): {}", upload_output.status, stderr)
    };

    let relay_urls = if args.relays.is_empty() {
        get_relay_urls()
    } else {
        args.relays
    };
    if !relay_urls.is_empty() {
        let client = Client::new(keys.clone());
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
