//! A crate providing the `get_file_hash!` procedural macro.
//!
//! This macro allows you to compute the SHA-256 hash of a file at compile time,
//! embedding the resulting hash string directly into your Rust executable.

pub use get_file_hash_core::get_file_hash;

#[cfg(feature = "nostr")]
use nostr_sdk::{Event, EventBuilder, EventId, Keys, Tag, UnsignedEvent};

#[cfg(feature = "nostr")]
const DEFAULT_MAX_POW_ATTEMPTS: u128 = 100_000_000;

/// Mine a Nostr event whose ID starts with the given lowercase-hex prefix.
///
/// The SDK's [`EventBuilder::pow`](nostr_sdk::EventBuilder::pow) only supports
/// NIP-13 leading-zero-bits difficulty. This helper does custom-prefix mining
/// by brute-forcing a `["nonce", "<nonce>", "0"]` tag until the computed
/// event ID matches the desired prefix.
///
/// # Arguments
///
/// * `keys` - The keypair used to sign the event.
/// * `builder` - A pre-configured event builder (content, tags, kind, ...).
/// * `prefix` - The desired hex prefix (case-insensitive). Each extra hex
///   character increases the expected search time by a factor of 16.
/// * `max_attempts` - Safety cap. Returns `None` if no matching nonce is found.
///
/// # Example
///
/// ```no_run
/// use get_file_hash::mine_event_with_prefix;
/// use nostr_sdk::{EventBuilder, Keys, Tag};
///
/// let keys = Keys::generate();
/// let builder = EventBuilder::text_note("hello")
///     .tag(Tag::parse(["example", "pow"]).unwrap());
///
/// if let Some(event) = mine_event_with_prefix(&keys, builder, "0000", 10_000_000) {
///     println!("mined event id: {}", event.id);
/// }
/// ```
#[cfg(feature = "nostr")]
pub fn mine_event_with_prefix(
    keys: &Keys,
    builder: EventBuilder,
    prefix: &str,
    max_attempts: u128,
) -> Option<Event> {
    let pubkey = keys.public_key();
    let prefix = prefix.to_ascii_lowercase();
    let mut nonce: u128 = 0;

    loop {
        let mut attempt = builder.clone();
        attempt = attempt.tag(Tag::pow(nonce, 0));

        let mut unsigned: UnsignedEvent = attempt.build(pubkey);
        let id: EventId = unsigned.id();

        if id.to_hex().starts_with(&prefix) {
            return unsigned.sign_with_keys(keys).ok();
        }

        nonce += 1;
        if nonce > max_attempts {
            return None;
        }
    }
}

/// Convenience wrapper around [`mine_event_with_prefix`] with a default attempt
/// cap of 100,000,000.
#[cfg(feature = "nostr")]
pub fn mine_event_with_prefix_or_default(
    keys: &Keys,
    builder: EventBuilder,
    prefix: &str,
) -> Option<Event> {
    mine_event_with_prefix(keys, builder, prefix, DEFAULT_MAX_POW_ATTEMPTS)
}

/// The SHA-256 hash of this crate's `build.rs` at the time of compilation.
pub const BUILD_HASH: &str = env!("BUILD_HASH");

/// The SHA-256 hash of this crate's `Cargo.toml` at the time of compilation.
pub const CARGO_TOML_HASH: &str = env!("CARGO_TOML_HASH");

/// The SHA-256 hash of this crate's `src/lib.rs` at the time of compilation.
pub const LIB_HASH: &str = env!("LIB_HASH");

/// The name of the package as specified in Cargo.toml.
pub const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");

/// The version of the package as specified in Cargo.toml.
pub const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(feature = "nostr")]
/// The git commit hash of the repository at the time of compilation.
pub const GIT_COMMIT_HASH: &str = env!("GIT_COMMIT_HASH");

#[cfg(feature = "nostr")]
/// The git branch of the repository at the time of compilation.
pub const GIT_BRANCH: &str = env!("GIT_BRANCH");

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};

    use super::*;

    /// Verifies that the exported CARGO_TOML_HASH is not empty.
    #[test]
    fn test_injected_hash_exists() {
        assert!(!BUILD_HASH.is_empty());
        println!("Verified build.rs Hash:
{}", BUILD_HASH);

        assert!(!CARGO_TOML_HASH.is_empty());
        println!("Verified Cargo.toml Hash:
{}", CARGO_TOML_HASH);

        assert!(!LIB_HASH.is_empty());
        println!("Verified src/lib.rs Hash:\n{}", LIB_HASH);

        assert!(!CARGO_PKG_NAME.is_empty());
        println!("Verified Package Name:\n{}", CARGO_PKG_NAME);

        assert!(!CARGO_PKG_VERSION.is_empty());
        println!("Verified Package Version:\n{}", CARGO_PKG_VERSION);

        #[cfg(feature = "nostr")]
        {
            assert!(!GIT_COMMIT_HASH.is_empty());
            println!("Verified Git Commit Hash:\n{}", GIT_COMMIT_HASH);

            assert!(!GIT_BRANCH.is_empty());
            println!("Verified Git Branch:\n{}", GIT_BRANCH);
        }
    }

    /// Tests that the `get_file_hash!` macro correctly computes the SHA-256
    /// hash of `lib.rs` and that it matches a manually computed hash of the
    /// same file.
    #[test]
    fn test_get_lib_hash() {
        let file_content = include_bytes!("lib.rs");

        let mut hasher = Sha256::new();
        hasher.update(file_content);
        let expected_hash = hasher
            .finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        let actual_hash = get_file_hash!("lib.rs");
        assert_eq!(actual_hash, expected_hash);
    }
}
