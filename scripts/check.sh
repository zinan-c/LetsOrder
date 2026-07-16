#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "Checking Rust backend..."
(cd "$ROOT_DIR" && cargo fmt --all --check && cargo check && cargo clippy --workspace --all-targets --all-features -- -D warnings && cargo test)

echo "Checking React frontend..."
(cd "$ROOT_DIR/frontend" && npm run lint && npm run build && npm run e2e)

echo "All checks passed."
