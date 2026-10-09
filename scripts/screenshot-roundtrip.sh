#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

banner() {
    printf '\n========== %s ==========\n' "$1"
}

run_test() {
    local filter="$1"
    shift
    cargo test --example screenshot --features nostr "$filter" -- --nocapture --test-threads=1 "$@"
}

banner "screenshot example mock round-trip tests"
run_test cli_round_trips_positionals_and_flags
run_test placeholder_png_round_trips_over_http

if [ -n "${NIP96_LIVE_TEST_SERVER:-}" ]; then
    banner "screenshot example live network round-trip test"
    printf 'Using NIP96_LIVE_TEST_SERVER=%s\n' "$NIP96_LIVE_TEST_SERVER"
    run_test live_network_round_trip_uploads_and_returns_url
else
    banner "screenshot example live network round-trip test"
    printf 'Skipping live network round-trip test; set NIP96_LIVE_TEST_SERVER to enable it.\n'
fi

banner "screenshot round-trip tests complete"