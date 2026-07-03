#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BACKEND_PORT="${PORT:-8080}"
FRONTEND_PORT="${FRONTEND_PORT:-5173}"
DATABASE_URL="${DATABASE_URL:-sqlite://$ROOT_DIR/backend/letsorder.db?mode=rwc}"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1" >&2
    exit 1
  fi
}

cleanup() {
  if [[ -n "${BACKEND_PID:-}" ]]; then
    kill "$BACKEND_PID" >/dev/null 2>&1 || true
  fi
  if [[ -n "${FRONTEND_PID:-}" ]]; then
    kill "$FRONTEND_PID" >/dev/null 2>&1 || true
  fi
}

trap cleanup EXIT INT TERM

require_command cargo
require_command npm

if [[ ! -d "$ROOT_DIR/frontend/node_modules" ]]; then
  echo "Installing frontend dependencies..."
  (cd "$ROOT_DIR/frontend" && npm install)
fi

echo "Starting LetsOrder backend on http://localhost:$BACKEND_PORT"
(
  cd "$ROOT_DIR"
  DATABASE_URL="$DATABASE_URL" PORT="$BACKEND_PORT" cargo run -p letsorder-backend
) &
BACKEND_PID=$!

echo "Starting LetsOrder frontend on http://localhost:$FRONTEND_PORT"
(
  cd "$ROOT_DIR/frontend"
  npm run dev -- --host 127.0.0.1 --port "$FRONTEND_PORT"
) &
FRONTEND_PID=$!

echo
echo "LetsOrder is starting:"
echo "  Frontend: http://localhost:$FRONTEND_PORT"
echo "  Backend:  http://localhost:$BACKEND_PORT"
echo "  Health:   http://localhost:$BACKEND_PORT/health"
echo
echo "Press Ctrl+C to stop both servers."

wait "$BACKEND_PID" "$FRONTEND_PID"
