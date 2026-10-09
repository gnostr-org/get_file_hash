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
    cargo test --example screenshot --features nostr "$filter" -- --nocapture --test-threads=1 --exact "$@"
}

run_local_round_trip() {
    local server_port="8765"
    local server_log
    server_log="$(mktemp)"
    trap 'rm -f "$server_log"' RETURN

    banner "screenshot example local network round-trip"
    python3 -u - <<'PY' >"$server_log" 2>&1 &
import json
from http.server import BaseHTTPRequestHandler, HTTPServer
from urllib.parse import urlparse

PORT = 8765
PAYLOAD = open("src/get_file_hash_core/src/icon.svg", "rb").read()

class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        path = urlparse(self.path).path
        if path == "/.well-known/nostr/nip96.json":
            body = json.dumps({
                "api_url": f"http://127.0.0.1:{PORT}/api/v2/nip96/upload",
            }).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if path == "/files/icon.svg":
            body = PAYLOAD
            self.send_response(200)
            self.send_header("Content-Type", "image/svg+xml")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        self.send_response(404)
        self.end_headers()

    def do_POST(self):
        path = urlparse(self.path).path
        if path == "/api/v2/nip96/upload":
            length = int(self.headers.get("Content-Length", "0"))
            self.rfile.read(length)
            body = json.dumps({
                "status": "success",
                "message": "ok",
                "nip94_event": {
                    "tags": [["url", f"http://127.0.0.1:{PORT}/files/icon.svg"]]
                },
            }).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        self.send_response(404)
        self.end_headers()

    def log_message(self, fmt, *args):
        pass

HTTPServer(("127.0.0.1", PORT), Handler).serve_forever()
PY
    local server_pid=$!
    trap 'kill "$server_pid" 2>/dev/null || true; rm -f "$server_log"' RETURN

    sleep 1
    cargo run --example screenshot --features nostr -- ./src/get_file_hash_core/src/icon.svg --server "http://127.0.0.1:${server_port}"

    banner "screenshot example wrote these files"
    ls -1 icon-*.svg icon-*.png 2>/dev/null || true

    kill "$server_pid" 2>/dev/null || true
}

banner "screenshot example mock round-trip tests"
run_test tests::cli_round_trips_positionals_and_flags
run_test tests::icon_svg_round_trips_over_http
run_test tests::png_round_trips_over_http
run_test tests::real_icon_png_round_trips_over_http
run_local_round_trip

banner "screenshot round-trip tests complete"