#!/usr/bin/env bash

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1" >&2
    exit 1
  fi
}

stop_process_on_port() {
  local port="$1"
  local label="$2"

  if ! command -v lsof >/dev/null 2>&1; then
    echo "Skipping stale $label cleanup: lsof is not available."
    return
  fi

  local pids
  pids="$(lsof -nP -tiTCP:"$port" -sTCP:LISTEN 2>/dev/null || true)"

  if [[ -z "$pids" ]]; then
    return
  fi

  echo "Stopping stale $label process on port $port: $pids"
  kill $pids >/dev/null 2>&1 || true
  sleep 1

  local remaining_pids
  remaining_pids="$(lsof -nP -tiTCP:"$port" -sTCP:LISTEN 2>/dev/null || true)"

  if [[ -n "$remaining_pids" ]]; then
    echo "Force stopping stale $label process on port $port: $remaining_pids"
    kill -9 $remaining_pids >/dev/null 2>&1 || true
  fi
}
