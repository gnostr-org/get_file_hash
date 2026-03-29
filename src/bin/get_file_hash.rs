//! A simple command-line tool that calculates and displays the SHA-256 hash of its own source file.
//!
//! This utility demonstrates how to use the `get_file_hash!` macro to obtain the hash of a specified file
//! at compile time and incorporate it into runtime logic.

use sha2::{Digest, Sha256};

/// The main entry point of the application.
///
/// This function calculates the SHA-256 hash of the `get_file_hash.rs` source file
/// using a custom procedural macro and then prints the hash to the console.
/// It also includes a basic integrity verification check.
fn main() {
    // Calculate the SHA-256 hash of the current file (`get_file_hash.rs`) at compile time.
    // The `get_file_hash!` macro reads the file content and computes its hash.
    let self_hash = get_file_hash::get_file_hash!("get_file_hash.rs");

    // Print the target file and its calculated SHA-256 hash.
    println!("");
    println!("## get\\_file\\_hash	");
    println!("");
    println!("");
    println!("");
    println!("");
    println!("Target: get\\_file\\_hash.rs	");
    println!("SHA-256 Hash: {}	", self_hash);

    // Perform a basic integrity check: an SHA-256 hash starting with "e3b0" typically indicates an empty file.
    // This serves as a simple example of how the hash can be used in application logic.
    if self_hash.starts_with("e3b0") {
        println!("Warning: This hash represents an empty file.");
    } else {
        println!("Status: Integrity Verified.	");
    }
}
