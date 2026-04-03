use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "gnostr-frost")]
#[command(version = "0.1.0")]
#[command(about = "BIP-64MOD + GCC Threshold Signature Tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Step 1: Generate a new T-of-N key set (Dealer Mode)
    Keygen {
        #[arg(long, default_value_t = 2)]
        threshold: u16,
        #[arg(long, default_value_t = 3)]
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
    /// Step 3: Sign a message hash using a vaulted nonce index
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
    /// Step 4: Aggregate shares into a final BIP-340 signature
    Aggregate {
        #[arg(short, long)]
        message: String,
        #[arg(required = true)]
        shares: Vec<String>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Keygen { threshold, total, output_dir: _ } => {
            println!("🛠️  Executing Keygen: {}-of-{}...", threshold, total);
            // logic: generate_with_dealer(...) -> write files
        }
        Commands::Batch { count, key: _ } => {
            println!("📦 Executing Batch: Generating {} nonces...", count);
            // logic: load key -> round1::commit() -> update vault
        }
        Commands::Sign { message, index, key: _, vault: _ } => {
            println!("✍️  Executing Sign: Index #{} for '{}'...", index, message);
            // logic: load key + vault -> round2::sign() -> purge index
        }
        Commands::Aggregate { message, shares } => {
            println!("🧬 Executing Aggregate: {} shares for '{}'...", shares.len(), message);
            // logic: round2::SignatureShare::deserialize() -> frost::aggregate()
        }
    }

    Ok(())
}
