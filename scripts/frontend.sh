#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$ROOT_DIR/scripts/common.sh"

FRONTEND_PORT="${FRONTEND_PORT:-5173}"

cd "$ROOT_DIR/frontend"

require_command npm
stop_process_on_port "$FRONTEND_PORT" "frontend"

if [[ ! -d node_modules ]]; then
  echo "Installing frontend dependencies..."
  npm install
fi

echo "Starting LetsOrder frontend on http://localhost:$FRONTEND_PORT"
npm run dev -- --host 127.0.0.1 --port "$FRONTEND_PORT"
