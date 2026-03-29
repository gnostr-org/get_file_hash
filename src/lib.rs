
//! A crate providing the `get_file_hash!` procedural macro.
//!
//! This macro allows you to compute the SHA-256 hash of a file at compile time,
//! embedding the resulting hash string directly into your Rust executable.
//! This is useful for integrity checks, versioning, or embedding unique identifiers.

/// Computes the SHA-256 hash of the specified file at compile time.
///
/// This macro takes a string literal representing a file path, reads the file's bytes
/// at compile time, computes its SHA-256 hash, and returns the hash as a hex-encoded `String`.
///
/// # Examples
///
/// ```rust
/// use get_file_hash::get_file_hash;
/// use std::fs;
///
/// // Assume 'my_file.txt' exists in the project root with content 'hello world'
/// // In a real scenario, you'd create this file for testing.
/// let hash = get_file_hash!("src/lib.rs");
/// println!("Hash: {}", hash);
/// ```
#[macro_export]
macro_rules! get_file_hash {
    ($file_path:expr) => {{
        let bytes = include_bytes!($file_path);
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let result = hasher.finalize();

        // Convert the GenericArray to a hex string
        result
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    }};
}

/// Unit tests for the `get_file_hash!` macro.
#[cfg(test)
mod tests {
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
