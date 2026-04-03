#!/bin/bash
set -e

# Configuration
MESSAGE="gnostr-bip64mod-test-message"
THRESHOLD=2
TOTAL=3
EXAMPLE="cli-parser"
FEATURES="nostr"

echo "🚀 Starting FROST 2-of-3 Threshold Sequence..."

# 1. Cleanup old files
rm -f *.json

# 2. Key Generation (Dealer Mode)
echo "--- Step 1: Keygen ---"
cargo run --example $EXAMPLE --features $FEATURES -- keygen --threshold $THRESHOLD --total $TOTAL -o .

# Find the generated key files
# They are named based on their Hex ID, e.g., p000...01_key.json
KEYS=( $(ls p*_key.json | sort) )
P1_KEY=${KEYS[0]}
P2_KEY=${KEYS[1]}

echo "Using Keys: $P1_KEY and $P2_KEY"

# 3. Batch Nonce Generation for Participants 1 and 2
echo -e "\n--- Step 2: Batching Nonces ---"
cargo run --example $EXAMPLE --features $FEATURES -- batch --count 10 --key "$P1_KEY"
# Note: The current CLI hardcodes vault names, so we move them to avoid overwriting
mv p1_batch_vault.json p1_vault.json

cargo run --example $EXAMPLE --features $FEATURES -- batch --count 10 --key "$P2_KEY"
mv p1_batch_vault.json p2_vault.json

# 4. Signing (Round 2)
# Both participants must sign the SAME message using the SAME index
echo -e "\n--- Step 3: Participant 1 Signing ---"
cargo run --example $EXAMPLE --features $FEATURES -- sign \
    --message "$MESSAGE" \
    --index 0 \
    --key "$P1_KEY" \
    --vault p1_vault.json

echo -e "\n--- Step 4: Participant 2 Signing ---"
cargo run --example $EXAMPLE --features $FEATURES -- sign \
    --message "$MESSAGE" \
    --index 0 \
    --key "$P2_KEY" \
    --vault p2_vault.json

# 5. Aggregation
echo -e "\n--- Step 5: Aggregating Shares ---"
SHARES=( $(ls p*_share.json) )
cargo run --example $EXAMPLE --features $FEATURES -- aggregate \
    --message "$MESSAGE" \
    "${SHARES[@]}"

echo -e "\n✅ Sequence Complete."
