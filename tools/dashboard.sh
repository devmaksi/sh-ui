#!/usr/bin/env bash
# tools/dashboard.sh — start or stop the bigpowers dashboard for this project.
#
# Reason for depth: the bigpowers install path is not stable across machines (global npm,
# pi-env, npx cache), so the project resolves it from the `scripts` symlink instead of
# hardcoding one absolute path in package.json.
#
# Usage: bash tools/dashboard.sh start [port]
#        bash tools/dashboard.sh stop
set -euo pipefail

SELF="${BASH_SOURCE[0]}"
PROJECT_DIR="$(cd "$(dirname "$SELF")/.." && pwd)"
PID_FILE="$PROJECT_DIR/.bigpowers/dashboard.pid"
PORT="${2:-7742}"

resolve_bigpowers() {
  if [ -n "${BIGPOWERS_HOME:-}" ]; then
    echo "$BIGPOWERS_HOME"
    return 0
  fi
  if [ -L "$PROJECT_DIR/scripts" ]; then
    dirname "$(readlink -f "$PROJECT_DIR/scripts")"
    return 0
  fi
  echo "Cannot find the bigpowers install. Set BIGPOWERS_HOME." >&2
  return 1
}

start() {
  local bp dashboard_bin
  bp="$(resolve_bigpowers)"
  dashboard_bin="$bp/dashboard/bin/dashboard.js"

  if [ ! -f "$dashboard_bin" ]; then
    echo "Not found: $dashboard_bin" >&2
    exit 1
  fi
  if [ ! -d "$bp/dashboard/node_modules/express" ]; then
    echo "Dashboard dependencies are missing. Run: cd $bp/dashboard && npm install" >&2
    exit 1
  fi

  if [ -f "$PID_FILE" ] && kill -0 "$(cat "$PID_FILE")" 2>/dev/null; then
    running_pid="$(cat "$PID_FILE")"
    echo "Dashboard already running with pid $running_pid on http://localhost:$PORT"
    exit 0
  fi

  mkdir -p "$(dirname "$PID_FILE")"
  nohup node "$dashboard_bin" --web --project "$PROJECT_DIR" --port "$PORT" \
    >"$PROJECT_DIR/.bigpowers/dashboard.log" 2>&1 &
  echo $! >"$PID_FILE"
  started_pid="$(cat "$PID_FILE")"
  echo "Dashboard: http://localhost:$PORT pid $started_pid log .bigpowers/dashboard.log"
}

stop() {
  if [ ! -f "$PID_FILE" ]; then
    echo "No dashboard pid file. Nothing to stop."
    exit 0
  fi
  local pid
  pid="$(cat "$PID_FILE")"
  if kill "$pid" 2>/dev/null; then
    echo "Stopped dashboard pid $pid"
  else
    echo "Process $pid was not running."
  fi
  rm -f "$PID_FILE"
}

case "${1:-start}" in
  start) start ;;
  stop) stop ;;
  *)
    echo "Usage: bash tools/dashboard.sh start [port] | stop" >&2
    exit 1
    ;;
esac
