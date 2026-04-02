# `build.rs` Documentation

This document explains the functionality of the `build.rs` script in this project. The `build.rs` script is a special Rust file that, if present, Cargo will compile and run *before* compiling the rest of your package. It's typically used for tasks that need to be performed during the build process, such as generating code, setting environment variables, or performing conditional compilation.

## Core Functionality

The `build.rs` script in this project performs the following key functions:

1.  **Environment Variable Injection:** It computes various project-related values at compile time and injects them as environment variables (`CARGO_RUSTC_ENV=...`) that can be accessed by the main crate using `env!("VAR_NAME")`. This includes:
    *   `CARGO_PKG_NAME`: The name of the current package (from `Cargo.toml`).
    *   `CARGO_PKG_VERSION`: The version of the current package (from `Cargo.toml`).
    *   `GIT_COMMIT_HASH`: The full commit hash of the current Git HEAD (if in a Git repository).
    *   `GIT_BRANCH`: The name of the current Git branch (if in a Git repository).
    *   `CARGO_TOML_HASH`: The SHA-256 hash of the `Cargo.toml` file.
    *   `LIB_HASH`: The SHA-256 hash of the `src/lib.rs` file.
    *   `BUILD_HASH`: The SHA-256 hash of the `build.rs` file itself.

2.  **Rerun Conditions:** It tells Cargo when to re-run the build script. This ensures that the injected environment variables and any conditional compilation logic are up-to-date if relevant files change:
    *   `Cargo.toml`
    *   `src/lib.rs`
    *   `build.rs`
    *   `.git/HEAD` (to detect changes in the Git repository like new commits or branch switches).
    *   `src/get_file_hash_core/src/online_relays_gps.csv` (conditionally, if the file exists).

3.  **Conditional Nostr Event Publishing (Release Builds with `nostr` feature):**
    If the project is being compiled in **release mode (`--release`)** and the **`nostr` feature is enabled (`--features nostr`)**, the `build.rs` script will connect to Nostr relays and publish events. This is intended for "deterministic Nostr event build examples" as indicated by the comments in the file.

    *   **Relay Management:** It retrieves a list of default relay URLs. During event publishing, it identifies and removes "unfriendly" or unresponsive relays (e.g., those with timeout, connection issues, or spam blocks) from the list for subsequent publications.
    *   **File Hashing and Key Generation:** For each Git-tracked file (when in a Git repository), it computes its SHA-256 hash. This hash is then used to derive a Nostr `SecretKey`.
    *   **Event Creation:**
        *   **Individual File Events:** For each Git-tracked file, a Nostr `text_note` event is created. This event includes tags for:
            *   `#file`: The path of the file.
            *   `#version`: The package version.
            *   `#commit`: The Git commit hash (if in a Git repository).
            *   `#branch`: The Git branch name (if in a Git repository).
        *   **Metadata Event:** It publishes a metadata event using `get_file_hash_core::publish_metadata_event`.
        *   **Linking Event (Build Manifest):** After processing all individual files, if any events were published, a final "build manifest" `text_note` event is created. This event links to all the individual file events that were published during the build using event tags.
    *   **Output Storage:** The JSON representation of successfully published Nostr events (specifically the `EventId`) is saved to `~/.gnostr/build/{package_version}/{file_path_str_sanitized}/{hash}/{public_key}/{event_id}.json`. This provides a local record of what was published.

### `publish_nostr_event_if_release` Function

This asynchronous helper function is responsible for:
*   Adding relays to the Nostr client.
*   Connecting to relays.
*   Signing the provided `EventBuilder` to create an `Event`.
*   Sending the event to the configured relays.
*   Logging success or failure for each relay.
*   Identifying and removing unresponsive relays from the `relay_urls` list.
*   Saving the published event's JSON to the local filesystem.

### `should_remove_relay` Function

This helper function determines if a relay should be considered "unfriendly" or unresponsive based on common error messages received during Nostr event publication.

## Usage

To prevent 'Too many open files' errors, especially during builds and tests involving numerous file operations or subprocesses (like `git ls-files` or parallel test execution), it may be necessary to increase the file descriptor limit.

*   **For local development**: Run `ulimit -n 4096` in your terminal session before executing `cargo build` or `cargo test`. This setting is session-specific.
*   **For CI environments**: The `.github/workflows/rust.yml` workflow is configured to set `ulimit -n 4096` for relevant test steps to ensure consistent execution.

The values set by `build.rs` can be accessed in your Rust code (e.g., `src/lib.rs`) at compile time using the `env!` macro. For example:
```rust
pub const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
```

The Nostr event publishing functionality of `build.rs` is primarily for release builds with the `nostr` feature enabled, allowing for the automatic, deterministic publication of project state to the Nostr network as part of the CI/CD pipeline.

## Example Commands

To interact with the `build.rs` script's features, especially those related to Nostr event publishing, you can use the following `cargo` commands:

*   **Build in release mode with Nostr feature (verbose output):**
    ```bash
    cargo build --release --workspace --features nostr -vv
    ```

*   **Run tests for `get_file_hash_core` sequentially with Nostr feature and verbose logging (as in CI):**
    ```bash
    RUST_LOG=info,nostr_sdk=debug,frost=debug cargo test -p get_file_hash_core --features nostr -- --test-threads 1 --nocapture
    ```

*   **Run all workspace tests in release mode with Nostr feature:**
    ```bash
    cargo test --workspace --release --features nostr
    ```

