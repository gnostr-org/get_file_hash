
//! A crate providing the `get_file_hash!` procedural macro.
//!
//! This macro allows you to compute the SHA-256 hash of a file at compile time,
//! embedding the resulting hash string directly into your Rust executable.
//! This is useful for integrity checks, versioning, or embedding unique identifiers.

pub use get_file_hash_core::get_file_hash;
#[cfg(test)]
mod tests {
    use crate::get_file_hash;
    use sha2::{Digest, Sha256};

    /// Tests that the `get_file_hash!` macro correctly computes the SHA-256 hash
    /// of `lib.rs` and that it matches a manually computed hash of the same file.
    #[test]
    fn test_get_file_hash() {
        let file_content = include_bytes!("lib.rs");

        let mut hasher = Sha256::new();
        hasher.update(file_content);
        let expected_hash = hasher.finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        let actual_hash = get_file_hash!("lib.rs");
        assert_eq!(actual_hash, expected_hash);
    }
}
