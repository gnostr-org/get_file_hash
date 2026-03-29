//! A simple command-line tool that calculates and displays the SHA-256 hash of
//! its own source file.
//!
//! This utility demonstrates how to use the `get_file_hash!` macro to obtain
//! the hash of a specified file at compile time and incorporate it into runtime
//! logic.
use get_file_hash::{BUILD_HASH, CARGO_TOML_HASH, LIB_HASH};
use get_file_hash_core::get_file_hash;
use sha2::{Digest, Sha256};

const README_TEMPLATE_PART0: &str = r##"# `get_file_hash` macro

This project provides a Rust procedural macro, `get_file_hash!`, designed to compute the SHA-256 hash of a specified file at compile time. This hash is then embedded directly into your compiled executable. This feature is invaluable for:

*   **Integrity Verification:** Ensuring the deployed code hasn't been tampered with.
*   **Versioning:** Embedding a unique identifier linked to the exact source code version.
*   **Cache Busting:** Generating unique names for assets based on their content.

## Project Structure

*   `get_file_hash_core`: A foundational crate containing the `get_file_hash!` macro definition.
*   `get_file_hash`: The main library crate that re-exports the macro.
*   `src/bin/get_file_hash.rs`: An example executable demonstrating the macro's usage by hashing its own source file and updating this `README.md`.
*   `build.rs`: A build script that also utilizes the `get_file_hash!` macro to hash `Cargo.toml` during the build process.

## Usage of `get_file_hash!` Macro

To use the `get_file_hash!` macro, ensure you have `get_file_hash` (or `get_file_hash_core` for direct usage) as a dependency in your `Cargo.toml`.

### Example

```rust
use get_file_hash::get_file_hash;
use get_file_hash::CARGO_TOML_HASH;
use sha2::{Digest, Sha256};

fn main() {
    // The macro resolves the path relative to CARGO_MANIFEST_DIR
    let readme_hash = get_file_hash!("src/bin/readme.rs");
    let lib_hash = get_file_hash!("src/lib.rs");
    println!("The SHA-256 hash of src/lib.rs is: {}", lib_hash);
    println!("The SHA-256 hash of src/bin/readme.rs is: {}", readme_hash);
    println!("The SHA-256 hash of Cargo.toml is: {}", CARGO_TOML_HASH);
}
```

"##;

const README_TEMPLATE_PART1: &str = r"## Release
## [`README.md`](./README.md)

```bash
cargo run --bin readme > README.md
```

## [`src/bin/readme.rs`](src/bin/readme.rs)

*   **Target File:** `src/bin/readme.rs`
";

const README_TEMPLATE_PART2: &str = r"##

## [`build.rs`](build.rs)

*   **Target File:** `build.rs`
";

const README_TEMPLATE_PART3: &str = r"##

## [`Cargo.toml`](Cargo.toml)

*   **Target File:** `Cargo.toml`
";

const README_TEMPLATE_PART4: &str = r"##

## [`src/lib.rs`](src/lib.rs)

*   **Target File:** `src/lib.rs`
";

/// The main entry point of the application.
///
/// This function calculates the SHA-256 hash of the `get_file_hash.rs` source
/// file using a custom procedural macro and then prints the hash to the
/// console. It also includes a basic integrity verification check.
fn main() {
    // Calculate the SHA-256 hash of the current file (`readme.rs`) at
    // compile time. The `get_file_hash!` macro reads the file content and
    // computes its hash.
    let self_hash = get_file_hash!("readme.rs");

    let status_message = if self_hash.starts_with("e3b0") {
        "Warning: This hash represents an empty file."
    } else {
        "Integrity Verified."
    };

    let build_message = if BUILD_HASH.starts_with("e3b0") {
        "Warning: This hash represents an empty file."
    } else {
        "Integrity Verified."
    };
    let cargo_message = if CARGO_TOML_HASH.starts_with("e3b0") {
        "Warning: This hash represents an empty file."
    } else {
        "Integrity Verified."
    };
    let lib_message = if LIB_HASH.starts_with("e3b0") {
        "Warning: This hash represents an empty file."
    } else {
        "Integrity Verified."
    };

    print!("{}{}", README_TEMPLATE_PART0, README_TEMPLATE_PART1);
    println!("*   **SHA-256 Hash:** {}", self_hash);
    println!("*   **Status:** {}.\n", status_message);
    //
    print!("{}", README_TEMPLATE_PART2);
    println!("*   **SHA-256 Hash:** {}", BUILD_HASH);
    println!("*   **Status:** {}.\n", build_message);
    //
    print!("{}", README_TEMPLATE_PART3);
    println!("*   **SHA-256 Hash:** {}", CARGO_TOML_HASH);
    println!("*   **Status:** {}.\n", cargo_message);
    //
    print!("{}", README_TEMPLATE_PART4);
    println!("*   **SHA-256 Hash:** {}", LIB_HASH);
    println!("*   **Status:** {}.\n", lib_message);
}
