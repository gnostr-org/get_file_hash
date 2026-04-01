/// deterministic nostr event build example
// deterministic nostr event build example
use get_file_hash_core::get_file_hash;
use sha2::{Digest, Sha256};

fn main() {
    println!("cargo::rustc-check-cfg=cfg(is_published_source)");
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let is_git_repo = std::path::Path::new(&manifest_dir).join(".git").exists();

    // Detect if this is a published source (e.g., from crates.io) vs. a git repository.
    // We consider it a published source if there's no .git directory.
    if !is_git_repo {
        println!("cargo:rustc-cfg=is_published_source");
    }

    println!("cargo:rustc-env=CARGO_PKG_NAME={}", env!("CARGO_PKG_NAME"));
    println!("cargo:rustc-env=CARGO_PKG_VERSION={}", env!("CARGO_PKG_VERSION"));

    if is_git_repo {
        let git_commit_hash = std::process::Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .output()
            .expect("Failed to execute git command for commit hash")
            .stdout;
        let git_commit_hash_str = String::from_utf8(git_commit_hash).unwrap();
        println!("cargo:rustc-env=GIT_COMMIT_HASH={}", git_commit_hash_str.trim());

        let git_branch = std::process::Command::new("git")
            .args(&["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .expect("Failed to execute git command for branch name")
            .stdout;
        let git_branch_str = String::from_utf8(git_branch).unwrap();
        println!("cargo:rustc-env=GIT_BRANCH={}", git_branch_str.trim());
    }

    println!("cargo:rerun-if-changed=.git/HEAD");

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
