use sha2::{Digest, Sha256};

// A macro that takes a file path string literal, hashes the embedded bytes
/// using SHA-256, and returns a hex-encoded String.
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

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
