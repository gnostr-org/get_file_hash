use clap::{Parser, Subcommand};
use frost_secp256k1_tr as frost;
use frost::round1::{self, SigningCommitments, SigningNonces};
use frost::keys::IdentifierList;
use rand_chacha::ChaCha20Rng;
use rand::SeedableRng;
use std::fs;
use std::path::PathBuf;
use std::collections::BTreeMap;

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

type NonceMap = BTreeMap<u32, SigningNonces>;
type CommitmentMap = BTreeMap<u32, SigningCommitments>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Keygen { threshold, total, output_dir } => {
            println!("🛠️  Executing Keygen: {}-of-{}...", threshold, total);

            let mut rng = ChaCha20Rng::from_entropy(); 

            let (shares, pubkey_package) = frost::keys::generate_with_dealer(
                *total, *threshold, IdentifierList::Default, &mut rng
            )?;

            let path = output_dir.as_deref().unwrap_or(std::path::Path::new("."));

            let pub_path = path.join("group_public.json");
            fs::write(&pub_path, serde_json::to_string_pretty(&pubkey_package)?)?;
            println!("✅ Saved Group Public Key to {:?}", pub_path);

            for (id, share) in shares {
                // KeyPackage is accessed via frost::keys::KeyPackage
                let key_pkg = frost::keys::KeyPackage::new(
                    id,
                    *share.signing_share(),
                    frost::keys::VerifyingShare::from(*share.signing_share()),
                    *pubkey_package.verifying_key(),
                    *threshold,
                );

                let id_hex = hex::encode(id.serialize());
                let file_name = format!("p{}_key.json", id_hex);
                let share_path = path.join(file_name);

                fs::write(&share_path, serde_json::to_string_pretty(&key_pkg)?)?;
                println!("✅ Saved KeyPackage for Participant {:?} to {:?}", id, share_path);
            }
        }
        Commands::Batch { count, key } => {
            println!("📦 Executing Batch: Generating {} nonces...", count);

            let key_json = fs::read_to_string(key)?;
            let key_pkg: frost::keys::KeyPackage = serde_json::from_str(&key_json)?;

            let mut rng = ChaCha20Rng::from_entropy();

            let mut public_commitments: CommitmentMap = BTreeMap::new();
            let mut secret_nonce_vault: NonceMap = BTreeMap::new();

            for i in 0..*count {
                let (nonces, commitments) = round1::commit(key_pkg.signing_share(), &mut rng);
                public_commitments.insert(i as u32, commitments);
                secret_nonce_vault.insert(i as u32, nonces);
            }

            let vault_path = "p1_batch_vault.json";
            fs::write(vault_path, serde_json::to_string(&secret_nonce_vault)?)?;

            println!("✅ Generated and vaulted {} nonces.", count);
            println!("📋 Public Commitments (Indices 0 to {}):", count - 1);
            for (idx, comms) in public_commitments {
                println!("  Index {}: {}", idx, hex::encode(comms.serialize()?));
            }
        }
        Commands::Sign { message, index, key: _, vault: _ } => {
            println!("✍️  Executing Sign: Index #{} for '{}'...", index, message);
        }
        Commands::Aggregate { message, shares } => {
            println!("🧬 Executing Aggregate: {} shares for '{}'...", shares.len(), message);
        }
    }

    Ok(())
}
