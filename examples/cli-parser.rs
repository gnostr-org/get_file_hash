use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "gnostr-frost")]
#[command(about = "BIP-64MOD + GCC Threshold Signature Tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Step 1: Generate a new T-of-N key set (Dealer Mode)
    Keygen {
        #[arg(short, long, default_value_t = 2)]
        threshold: u16,
        #[arg(short, long, default_value_t = 3)]
        total: u16,
        #[arg(short, long)]
        output_dir: Option<PathBuf>,
    },
    /// Step 2: Generate a batch of public/private nonces
    Batch {
        #[arg(short, long, default_value_t = 10)]
        count: u16,
        #[arg(short, long, default_value = "p1_key.json")]
        key: PathBuf,
    },
    /// Step 3: Sign a specific message using a vaulted nonce index
    Sign {
        #[arg(short, long)]
        message: String,
        #[arg(short, long)]
        index: u64,
        #[arg(short, long, default_value = "p1_key.json")]
        key: PathBuf,
        #[arg(short, long, default_value = "p1_batch_vault.json")]
        vault: PathBuf,
    },
    /// Step 4: Aggregate partial signatures into a final BIP-340 signature
    Aggregate {
        #[arg(short, long)]
        message: String,
        /// Hex-encoded partial signatures (space separated)
        #[arg(required = true)]
        shares: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Keygen { threshold, total, output_dir } => {
            println!("Generating {}-of-{} FROST keys...", threshold, total);
            // Call logic from example-1/6
        }
        Commands::Batch { count, key } => {
            println!("Generating {} nonces using key: {:?}", count, key);
            // Call logic from example-8
        }
        Commands::Sign { message, index, key, vault } => {
            println!("Signing index {} for message: {}", index, message);
            // Call logic from example-7/8
        }
        Commands::Aggregate { message, shares } => {
            println!("Aggregating {} shares for message: {}", shares.len(), message);
            // Call logic from example-5
        }
    }
}
