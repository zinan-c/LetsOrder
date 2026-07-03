#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BACKEND_PORT="${PORT:-8080}"
DATABASE_URL="${DATABASE_URL:-sqlite://$ROOT_DIR/backend/letsorder.db?mode=rwc}"

cd "$ROOT_DIR"

echo "Starting LetsOrder backend on http://localhost:$BACKEND_PORT"
echo "Database: $DATABASE_URL"

DATABASE_URL="$DATABASE_URL" PORT="$BACKEND_PORT" cargo run -p letsorder-backend
