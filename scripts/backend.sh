#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$ROOT_DIR/scripts/common.sh"

BACKEND_PORT="${PORT:-8080}"
DATABASE_URL="${DATABASE_URL:-sqlite://$ROOT_DIR/backend/letsorder.db?mode=rwc}"

cd "$ROOT_DIR"

require_command cargo
stop_process_on_port "$BACKEND_PORT" "backend"

echo "Starting LetsOrder backend on http://localhost:$BACKEND_PORT"
echo "Database: $DATABASE_URL"

DATABASE_URL="$DATABASE_URL" PORT="$BACKEND_PORT" cargo run -p letsorder-backend
