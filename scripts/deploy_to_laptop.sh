#!/usr/bin/env bash
# Build on server and deploy to laptop
# Usage: ./scripts/deploy_to_laptop.sh [collector|frontend|both]
set -euo pipefail

TARGET="${1:-collector}"
REMOTE="matth@nixos"

case "$TARGET" in
  collector)
    echo "Building collector..."
    nix develop --command cargo build --release -p lifelog-collector
    echo "Deploying to laptop..."
    ssh "$REMOTE" "systemctl --user stop lifelog-collector 2>/dev/null || true"
    scp target/release/lifelog-collector "$REMOTE:~/Projects/lifelog/target/release/"
    ssh "$REMOTE" "systemctl --user start lifelog-collector"
    echo "Collector deployed and restarted."
    ;;
  frontend)
    echo "Building frontend..."
    nix develop --command cargo build --release -p lifelog-interface --bin lifelog-server-frontend
    echo "Deploying to laptop..."
    ssh "$REMOTE" "pkill -f lifelog-server-frontend 2>/dev/null || true"
    scp target/release/lifelog-server-frontend "$REMOTE:~/Projects/lifelog/target/release/"
    echo "Frontend deployed. Launch with: systemd-run --user --no-block /usr/bin/env bash ~/Projects/lifelog/scripts/run_frontend.sh"
    ;;
  both)
    "$0" collector
    "$0" frontend
    ;;
  *)
    echo "Usage: $0 [collector|frontend|both]"
    exit 1
    ;;
esac
