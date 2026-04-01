/// deterministic nostr event build example
// deterministic nostr event build example
use get_file_hash_core::get_file_hash;
use sha2::{Digest, Sha256};

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let is_git_repo = std::path::Path::new(&manifest_dir).join(".git").exists();

    if !is_git_repo {
        println!("cargo:rustc-cfg=is_published_source");
    }

    let cargo_toml_hash = get_file_hash!("Cargo.toml");
    println!("cargo:rustc-env=CARGO_TOML_HASH={}", cargo_toml_hash);

    let lib_hash = get_file_hash!("src/lib.rs");
    println!("cargo:rustc-env=LIB_HASH={}", lib_hash);

    let build_hash = get_file_hash!("build.rs");
    println!("cargo:rustc-env=BUILD_HASH={}", build_hash);

    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=build.rs");

    // Prevent `cargo build` from outputting warnings about `rustc-cfg` if no `.git` directory is found.
    println!("cargo:rerun-if-changed=ALWAYS_RUN_NONEXISTENT_FILE");
}
// deterministic nostr event build example
