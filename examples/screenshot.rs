//! NIP-96 Screenshot Upload Example
//!
//! Demonstrates how to upload a screenshot or image to a NIP-96 compliant file
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
//!                       example uses the embedded `icon.svg` asset so the
//!                       command works out of the box with a real image.
//! * `--server`        – NIP-96 server base URL. Defaults to `https://nostr.build`.
//! * `--nostr-sec`     – Bech32-encoded secret key (`nsec1...`). When omitted,
//!                       the example falls back to `NOSTR_SEC`, then the file
//!                       SHA-256.
//! * `--relay`         – Extra relay URL to syndicate the upload result to.
//!
//! # Environment variables
//!
//! * `NOSTR_SEC`       – bech32-encoded secret key (`nsec1...`).  When absent
//!                       the screenshot or embedded image SHA-256 is used as
//!                       a deterministic private key.
//! * `NIP96_SERVER`    – NIP-96 server base URL.
//!                       Defaults to `https://nostr.build`.

use clap::{ArgAction, Parser};
use chrono::Utc;
use get_file_hash_core::get_relay_urls;
use nostr::nips::nip96;
use nostr::prelude::*;
use nostr_sdk::Client;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use sha2::{Digest, Sha256};

const EMBEDDED_ICON_SVG: &[u8] = include_bytes!("../src/get_file_hash_core/src/icon.svg");
const EMBEDDED_ICON_SVG_CONTENT_TYPE: &str = "image/svg+xml";
const EMBEDDED_ICON_PNG: &[u8] = include_bytes!("../src/get_file_hash_core/src/icon.png");
const EMBEDDED_PLACEHOLDER_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
    0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
    0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00,
    0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x60, 0x00, 0x00, 0x00,
    0x02, 0x00, 0x01, 0xE5, 0x27, 0xD4, 0xA6, 0x00, 0x00, 0x00, 0x00, 0x49,
    0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

#[derive(Debug, Parser)]
#[command(name = "screenshot", version, about = "Upload an image to NIP-96 and syndicate the result")]
struct Args {
    /// Screenshot file path. When omitted, the embedded icon.svg is used.
    #[arg(value_name = "PATH")]
    path: Option<PathBuf>,

    /// NIP-96 server base URL.
    #[arg(long, value_name = "URL")]
    server: Option<String>,

    /// Bech32 secret key (`nsec1...`). Overrides NOSTR_SEC when set.
    #[arg(long = "nostr-sec", value_name = "NSEC")]
    nostr_sec: Option<String>,

    /// Additional relay URL to syndicate the upload result to.
    #[arg(long = "relay", value_name = "URL", action = ArgAction::Append)]
    relays: Vec<String>,
}

fn content_type_for_path(path: &Path) -> &'static str {
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

fn timestamped_icon_output_path_in_dir(dir: &Path, original_name: &str) -> PathBuf {
    let timestamp = Utc::now().timestamp();
    let original = Path::new(original_name);
    let stem = original
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("icon");
    let ext = original.extension().and_then(|value| value.to_str()).unwrap_or("");
    let file_name = if ext.is_empty() {
        format!("{stem}-{timestamp}")
    } else {
        format!("{stem}-{timestamp}.{ext}")
    };
    dir.join(file_name)
}

fn timestamped_icon_output_path_for_content_type(dir: &Path, content_type: &str) -> PathBuf {
    match content_type {
        "image/png" => timestamped_icon_output_path_in_dir(dir, "icon.png"),
        "image/svg+xml" => timestamped_icon_output_path_in_dir(dir, "icon.svg"),
        _ => timestamped_icon_output_path_in_dir(dir, "icon.bin"),
    }
}

fn timestamped_icon_output_path_for_download_url(dir: &Path, download_url: &Url) -> PathBuf {
    let filename = download_url
        .path_segments()
        .and_then(|segments| segments.last())
        .unwrap_or("icon.bin");
    let original = Path::new(filename);
    let stem = original
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("icon");
    let ext = original.extension().and_then(|value| value.to_str()).unwrap_or("");
    let timestamp = Utc::now().timestamp();
    let file_name = if ext.is_empty() {
        format!("{stem}-{timestamp}")
    } else {
        format!("{stem}-{timestamp}.{ext}")
    };
    dir.join(file_name)
}

