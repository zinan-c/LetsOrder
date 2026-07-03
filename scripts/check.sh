#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "Checking Rust backend..."
(cd "$ROOT_DIR" && cargo fmt --all --check && cargo check)

echo "Checking React frontend..."
(cd "$ROOT_DIR/frontend" && npm run build)

echo "All checks passed."
