#!/bin/bash
set -e
MESSAGE="gnostr-bip64mod-test"
EXAMPLE="cli-parser"
FEATURES="nostr"

# 1. Keygen
cargo run --example $EXAMPLE --features nostr -- keygen --threshold 2 --total 3 -o .

# Capture IDs
P1_KEY=$(ls p*_key.json | sed -n '1p')
P2_KEY=$(ls p*_key.json | sed -n '2p')
P1_ID=$(echo $P1_KEY | cut -d'_' -f1)
P2_ID=$(echo $P2_KEY | cut -d'_' -f1)

# 2. Batch
cargo run --example $EXAMPLE --features nostr -- batch --key "$P1_KEY"
cargo run --example $EXAMPLE --features nostr -- batch --key "$P2_KEY"

# 3. Sign
cargo run --example $EXAMPLE --features nostr -- sign --message "$MESSAGE" --index 0 --key "$P1_KEY" --vault "${P1_ID}_vault.json"
cargo run --example $EXAMPLE --features nostr -- sign --message "$MESSAGE" --index 0 --key "$P2_KEY" --vault "${P2_ID}_vault.json"

# 4. Aggregate
cargo run --example $EXAMPLE --features nostr -- aggregate --message "$MESSAGE" p*_share.json
# 5. Verify the final signature
echo -e "\n--- Step 6: Verifying Final Signature ---"
# Configuration
MESSAGE="gnostr-bip64mod-test"
EXAMPLE="cli-parser"
FEATURES="nostr" # Ensure this matches your Cargo.toml feature name

# Capture the Signature Hex from the Aggregate output
# We use 'tail -n 2' and 'head -n 1' to grab the specific line with the hex
RAW_OUTPUT=$(cargo run --example $EXAMPLE --features $FEATURES -- aggregate --message "$MESSAGE" p*_share.json)
SIG_HEX=$(echo "$RAW_OUTPUT" | grep "Final BIP-340 Signature:" | awk '{print $4}')

echo -e "\n--- Step 6: Verifying Final Signature ---"
echo "Message: $MESSAGE"
echo "Sig: $SIG_HEX"

cargo run --example $EXAMPLE --features $FEATURES -- verify \
    --message "$MESSAGE" \
    --signature "$SIG_HEX" \
    --public-key "group_public.json"
