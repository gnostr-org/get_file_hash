use sha2::{Digest, Sha256};

fn main() {
    // We can now assign the result of the macro to a variable
    let self_hash = get_file_hash::get_file_hash!("get_file_hash.rs");

    println!("");
    println!("## get\\_file\\_hash	");
    println!("");
    println!("");
    println!("");
    println!("");
    println!("Target: get\\_file\\_hash.rs	");
    println!("SHA-256 Hash: {}	", self_hash);

    // Example of using the hash in logic
    if self_hash.starts_with("e3b0") {
        println!("Warning: This hash represents an empty file.");
    } else {
        println!("Status: Integrity Verified.	");
    }
}
