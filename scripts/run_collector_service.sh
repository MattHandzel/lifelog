#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="${REPO_DIR:-$HOME/Projects/lifelog}"
COLLECTOR_BIN="${COLLECTOR_BIN:-$REPO_DIR/target/release/lifelog-collector}"
SERVER_ADDR="${SERVER_ADDR:-https://YOUR_SERVER_IP:7182}"

if [[ ! -x "$COLLECTOR_BIN" ]]; then
  echo "collector binary is missing: $COLLECTOR_BIN" >&2
  echo "build it with: cd $REPO_DIR && nix develop --command cargo build -p lifelog-collector" >&2
  exit 1
fi

# Ensure env vars are exported for nix develop subshell.
export LIFELOG_TLS_CA_CERT_PATH="${LIFELOG_TLS_CA_CERT_PATH:-}"
export LIFELOG_TLS_SERVER_NAME="${LIFELOG_TLS_SERVER_NAME:-localhost}"
export LIFELOG_AUTH_TOKEN="${LIFELOG_AUTH_TOKEN:-}"
export LIFELOG_COLLECTOR_ID="${LIFELOG_COLLECTOR_ID:-}"
export LIFELOG_CONFIG_PATH="${LIFELOG_CONFIG_PATH:-}"
export RUST_LOG="${RUST_LOG:-info}"

# Ensure user service has a sensible runtime base even before GUI env import.
export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
if [[ -z "${WAYLAND_DISPLAY:-}" && -S "$XDG_RUNTIME_DIR/wayland-0" ]]; then
  export WAYLAND_DISPLAY="wayland-0"
fi

# Execute under nix develop so runtime libs/tools are always present.
exec nix develop --command "$COLLECTOR_BIN" --server-address "$SERVER_ADDR"
