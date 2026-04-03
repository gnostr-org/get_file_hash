#!/bin/bash
set -e
MESSAGE="gnostr-bip64mod-test"
EXAMPLE="cli-parser"

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