fn extract_multipart_file_bytes(bytes: &[u8]) -> &[u8] {
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let file_content_type = args
        .path
        .as_deref()
        .map(content_type_for_path)
        .unwrap_or(EMBEDDED_ICON_SVG_CONTENT_TYPE);

    // ------------------------------------------------------------------
    // 1. Resolve the screenshot file path
    // ------------------------------------------------------------------
    let (file_path, file_data) = match args.path {
        Some(path) => {
            println!("Reading file: {}", path.display());
            (path.display().to_string(), std::fs::read(&path)?)
        }
        None => {
            let path = env::current_dir()?.join("icon.svg");
            fs::write(&path, EMBEDDED_ICON_SVG)?;
            println!("No file path provided; wrote embedded icon.svg to: {}", path.display());
            (path.display().to_string(), EMBEDDED_ICON_SVG.to_vec())
        }
    };
    println!("File size: {} bytes", file_data.len());

    // ------------------------------------------------------------------
    // 2. Resolve signing keys
    // ------------------------------------------------------------------
    let nostr_sec = args.nostr_sec.or_else(|| env::var("NOSTR_SEC").ok());
    let keys = match nostr_sec.as_deref() {
        Some(nsec) => {
            println!("Using keys from NOSTR_SEC / --nostr-sec");
            Keys::parse(nsec)?
        }
        None => {
            let secret_key_hex = hex::encode(Sha256::digest(&file_data));
            println!("NOSTR_SEC not set – deriving deterministic keys from file sha256");
            Keys::parse(&secret_key_hex)?
        }
    };

    // ------------------------------------------------------------------
    // 3. Resolve the NIP-96 server
    // ------------------------------------------------------------------
    let server_url = Url::parse(
        &args
            .server
            .or_else(|| env::var("NIP96_SERVER").ok())
            .unwrap_or_else(|| "https://nostr.build".to_string()),
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
         -F 'file=@{};type={}'",
        upload_request.url(),
        upload_request.authorization(),
        file_path,
        file_content_type
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
            &format!("file=@{};type={}", file_path, file_content_type),
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[test]
    fn cli_round_trips_positionals_and_flags() {
        let args = Args::try_parse_from([
            "screenshot",
            "shot.png",
            "--server",
            "https://example.com",
            "--nostr-sec",
            "nsec1example00000000000000000000000000000000000000000000000000000",
            "--relay",
            "wss://relay.one",
            "--relay",
            "wss://relay.two",
        ])
        .expect("Args should parse");

        assert_eq!(args.path.as_deref(), Some(std::path::Path::new("shot.png")));
        assert_eq!(args.server.as_deref(), Some("https://example.com"));
        assert_eq!(
            args.nostr_sec.as_deref(),
            Some("nsec1example00000000000000000000000000000000000000000000000000000")
        );
        assert_eq!(
            args.relays,
            vec!["wss://relay.one".to_string(), "wss://relay.two".to_string()]
        );
    }

    #[test]
    fn icon_svg_round_trips_over_http() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let addr = listener.local_addr().expect("local addr");
        let uploaded = Arc::new(Mutex::new(None::<Vec<u8>>));
        let uploaded_for_server = Arc::clone(&uploaded);

        let server = thread::spawn(move || {
            for _ in 0..2 {
                let (mut stream, _) = listener.accept().expect("accept connection");
                handle_request(&mut stream, &uploaded_for_server).expect("handle request");
            }
        });

        let input_path = std::env::temp_dir().join(format!(
            "get_file_hash-screenshot-test-{}.svg",
            std::process::id()
        ));
        fs::write(&input_path, EMBEDDED_ICON_SVG).expect("write embedded icon svg");

        let upload_status = std::process::Command::new("curl")
            .args([
                "--silent",
                "--fail",
                "--location",
                "-X",
                "POST",
                &format!("http://{addr}/upload"),
                "--data-binary",
                &format!("@{}", input_path.display()),
                "-H",
                "Content-Type: image/svg+xml",
            ])
            .status()
            .expect("run upload curl");
        assert!(upload_status.success(), "upload curl failed: {upload_status}");

        let download = std::process::Command::new("curl")
            .args([
                "--silent",
                "--fail",
                "--location",
                &format!("http://{addr}/image.svg"),
            ])
            .output()
            .expect("run download curl");
        assert!(download.status.success(), "download curl failed: {}", download.status);
        assert_eq!(download.stdout.as_slice(), EMBEDDED_ICON_SVG);
        let output_dir = std::env::current_dir().expect("current dir");
        let saved_path = timestamped_icon_output_path_for_content_type(&output_dir, "image/svg+xml");
        fs::write(&saved_path, &download.stdout).expect("save returned svg icon");
        assert_eq!(fs::read(&saved_path).expect("read saved svg icon"), EMBEDDED_ICON_SVG);
        fs::remove_file(&saved_path).ok();

        fs::remove_file(&input_path).ok();
        server.join().expect("server thread");
    }

    #[test]
    fn png_round_trips_over_http() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let addr = listener.local_addr().expect("local addr");
        let uploaded = Arc::new(Mutex::new(None::<Vec<u8>>));
        let uploaded_for_server = Arc::clone(&uploaded);

        let server = thread::spawn(move || {
            for _ in 0..2 {
                let (mut stream, _) = listener.accept().expect("accept connection");
                handle_request(&mut stream, &uploaded_for_server).expect("handle request");
            }
        });

        let input_path = std::env::temp_dir().join(format!(
            "get_file_hash-screenshot-test-{}.png",
            std::process::id()
        ));
        fs::write(&input_path, EMBEDDED_PLACEHOLDER_PNG).expect("write embedded placeholder png");

        let upload_status = std::process::Command::new("curl")
            .args([
                "--silent",
                "--fail",
                "--location",
                "-X",
                "POST",
                &format!("http://{addr}/upload"),
                "--data-binary",
                &format!("@{}", input_path.display()),
                "-H",
                "Content-Type: image/png",
            ])
            .status()
            .expect("run upload curl");
        assert!(upload_status.success(), "upload curl failed: {upload_status}");

        let download = std::process::Command::new("curl")
            .args([
                "--silent",
                "--fail",
                "--location",
                &format!("http://{addr}/image.png"),
            ])
            .output()
            .expect("run download curl");
        assert!(download.status.success(), "download curl failed: {}", download.status);
        assert_eq!(download.stdout.as_slice(), EMBEDDED_PLACEHOLDER_PNG);
        let output_dir = std::env::current_dir().expect("current dir");
        let saved_path = timestamped_icon_output_path_for_content_type(&output_dir, "image/png");
        fs::write(&saved_path, &download.stdout).expect("save returned png icon");
        assert_eq!(fs::read(&saved_path).expect("read saved png icon"), EMBEDDED_PLACEHOLDER_PNG);
        fs::remove_file(&saved_path).ok();

        fs::remove_file(&input_path).ok();
        server.join().expect("server thread");
    }

    #[test]
    fn real_icon_png_round_trips_over_http() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let addr = listener.local_addr().expect("local addr");
        let uploaded = Arc::new(Mutex::new(None::<Vec<u8>>));
        let uploaded_for_server = Arc::clone(&uploaded);

        let server = thread::spawn(move || {
            for _ in 0..2 {
                let (mut stream, _) = listener.accept().expect("accept connection");
                handle_request(&mut stream, &uploaded_for_server).expect("handle request");
            }
        });

        let input_path = std::env::temp_dir().join(format!(
            "get_file_hash-real-icon-test-{}.png",
            std::process::id()
        ));
        fs::write(&input_path, EMBEDDED_ICON_PNG).expect("write embedded icon png");

        let upload_status = std::process::Command::new("curl")
            .args([
                "--silent",
                "--fail",
                "--location",
                "-X",
                "POST",
                &format!("http://{addr}/upload"),
                "--data-binary",
                &format!("@{}", input_path.display()),
                "-H",
                "Content-Type: image/png",
            ])
            .status()
            .expect("run upload curl");
        assert!(upload_status.success(), "upload curl failed: {upload_status}");

        let download = std::process::Command::new("curl")
            .args([
                "--silent",
                "--fail",
                "--location",
                &format!("http://{addr}/image.png"),
            ])
            .output()
            .expect("run download curl");
        assert!(download.status.success(), "download curl failed: {}", download.status);
        assert_eq!(download.stdout.as_slice(), EMBEDDED_ICON_PNG);
        let output_dir = std::env::current_dir().expect("current dir");
        let saved_path = timestamped_icon_output_path_for_content_type(&output_dir, "image/png");
        fs::write(&saved_path, &download.stdout).expect("save returned icon png");
        assert_eq!(fs::read(&saved_path).expect("read saved icon png"), EMBEDDED_ICON_PNG);
        fs::remove_file(&saved_path).ok();

        fs::remove_file(&input_path).ok();
        server.join().expect("server thread");
    }

    #[tokio::test]
    async fn live_network_round_trip_uploads_and_returns_url() {
        let server = match std::env::var("NIP96_LIVE_TEST_SERVER") {
            Ok(value) => value,
            Err(_) => return,
        };

        println!("LIVE NIP-96 TEST SERVER: {server}");

        let file_path = std::env::temp_dir().join(format!(
            "get_file_hash-live-screenshot-{}.svg",
            std::process::id()
        ));
        fs::write(&file_path, EMBEDDED_ICON_SVG).expect("write live test svg");

        let keys = Keys::generate();
        let server_url = Url::parse(&server).expect("parse live test server URL");
        let config_url = nip96::get_server_config_url(&server_url).expect("server config url");

        let config_output = Command::new("curl")
            .args(["--silent", "--fail", "--location", config_url.as_str()])
            .output()
            .expect("fetch live server config");
        assert!(
            config_output.status.success(),
            "live server config fetch failed: {}",
            String::from_utf8_lossy(&config_output.stderr)
        );

        let config = nip96::ServerConfig::from_json(&config_output.stdout).expect("parse config");
        let upload_request = nip96::UploadRequest::new(&keys, &config, EMBEDDED_ICON_SVG)
            .await
            .expect("build upload request");

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
                &format!("file=@{};type={}", file_path.display(), EMBEDDED_ICON_SVG_CONTENT_TYPE),
            ])
            .output()
            .expect("run live upload curl");
        assert!(
            upload_output.status.success(),
            "live upload failed: {}",
            String::from_utf8_lossy(&upload_output.stderr)
        );

        let response = nip96::UploadResponse::from_json(&upload_output.stdout).expect("upload response");
        let download_url = response.download_url().expect("download url");
        println!("LIVE UPLOAD DOWNLOAD URL: {download_url}");

        let download = Command::new("curl")
            .args(["--silent", "--fail", "--location", download_url.as_str()])
            .output()
            .expect("run live download curl");
        assert!(
            download.status.success(),
            "live download failed: {}",
            String::from_utf8_lossy(&download.stderr)
        );
        assert_eq!(download.stdout.as_slice(), EMBEDDED_ICON_SVG);
        let output_dir = std::env::current_dir().expect("current dir");
        let saved_path = timestamped_icon_output_path_for_content_type(&output_dir, "image/svg+xml");
        fs::write(&saved_path, &download.stdout).expect("save returned live svg icon");
        assert_eq!(fs::read(&saved_path).expect("read saved live svg icon"), EMBEDDED_ICON_SVG);
        fs::remove_file(&saved_path).ok();
        println!("LIVE RELAY ONE-LINE: uploaded screenshot bytes and fetched them back unchanged");

        fs::remove_file(&file_path).ok();
    }

    fn handle_request(
        stream: &mut TcpStream,
        uploaded: &Arc<Mutex<Option<Vec<u8>>>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (method, path, body) = read_http_request(stream)?;
        match (method.as_str(), path.as_str()) {
            ("POST", "/upload") => {
                *uploaded.lock().expect("lock upload buffer") = Some(body);
                write_http_response(stream, 200, "text/plain", b"ok")?;
            }
            ("GET", "/image.svg") => {
                let svg = uploaded
                    .lock()
                    .expect("lock upload buffer")
                    .clone()
                    .expect("uploaded svg available");
                write_http_response(stream, 200, "image/svg+xml", &svg)?;
            }
            ("GET", "/image.png") => {
                let png = uploaded
                    .lock()
                    .expect("lock upload buffer")
                    .clone()
                    .expect("uploaded png available");
                write_http_response(stream, 200, "image/png", &png)?;
            }
            _ => {
                write_http_response(stream, 404, "text/plain", b"not found")?;
            }
        }
        Ok(())
    }

    fn read_http_request(
        stream: &mut TcpStream,
    ) -> Result<(String, String, Vec<u8>), Box<dyn std::error::Error>> {
        let mut buf = Vec::new();
        let mut header_end = None;
        loop {
            let mut chunk = [0u8; 1024];
            let n = stream.read(&mut chunk)?;
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&chunk[..n]);
            if let Some(pos) = find_subslice(&buf, b"\r\n\r\n") {
                header_end = Some(pos + 4);
                break;
            }
        }

        let header_end = header_end.expect("request headers");
        let headers = String::from_utf8(buf[..header_end].to_vec())?;
        let mut content_length = 0usize;
        let mut lines = headers.lines();
        let request_line = lines.next().ok_or("missing request line")?;
        for line in lines {
            if let Some(value) = line.strip_prefix("Content-Length: ") {
                content_length = value.trim().parse()?;
            }
        }

        while buf.len() < header_end + content_length {
            let mut chunk = [0u8; 1024];
            let n = stream.read(&mut chunk)?;
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&chunk[..n]);
        }

        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap_or_default().to_string();
        let path = parts.next().unwrap_or_default().to_string();
        Ok((method, path, buf[header_end..header_end + content_length].to_vec()))
    }

    fn write_http_response(
        stream: &mut TcpStream,
        status: u16,
        content_type: &str,
        body: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let reason = match status {
            200 => "OK",
            404 => "Not Found",
            _ => "OK",
        };
        write!(
            stream,
            "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )?;
        stream.write_all(body)?;
        stream.flush()?;
        Ok(())
    }

    fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack.windows(needle.len()).position(|window| window == needle)
    }
}
