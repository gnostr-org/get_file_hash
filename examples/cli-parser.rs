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
    /// Step 5: Verify a BIP-340 signature against the group public key
    Verify {
        #[arg(short, long)]
        message: String,
        #[arg(short, long)]
        signature: String,
        #[arg(short, long)]
        public_key: PathBuf,
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
                fs::write(path.join(file_name), serde_json::to_string_pretty(&key_pkg)?)?;
            }
        }

        Commands::Batch { count, key } => {
            println!("📦 Executing Batch...");
            let key_pkg: frost::keys::KeyPackage = serde_json::from_str(&fs::read_to_string(key)?)?;
            let mut rng = ChaCha20Rng::from_entropy();
            let mut public_commitments = CommitmentMap::new();
            let mut secret_nonce_vault = NonceMap::new();
            
            for i in 0..*count {
                let (nonces, commitments) = round1::commit(key_pkg.signing_share(), &mut rng);
                public_commitments.insert(i as u32, commitments);
                secret_nonce_vault.insert(i as u32, nonces);
            }

            let id_hex = hex::encode(key_pkg.identifier().serialize());
            fs::write(format!("p{}_vault.json", id_hex), serde_json::to_string(&secret_nonce_vault)?)?;
            fs::write(format!("p{}_public_comms.json", id_hex), serde_json::to_string(&public_commitments)?)?;
            println!("✅ Nonces and Commitments saved for ID {}", id_hex);
        }

        Commands::Sign { message, index, key, vault } => {
            println!("✍️  Executing Sign: Index #{}...", index);
            let key_pkg: frost::keys::KeyPackage = serde_json::from_str(&fs::read_to_string(key)?)?;
            let mut vault_data: NonceMap = serde_json::from_str(&fs::read_to_string(vault)?)?;
            let signing_nonces = vault_data.remove(&(*index as u32)).ok_or("Nonce not found!")?;
            fs::write(vault, serde_json::to_string(&vault_data)?)?;

            let mut commitments_map = BTreeMap::new();
            commitments_map.insert(*key_pkg.identifier(), *signing_nonces.commitments());

            // Discovery logic for peers
            for entry in fs::read_dir(".")? {
                let path = entry?.path();
                let fname = path.file_name().unwrap().to_str().unwrap();
                if fname.starts_with('p') && fname.contains("_public_comms.json") {
                    let id_hex = fname.strip_prefix('p').unwrap().strip_suffix("_public_comms.json").unwrap();
                    let peer_id: frost::Identifier = serde_json::from_str(&format!("\"{}\"", id_hex))?;
                    if peer_id != *key_pkg.identifier() {
                        let peer_comms: CommitmentMap = serde_json::from_str(&fs::read_to_string(&path)?)?;
                        if let Some(c) = peer_comms.get(&(*index as u32)) {
                            commitments_map.insert(peer_id, *c);
                        }
                    }
                }
            }

            let signing_package = frost::SigningPackage::new(commitments_map, message.as_bytes());
            let share = frost::round2::sign(&signing_package, &signing_nonces, &key_pkg)?;
            let share_file = format!("p{}_share.json", hex::encode(key_pkg.identifier().serialize()));
            fs::write(&share_file, serde_json::to_string(&share)?)?;
            println!("✅ Share saved to {}", share_file);
        }

        Commands::Aggregate { message, shares } => {
            println!("🧬 Executing Aggregate...");
            let pubkey_package: frost::keys::PublicKeyPackage = serde_json::from_str(&fs::read_to_string("group_public.json")?)?;
            let mut commitments_map = BTreeMap::new();
            let mut signature_shares = BTreeMap::new();

            for share_path in shares {
                let share: frost::round2::SignatureShare = serde_json::from_str(&fs::read_to_string(share_path)?)?;
                let fname = std::path::Path::new(share_path).file_name().unwrap().to_str().unwrap();
                let id_hex = fname.strip_prefix('p').unwrap().strip_suffix("_share.json").unwrap();
                let peer_id: frost::Identifier = serde_json::from_str(&format!("\"{}\"", id_hex))?;

                let comms_file = format!("p{}_public_comms.json", id_hex);
                let peer_comms: CommitmentMap = serde_json::from_str(&fs::read_to_string(comms_file)?)?;
                commitments_map.insert(peer_id, *peer_comms.get(&0).unwrap());
                signature_shares.insert(peer_id, share);
            }

            let signing_package = frost::SigningPackage::new(commitments_map, message.as_bytes());
            let group_sig = frost::aggregate(&signing_package, &signature_shares, &pubkey_package)?;
            let sig_hex = hex::encode(group_sig.serialize()?);
            println!("✅ Aggregation Successful!\nFinal BIP-340 Signature: {}", sig_hex);
            fs::write("final_signature.json", serde_json::to_string(&group_sig)?)?;
        }

        Commands::Verify { message, signature, public_key } => {
            println!("🔍 Executing Verify...");
            let pubkey_package: frost::keys::PublicKeyPackage = serde_json::from_str(&fs::read_to_string(public_key)?)?;
            let sig_bytes = hex::decode(signature)?;
            let group_sig = frost::Signature::deserialize(&sig_bytes)?;

            match pubkey_package.verifying_key().verify(message.as_bytes(), &group_sig) {
                Ok(_) => println!("✅ SUCCESS: The signature is VALID!"),
                Err(_) => println!("❌ FAILURE: Invalid signature."),
            }
        }
    }
    Ok(())
}
