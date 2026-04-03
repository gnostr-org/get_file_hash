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
## NIP-34 Integration: Git Repository Events on Nostr

This library provides a set of powerful macros and functions for integrating Git repository events with the Nostr protocol, adhering to the [NIP-34: Git Repositories on Nostr](https://github.com/nostr-protocol/nips/blob/master/34.md) specification.

These tools allow you to publish various Git-related events to Nostr relays, enabling decentralized tracking and collaboration for your code repositories.

### Available NIP-34 Macros

Each macro provides a convenient way to publish specific NIP-34 event kinds:

*   [`repository_announcement!`](#repository_announcement)
    *   Publishes a `Repository Announcement` event (Kind 30617) to announce a new or updated Git repository.
*   [`publish_patch!`](#publish_patch)
    *   Publishes a `Patch` event (Kind 1617) containing a Git patch (diff) for a specific commit.
*   [`publish_pull_request!`](#publish_pull_request)
    *   Publishes a `Pull Request` event (Kind 1618) to propose changes and facilitate code review.
*   [`publish_pr_update!`](#publish_pr_update)
    *   Publishes a `Pull Request Update` event (Kind 1619) to update an existing pull request.
*   [`publish_repository_state!`](#publish_repository_state)
    *   Publishes a `Repository State` event (Kind 1620) to announce the current state of a branch (e.g., its latest commit).
*   [`publish_issue!`](#publish_issue)
    *   Publishes an `Issue` event (Kind 1621) to report bugs, request features, or track tasks.

### Running NIP-34 Examples

To see these macros in action, navigate to the `examples/` directory and run each example individually with the `nostr` feature enabled:

```bash
cargo run --example repository_announcement --features nostr
cargo run --example publish_patch --features nostr
cargo run --example publish_pull_request --features nostr
cargo run --example publish_pr_update --features nostr
cargo run --example publish_repository_state --features nostr
cargo run --example publish_issue --features nostr
```

*   **SHA-256 Hash:** 6c6325c5a4c14f44cbda6ca53179ab3d6666ce7c916365668c6dd1d79215db59
*   **Status:** Integrity Verified..

##

## [`build.rs`](build.rs)

*   **Target File:** `build.rs`
*   **SHA-256 Hash:** 20c958c8cbb5c77cf5eb3763b6da149b61241d328df52d39b7aa97903305c889
*   **Status:** Integrity Verified..

##

## [`Cargo.toml`](Cargo.toml)

*   **Target File:** `Cargo.toml`
*   **SHA-256 Hash:** 94d00044d1e916aafcf6d58720d37874e35ab3bb2bdbc15959a8bfd1370c3d3e
*   **Status:** Integrity Verified..

##

## [`src/lib.rs`](src/lib.rs)

*   **Target File:** `src/lib.rs`
*   **SHA-256 Hash:** 591593482a6c9aac8793aa1e488e613f52a4effb1ec3465fd9d6a54537f2b123
*   **Status:** Integrity Verified..

