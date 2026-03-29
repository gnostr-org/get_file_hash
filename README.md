# Get File Hash Utility

This utility provides a `get_file_hash!` procedural macro that computes the SHA-256 hash of a file at compile time, embedding the resulting hash string directly into your Rust executable. This is useful for integrity checks, versioning, or embedding unique identifiers.

## Usage Example

```rust
use get_file_hash::get_file_hash;
use sha2::{Digest, Sha256};

let hash = get_file_hash!("lib.rs");
println!("Hash: {}", hash);
```

## Current File Hash Information (of `src/bin/get_file_hash.rs`)

*   **Target File:** `get_file_hash.rs`

*   **SHA-256 Hash:** `3872374a2e666dec4817133a1a1ab31d888f31c1f8a6af78e3a0de1dddf515d3`

*   **Status:** Integrity Verified.

