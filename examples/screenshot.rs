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
//! # Dry-run (ephemeral keys, prints the auth header and curl command)
//! cargo run --example screenshot --features nostr
//! ```
//!
//! # Environment variables
//!
//! * `NOSTR_SEC`       – bech32-encoded secret key (`nsec1...`).  When absent
//!                       an ephemeral key pair is generated and the upload is
//!                       skipped (dry-run mode).
//! * `NIP96_SERVER`    – NIP-96 server base URL.
//!                       Defaults to `https://nostr.build`.
//! * `PATH`            – Optional screenshot file path. When omitted, the
//!                       example uses an embedded 1x1 PNG placeholder so the
//!                       command works out of the box.

use nostr::nips::nip96;
use nostr::prelude::*;
use std::env;
use std::process::Command;

const PLACEHOLDER_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
    0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
    0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00,
    0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x60, 0x00, 0x00, 0x00,
    0x02, 0x00, 0x01, 0xE5, 0x27, 0xD4, 0xA6, 0x00, 0x00, 0x00, 0x00, 0x49,
    0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ------------------------------------------------------------------
    // 1. Resolve signing keys
    // ------------------------------------------------------------------
    let (keys, dry_run) = match env::var("NOSTR_SEC") {
        Ok(nsec) => {
            println!("Using keys from NOSTR_SEC");
            (Keys::parse(&nsec)?, false)
        }
        Err(_) => {
            eprintln!(
                "NOSTR_SEC not set – generating ephemeral keys (dry-run, no upload)"
            );
            (Keys::generate(), true)
        }
    };

    // ------------------------------------------------------------------
    // 2. Resolve the screenshot file path
    // ------------------------------------------------------------------
    let file_path = env::args().nth(1);
    let (file_path, file_data) = match file_path {
        Some(path) => {
            println!("Reading file: {}", path);
            (path.clone(), std::fs::read(&path)?)
        }
        None => {
            let path = "screenshot.png".to_string();
            println!("No file path provided; using embedded placeholder PNG");
            (path, PLACEHOLDER_PNG.to_vec())
        }
    };
    println!("File size: {} bytes", file_data.len());

    // ------------------------------------------------------------------
    // 3. Resolve the NIP-96 server
    // ------------------------------------------------------------------
    let server_url = Url::parse(
        &env::var("NIP96_SERVER").unwrap_or_else(|_| "https://nostr.build".to_string()),
    )?;
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

    if dry_run {
        println!("\n[dry-run] Skipping upload (set NOSTR_SEC to upload for real).");
        return Ok(());
    }

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

    if !upload_output.status.success() {
        eprintln!(
            "Upload failed (curl exit status {}): {}",
            upload_output.status,
            String::from_utf8_lossy(&upload_output.stderr)
        );
        return Ok(());
    }

    // ------------------------------------------------------------------
    // 7. Parse and display the result
    // ------------------------------------------------------------------
    let response = nip96::UploadResponse::from_json(&upload_output.stdout)?;
    match response.download_url() {
        Ok(url) => println!("Upload successful!\nFile URL: {url}"),
        Err(e) => eprintln!("Upload response error: {e}"),
    }

    Ok(())
}
