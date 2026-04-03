#!/bin/bash
set -e

MESSAGE="gnostr-bip64mod-test"
EXAMPLE="cli-parser"
FEATURES="nostr"

# 1. Keygen (Generates the 64-char hex files)
cargo run --example $EXAMPLE --features $FEATURES -- keygen --threshold 2 --total 3 -o .

# 2. Extract the actual filenames generated
P1_KEY=$(ls p*_key.json | sed -n '1p')
P2_KEY=$(ls p*_key.json | sed -n '2p')

# Extract ID and Vault names dynamically
P1_ID=$(echo $P1_KEY | cut -d'_' -f1)
P2_ID=$(echo $P2_KEY | cut -d'_' -f1)

# 3. Batch
cargo run --example $EXAMPLE --features $FEATURES -- batch --key "$P1_KEY"
cargo run --example $EXAMPLE --features $FEATURES -- batch --key "$P2_KEY"

# 4. Sign (Using index 0)
cargo run --example $EXAMPLE --features $FEATURES -- sign \
    --message "$MESSAGE" --index 0 --key "$P1_KEY" --vault "${P1_ID}_vault.json"

cargo run --example $EXAMPLE --features $FEATURES -- sign \
    --message "$MESSAGE" --index 0 --key "$P2_KEY" --vault "${P2_ID}_vault.json"

# 5. Aggregate
RAW_OUT=$(cargo run --example $EXAMPLE --features $FEATURES -- aggregate --message "$MESSAGE" p*_share.json)
SIG_HEX=$(echo "$RAW_OUT" | grep "Final BIP-340 Signature:" | awk '{print $4}')

# 6. Verify
cargo run --example $EXAMPLE --features $FEATURES -- verify \
    --message "$MESSAGE" \
    --signature "$SIG_HEX" \
    --public-key "group_public.json"
