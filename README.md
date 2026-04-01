# `get_file_hash` macro

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

## Release
## [`README.md`](./README.md)

```bash
cargo run --bin readme > README.md
```

## [`src/bin/readme.rs`](src/bin/readme.rs)

*   **Target File:** `src/bin/readme.rs`
*   **SHA-256 Hash:** b337cb82aa8840ce7fcbcfbc9a0c2d2542d3eb1f2ab6358efba2d2f7a5af730c
*   **Status:** Integrity Verified..

##

## [`build.rs`](build.rs)

*   **Target File:** `build.rs`
*   **SHA-256 Hash:** a226f2e3757ba9cd4839c4082dc9755a3fb9e827d3278bcc78e36bd77ddf5bd7
*   **Status:** Integrity Verified..

##

## [`Cargo.toml`](Cargo.toml)

*   **Target File:** `Cargo.toml`
*   **SHA-256 Hash:** 5e1812eec540dfaaa0cc628dff96b1193527faa86b11f9f5a8222a63816d17cc
*   **Status:** Integrity Verified..

##

## [`src/lib.rs`](src/lib.rs)

*   **Target File:** `src/lib.rs`
*   **SHA-256 Hash:** e8f219b4d6c88a33989a8c1b43dcfbe244d89e81fe5e29a64e808fd534702e81
*   **Status:** Integrity Verified..

