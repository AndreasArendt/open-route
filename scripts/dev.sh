#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "[dev] Missing required command: $1" >&2
    exit 1
  fi
}

require_cmd docker
require_cmd cargo
require_cmd npm

port_is_busy() {
  local port="$1"
  if command -v lsof >/dev/null 2>&1; then
    lsof -nP -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1
  else
    return 1
  fi
}

cd "$ROOT_DIR"

echo "[dev] Starting GraphHopper in Docker..."
docker compose up -d graphhopper

echo "[dev] Stopping Docker planner/UI services (if running)..."
docker compose stop planner ui >/dev/null 2>&1 || true

if port_is_busy 8080; then
  echo "[dev] Port 8080 is already in use. Stop the process using it and retry." >&2
  exit 1
fi

if port_is_busy 5173; then
  echo "[dev] Port 5173 is already in use. Stop the process using it and retry." >&2
  exit 1
fi

if [ ! -d "$ROOT_DIR/ui/node_modules" ]; then
  echo "[dev] Installing UI dependencies (first run)..."
  (
    cd "$ROOT_DIR/ui"
    npm install
  )
fi

echo "[dev] Starting planner on http://localhost:8080 ..."
(
  cd "$ROOT_DIR/planner"
  GH_BASE_URL="${GH_BASE_URL:-http://localhost:8989}" \
  RUST_LOG="${RUST_LOG:-info}" \
  cargo run
) &
PLANNER_PID=$!

echo "[dev] Starting UI on http://localhost:5173 ..."
(
  cd "$ROOT_DIR/ui"
  npm run dev -- --host 0.0.0.0 --port 5173
) &
UI_PID=$!

cleanup() {
  echo
  echo "[dev] Stopping local planner/UI processes..."
  kill "$PLANNER_PID" "$UI_PID" 2>/dev/null || true
  wait "$PLANNER_PID" "$UI_PID" 2>/dev/null || true
}

trap cleanup INT TERM EXIT

while kill -0 "$PLANNER_PID" 2>/dev/null && kill -0 "$UI_PID" 2>/dev/null; do
  sleep 1
done

if ! kill -0 "$PLANNER_PID" 2>/dev/null; then
  echo "[dev] Planner exited."
fi

if ! kill -0 "$UI_PID" 2>/dev/null; then
  echo "[dev] UI exited."
fi
