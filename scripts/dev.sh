#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$ROOT_DIR/scripts/common.sh"

BACKEND_PORT="${PORT:-8080}"
FRONTEND_PORT="${FRONTEND_PORT:-5173}"
DATABASE_URL="${DATABASE_URL:-sqlite://$ROOT_DIR/backend/letsorder.db?mode=rwc}"

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

stop_process_on_port "$BACKEND_PORT" "backend"
stop_process_on_port "$FRONTEND_PORT" "frontend"

if [[ "$DATABASE_URL" == sqlite://* ]]; then
  DATABASE_PATH="${DATABASE_URL#sqlite://}"
  DATABASE_PATH="${DATABASE_PATH%%\?*}"
  if [[ -n "$DATABASE_PATH" && -f "$DATABASE_PATH" ]]; then
    echo "Clearing development database: $DATABASE_PATH"
    rm -f "$DATABASE_PATH"
  fi
else
  echo "Skipping automatic database cleanup for non-SQLite DATABASE_URL."
fi

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

echo "Waiting for backend health check..."
wait_for_http "http://127.0.0.1:$BACKEND_PORT/health" "Backend"

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
