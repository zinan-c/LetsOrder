#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$ROOT_DIR/scripts/common.sh"

BACKEND_PORT="${PORT:-8080}"
FRONTEND_PORT="${FRONTEND_PORT:-5173}"

echo "Stopping LetsOrder services if they are running..."
stop_process_on_port "$BACKEND_PORT" "backend"
stop_process_on_port "$FRONTEND_PORT" "frontend"
echo "LetsOrder services stopped."
