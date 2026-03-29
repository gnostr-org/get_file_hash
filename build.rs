use sha2::{Digest, Sha256};

fn main() {
    let cargo_toml_hash = get_file_hash_core::get_file_hash!("Cargo.toml");
    println!("cargo:warning=Hash of Cargo.toml: {}", cargo_toml_hash);
}