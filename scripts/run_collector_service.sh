#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="${REPO_DIR:-$HOME/Projects/lifelog}"
COLLECTOR_BIN="${COLLECTOR_BIN:-$REPO_DIR/target/debug/lifelog-collector}"
SERVER_ADDR="${SERVER_ADDR:-http://100.118.206.104:7182}"

if [[ ! -x "$COLLECTOR_BIN" ]]; then
  echo "collector binary is missing: $COLLECTOR_BIN" >&2
  echo "build it with: cd $REPO_DIR && nix develop --command cargo build -p lifelog-collector" >&2
  exit 1
fi

# Ensure user service has a sensible runtime base even before GUI env import.
export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
if [[ -z "${WAYLAND_DISPLAY:-}" && -S "$XDG_RUNTIME_DIR/wayland-0" ]]; then
  export WAYLAND_DISPLAY="wayland-0"
fi

# Execute under nix develop so runtime libs/tools are always present.
exec nix develop --command "$COLLECTOR_BIN" --server-address "$SERVER_ADDR"
