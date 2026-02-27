#!/usr/bin/env bash
set -euo pipefail

REMOTE_HOST="${REMOTE_HOST:-matth@server.matthandzel.com}"
REMOTE_REPO="${REMOTE_REPO:-/home/matth/Projects/lifelog}"
REMOTE_DB_ENDPOINT="${REMOTE_DB_ENDPOINT:-http://127.0.0.1:7183}"
REMOTE_NS="${REMOTE_NS:-lifelog}"
REMOTE_DB="${REMOTE_DB:-main}"
STREAM_ID="${STREAM_ID:-processes}"
DURATION_SECS="${DURATION_SECS:-75}"
COLLECTOR_BIN="${COLLECTOR_BIN:-./target/debug/lifelog-collector}"
SERVER_ADDR="${SERVER_ADDR:-}"
SKIP_COLLECTOR_RUN="${SKIP_COLLECTOR_RUN:-0}"

usage() {
  cat <<USAGE
Usage: $0 [--server-address URL] [--duration SECONDS]

Environment overrides:
  REMOTE_HOST, REMOTE_REPO, REMOTE_DB_ENDPOINT, REMOTE_NS, REMOTE_DB,
  STREAM_ID, DURATION_SECS, COLLECTOR_BIN, SERVER_ADDR, SKIP_COLLECTOR_RUN

Example:
  $0 --duration 90
  SERVER_ADDR=http://100.118.206.104:7182 $0
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --server-address)
      SERVER_ADDR="$2"
      shift 2
      ;;
    --duration)
      DURATION_SECS="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown arg: $1" >&2
      usage
      exit 2
      ;;
  esac
done

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "Missing required command: $1" >&2
    exit 1
  }
}

require_cmd ssh
require_cmd timeout
require_cmd rg

if [[ -z "$SERVER_ADDR" ]]; then
  TS_IP="$(ssh "$REMOTE_HOST" "tailscale ip -4 | head -n1" 2>/dev/null || true)"
  if [[ -z "$TS_IP" ]]; then
    echo "Could not determine remote Tailscale IP. Pass --server-address." >&2
    exit 1
  fi
  SERVER_ADDR="http://${TS_IP}:7182"
fi

# Runtime libs for collector when running outside nix develop.
LD_PATH_PARTS=()
for libname in libX11.so.6 libXtst.so.6 libv4l2.so.0 libasound.so.2; do
  libpath="$(find /nix/store -name "$libname" 2>/dev/null | head -n1 || true)"
  if [[ -n "$libpath" ]]; then
    LD_PATH_PARTS+=("$(dirname "$libpath")")
  fi
done
# de-dup while preserving order
if [[ "${#LD_PATH_PARTS[@]}" -gt 0 ]]; then
  mapfile -t LD_PATH_PARTS < <(printf "%s\n" "${LD_PATH_PARTS[@]}" | awk '!seen[$0]++')
fi
LD_LIBRARY_PATH_LOCAL="$(IFS=:; echo "${LD_PATH_PARTS[*]:-}")"

if [[ "$SKIP_COLLECTOR_RUN" != "1" && ! -x "$COLLECTOR_BIN" ]]; then
  echo "Collector binary not found/executable: $COLLECTOR_BIN" >&2
  exit 1
fi

query_count() {
  local out
  out="$(ssh "$REMOTE_HOST" "cd '$REMOTE_REPO' && printf \"SELECT count() AS n FROM upload_chunks WHERE stream_id = '$STREAM_ID' GROUP ALL;\\n\" | nix develop --command surreal sql --endpoint '$REMOTE_DB_ENDPOINT' --user root --pass root --namespace '$REMOTE_NS' --database '$REMOTE_DB' --hide-welcome --json" 2>/dev/null || true)"
  echo "$out" | rg -o '"n"\s*:\s*[0-9]+' | head -n1 | rg -o '[0-9]+' || echo 0
}

ensure_remote_server() {
  local running
  running="$(ssh "$REMOTE_HOST" "pgrep -f 'target/debug/lifelog-server-backend' >/dev/null; echo \$?" 2>/dev/null || echo 1)"
  if [[ "$running" == "0" ]]; then
    return
  fi

  echo "Remote server not running; starting it..."
  ssh "$REMOTE_HOST" "cd '$REMOTE_REPO' && nohup env LIFELOG_HOST=0.0.0.0 LIFELOG_DB_USER=root LIFELOG_DB_PASS=root RUST_LOG=info nix develop --command cargo run -p lifelog-server --bin lifelog-server-backend > /tmp/lifelog-server-remote.log 2>&1 < /dev/null &" >/dev/null

  sleep 5
  ssh "$REMOTE_HOST" "pgrep -f 'target/debug/lifelog-server-backend' >/dev/null" || {
    echo "Failed to start remote server. Check /tmp/lifelog-server-remote.log on $REMOTE_HOST" >&2
    exit 1
  }
}

echo "Remote host:         $REMOTE_HOST"
echo "Server address:      $SERVER_ADDR"
echo "Stream for check:    $STREAM_ID"
echo "Collector duration:  ${DURATION_SECS}s"

ensure_remote_server

before_count="$(query_count)"
echo "Before count ($STREAM_ID): $before_count"

if [[ "$SKIP_COLLECTOR_RUN" == "1" ]]; then
  echo "SKIP_COLLECTOR_RUN=1; not launching collector. Waiting ${DURATION_SECS}s for remote ingest delta..."
  sleep "$DURATION_SECS"
else
  echo "Running collector..."
  set +e
  if [[ -n "$LD_LIBRARY_PATH_LOCAL" ]]; then
    LD_LIBRARY_PATH="$LD_LIBRARY_PATH_LOCAL" timeout "${DURATION_SECS}s" "$COLLECTOR_BIN" --server-address "$SERVER_ADDR"
    rc=$?
  else
    timeout "${DURATION_SECS}s" "$COLLECTOR_BIN" --server-address "$SERVER_ADDR"
    rc=$?
  fi
  set -e

  if [[ "$rc" -ne 0 && "$rc" -ne 124 ]]; then
    echo "Collector exited with code $rc" >&2
    exit "$rc"
  fi
fi

after_count="$(query_count)"
echo "After count  ($STREAM_ID): $after_count"

if [[ "$after_count" -gt "$before_count" ]]; then
  echo "PASS: remote ingest increased by $((after_count - before_count)) rows"
  exit 0
fi

echo "FAIL: no increase detected in upload_chunks for stream '$STREAM_ID'" >&2
echo "Check remote logs: ssh $REMOTE_HOST 'tail -n 120 /tmp/lifelog-server-remote.log'" >&2
exit 3
