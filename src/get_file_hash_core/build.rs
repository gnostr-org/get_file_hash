#[cfg(all(not(debug_assertions), feature = "nostr"))]
#[tokio::main]
async fn main() {
    use std::fs;
    use std::path::PathBuf;

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let dest_path = out_dir.join("online_relays_gps.csv");

    // Only download if the file doesn't exist or is empty
    if !dest_path.exists() || fs::metadata(&dest_path).map(|m| m.len() == 0).unwrap_or(true) {
        println!("cargo:warning=Downloading online_relays_gps.csv...");
        let url = "https://raw.githubusercontent.com/permissionlesstech/bitchat/main/relays/online_relays_gps.csv";
        match reqwest::get(url).await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.text().await {
                        Ok(content) => {
                            fs::write(&dest_path, content).expect("Unable to write online_relays_gps.csv");
                            println!("cargo:warning=Successfully downloaded online_relays_gps.csv to {:?}", dest_path);
                        },
                        Err(e) => {
                            println!("cargo:warning=Failed to get text from response: {}", e);
                        }
                    }
                } else {
                    println!("cargo:warning=Failed to download online_relays_gps.csv: HTTP status {}", response.status());
                }
            },
            Err(e) => {
                println!("cargo:warning=Failed to fetch online_relays_gps.csv: {}", e);
            }
        }
    }
}

#[cfg(not(all(not(debug_assertions), feature = "nostr")))]
fn main() {
    // Placeholder for when the nostr feature is not enabled or in debug mode
    println!("cargo:warning=Skipping online_relays_gps.csv download (nostr feature not enabled or debug mode)");
}
