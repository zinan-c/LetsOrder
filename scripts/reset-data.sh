#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATABASE_PATH="${DATABASE_PATH:-$ROOT_DIR/backend/letsorder.db}"

if [[ ! -f "$DATABASE_PATH" ]]; then
  echo "Database file not found: $DATABASE_PATH" >&2
  exit 1
fi

sqlite3 "$DATABASE_PATH" <<'SQL'
PRAGMA foreign_keys = OFF;
DELETE FROM activity_logs;
DELETE FROM photos;
DELETE FROM menu_items;
DELETE FROM participants;
DELETE FROM gatherings;
VACUUM;
PRAGMA foreign_keys = ON;
SQL

echo "Cleared persisted LetsOrder data from $DATABASE_PATH."
