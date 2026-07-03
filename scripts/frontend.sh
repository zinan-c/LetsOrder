#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FRONTEND_PORT="${FRONTEND_PORT:-5173}"

cd "$ROOT_DIR/frontend"

if [[ ! -d node_modules ]]; then
  echo "Installing frontend dependencies..."
  npm install
fi

echo "Starting LetsOrder frontend on http://localhost:$FRONTEND_PORT"
npm run dev -- --host 127.0.0.1 --port "$FRONTEND_PORT"
