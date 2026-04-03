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
        #[arg(short, long)]
        key: PathBuf,
    },
    /// Step 3: Sign a message hash using a vaulted nonce index
    Sign {
        #[arg(short, long)]
        message: String,
        #[arg(short, long)]
        index: u64,
        #[arg(short, long)]
        key: PathBuf,
        #[arg(short, long)]
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
            let key_pkg: frost::keys::KeyPackage = serde_json::from_str(&fs::read_to_string(key)?)?;
            let mut rng = ChaCha20Rng::from_entropy();

            let mut public_commitments: CommitmentMap = BTreeMap::new();
            let mut secret_nonce_vault: NonceMap = BTreeMap::new();
            
            for i in 0..*count {
                let (nonces, commitments) = round1::commit(key_pkg.signing_share(), &mut rng);
                public_commitments.insert(i as u32, commitments);
                secret_nonce_vault.insert(i as u32, nonces);
            }

            let id_hex = hex::encode(key_pkg.identifier().serialize());
            let vault_path = format!("p{}_vault.json", id_hex);
            fs::write(&vault_path, serde_json::to_string(&secret_nonce_vault)?)?;

            let comms_path = format!("p{}_public_comms.json", id_hex);
            fs::write(&comms_path, serde_json::to_string(&public_commitments)?)?;

            println!("✅ Vaulted nonces to {}", vault_path);
            println!("✅ Public commitments saved to {}", comms_path);
        }

        Commands::Sign { message, index, key, vault } => {
            println!("✍️  Executing Sign: Index #{} for '{}'...", index, message);

            let key_pkg: frost::keys::KeyPackage = serde_json::from_str(&fs::read_to_string(key)?)?;
            let mut vault_data: NonceMap = serde_json::from_str(&fs::read_to_string(vault)?)?;

            let signing_nonces = vault_data.remove(&(*index as u32))
                .ok_or("Nonce index not found!")?;
            fs::write(vault, serde_json::to_string(&vault_data)?)?;

            let mut commitments_map = BTreeMap::new();
            commitments_map.insert(*key_pkg.identifier(), *signing_nonces.commitments());

            // Scan directory for peer commitments to satisfy threshold
            let paths = fs::read_dir(".")?;
            for path in paths {
                let path = path?.path();
                let filename = path.file_name().unwrap().to_str().unwrap();
                
                if filename.starts_with('p') && filename.contains("_public_comms.json") {
                    let id_hex = filename.strip_prefix('p').unwrap().strip_suffix("_public_comms.json").unwrap();
                    // Wrap hex in quotes for serde to treat it as a string-based Identifier
                    let peer_id: frost::Identifier = serde_json::from_str(&format!("\"{}\"", id_hex))?;

                    if peer_id != *key_pkg.identifier() {
                        let peer_comms: CommitmentMap = serde_json::from_str(&fs::read_to_string(&path)?)?;
                        if let Some(comm) = peer_comms.get(&(*index as u32)) {
                            commitments_map.insert(peer_id, *comm);
                            println!("  📎 Added commitment from peer: {}", id_hex);
                        }
                    }
                }
            }

            if (commitments_map.len() as u16) < *key_pkg.min_signers() {
                return Err(format!("Threshold not met! Have {}, need {}", commitments_map.len(), key_pkg.min_signers()).into());
            }

            let signing_package = frost::SigningPackage::new(commitments_map, message.as_bytes());
            let signature_share = frost::round2::sign(&signing_package, &signing_nonces, &key_pkg)?;

            let share_file = format!("p{}_share.json", hex::encode(key_pkg.identifier().serialize()));
            fs::write(&share_file, serde_json::to_string(&signature_share)?)?;
            println!("✅ Share saved to {}", share_file);
        }

Commands::Aggregate { message, shares } => {
            println!("🧬 Executing Aggregate: {} shares for '{}'...", shares.len(), message);
            
            // 1. Load Group Public Key
            let pub_json = fs::read_to_string("group_public.json")?;
            let pubkey_package: frost::keys::PublicKeyPackage = serde_json::from_str(&pub_json)?;

            // 2. Reconstruct the SigningPackage
            // Note: In this CLI flow, we assume we're using Index 0.
            // In production, you'd track which nonce index was used.
            let mut commitments_map = BTreeMap::new();
            let mut signature_shares: BTreeMap<frost::Identifier, frost::round2::SignatureShare> = BTreeMap::new();

            for share_path in shares {
                let share: frost::round2::SignatureShare = serde_json::from_str(&fs::read_to_string(share_path)?)?;
                
                // Parse ID from filename: p<ID>_share.json
                let filename = std::path::Path::new(share_path).file_name().unwrap().to_str().unwrap();
                let id_hex = filename.strip_prefix('p').unwrap().strip_suffix("_share.json").unwrap();
                let peer_id: frost::Identifier = serde_json::from_str(&format!("\"{}\"", id_hex))?;

                // Find the corresponding public commitment for this participant (using index 0 for this demo)
                let comms_file = format!("p{}_public_comms.json", id_hex);
                let peer_comms: CommitmentMap = serde_json::from_str(&fs::read_to_string(comms_file)?)?;
                let comm = peer_comms.get(&0).ok_or("Could not find commitment for index 0")?;

                commitments_map.insert(peer_id, *comm);
                signature_shares.insert(peer_id, share);
            }

            let signing_package = frost::SigningPackage::new(commitments_map, message.as_bytes());

            // 3. Aggregate everything into a BIP-340 Signature
            let group_signature = frost::aggregate(&signing_package, &signature_shares, &pubkey_package)?;

            // 4. Output the final signature (Hex encoded)
            let sig_hex = hex::encode(group_signature.serialize()?);
            println!("✅ Aggregation Successful!");
            println!("🔏 Final BIP-340 Signature: {}", sig_hex);
            
            fs::write("final_signature.json", serde_json::to_string(&group_signature)?)?;
            println!("💾 Signature saved to final_signature.json");
        }
    }

    Ok(())
}
