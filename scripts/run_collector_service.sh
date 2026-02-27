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

# Best-effort dynamic LD_LIBRARY_PATH for binaries executed outside nix develop.
ld_parts=()
for lib in libX11.so.6 libXtst.so.6 libv4l2.so.0 libasound.so.2 libwayland-client.so.0; do
  p="$(find /nix/store -name "$lib" 2>/dev/null | head -n1 || true)"
  if [[ -n "$p" ]]; then
    ld_parts+=("$(dirname "$p")")
  fi
done
if [[ ${#ld_parts[@]} -gt 0 ]]; then
  mapfile -t ld_parts < <(printf "%s\n" "${ld_parts[@]}" | awk '!seen[$0]++')
  export LD_LIBRARY_PATH="$(IFS=:; echo "${ld_parts[*]}")${LD_LIBRARY_PATH:+:${LD_LIBRARY_PATH}}"
fi

exec "$COLLECTOR_BIN" --server-address "$SERVER_ADDR"
