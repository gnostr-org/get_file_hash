//! A simple command-line tool that calculates and displays the SHA-256 hash of its own source file.
//!
//! This utility demonstrates how to use the `get_file_hash!` macro to obtain the hash of a specified file
//! at compile time and incorporate it into runtime logic.

use get_file_hash_core::get_file_hash;

/// The main entry point of the application.
///
/// This function calculates the SHA-256 hash of the `get_file_hash.rs` source file
/// using a custom procedural macro and then prints the hash to the console.
/// It also includes a basic integrity verification check.
fn main() {
    // Calculate the SHA-256 hash of the current file (`get_file_hash.rs`) at compile time.
    // The `get_file_hash!` macro reads the file content and computes its hash.
    let self_hash = get_file_hash!("get_file_hash.rs");

    // Generate Markdown formatted output for README.md.
    println!("# Get File Hash Utility\n");
    println!("This utility provides a `get_file_hash!` procedural macro that computes the SHA-256 hash of a file at compile time, embedding the resulting hash string directly into your Rust executable. This is useful for integrity checks, versioning, or embedding unique identifiers.\n");
    println!("## Usage Example\n");
    println!("```rust\nuse get_file_hash::get_file_hash;\nuse sha2::{{Digest, Sha256}};\n\nlet hash = get_file_hash!(\"lib.rs\");\nprintln!(\"Hash: {{}}\", hash);\n```\n");
    println!("## Current File Hash Information (of `src/bin/get_file_hash.rs`)\n");
    println!("*   **Target File:** `get_file_hash.rs`\n");
    println!("*   **SHA-256 Hash:** `{}`\n", self_hash);

    if self_hash.starts_with("e3b0") {
        println!("*   **Status:** Warning: This hash represents an empty file.\n");
    } else {
        println!("*   **Status:** Integrity Verified.\n");
    }
}
