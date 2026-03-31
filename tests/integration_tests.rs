use std::process::Command;
use std::fs;
use sha2::{Digest, Sha256};

fn calculate_sha256(file_path: &str) -> String {
    let content = fs::read(file_path).expect("Unable to read file");
    let mut hasher = Sha256::new();
    hasher.update(&content);
    hasher.finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

#[test]
fn test_get_file_hash_binary_no_features() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("get_file_hash")
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Assert that the command ran successfully
    assert!(output.status.success(), "Command failed with stderr: {}", stderr);

    // Manually calculate the hash of the binary's source file
    let expected_hash = calculate_sha256("src/bin/get_file_hash.rs");

    // Assert that the output contains the correct hash
    // Check for the raw hash first
    assert!(stdout.contains(&expected_hash), "Output did not contain raw expected hash. Expected: {}, Actual: {}", expected_hash, stdout);

    // Then check for the formatted string, including backticks
    // Use a regex-like check for more flexibility with newlines if needed, or refine to exact match
    let expected_hash_line = format!("*   **SHA-256 Hash:** `{}`", expected_hash);
    assert!(stdout.contains(&expected_hash_line), "Output did not contain expected hash line. Expected line: {}, Actual: {}", expected_hash_line, stdout);

    // Assert that the output contains "Integrity Verified."
    assert!(stdout.contains("Integrity Verified."), "Output did not contain 'Integrity Verified.'. stdout: {}", stdout);

    println!("Output from get_file_hash binary (no features):
{}", stdout);
}

#[test]
fn test_get_file_hash_binary_with_nostr_feature() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("get_file_hash")
        .arg("--features")
        .arg("nostr")
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Assert that the command ran successfully
    assert!(output.status.success(), "Command failed with stderr: {}", stderr);

    // Manually calculate the hash of the binary's source file
    let expected_hash = calculate_sha256("src/bin/get_file_hash.rs");

    // Assert that the output contains the correct hash
    assert!(stdout.contains(&expected_hash), "Output did not contain raw expected hash. Expected: {}, Actual: {}", expected_hash, stdout);

    // Then check for the formatted string, including backticks
    let expected_hash_line = format!("*   **SHA-256 Hash:** `{}`", expected_hash);
    assert!(stdout.contains(&expected_hash_line), "Output did not contain expected hash line. Expected line: {}, Actual: {}", expected_hash_line, stdout);

    // Assert that the output contains "Integrity Verified."
    assert!(stdout.contains("Integrity Verified."), "Output did not contain 'Integrity Verified.'. stdout: {}", stdout);

    println!("Output from get_file_hash binary (with nostr feature):
{}", stdout);
}
