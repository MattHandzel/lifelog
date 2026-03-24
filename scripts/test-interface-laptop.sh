#!/usr/bin/env bash
# test-interface-laptop.sh — Launch, screenshot, and cleanup the Tauri interface on the laptop
# Usage: ./scripts/test-interface-laptop.sh [screenshot_name]
#
# This script:
# 1. Pulls latest code on the laptop
# 2. Builds the interface
# 3. Launches it with proper Wayland/TLS env
# 4. Takes a fullscreen screenshot
# 5. Copies screenshot locally
# 6. KILLS the interface (user needs their laptop!)
#
# Prerequisites:
# - SSH access to matth@nixos (laptop)
# - Server cert at ~/.config/lifelog/tls/server-ca.pem on laptop
# - Hyprland running on laptop

set -euo pipefail

LAPTOP="matth@nixos"
SCREENSHOT_NAME="${1:-lifelog-test-$(date +%H%M%S)}"
LOCAL_DIR="/tmp"

echo "=== Step 1: Pull and build on laptop ==="
ssh "$LAPTOP" bash <<'REMOTE'
cd ~/Projects/lifelog && git pull
nix develop --command cargo build -p lifelog-interface 2>&1 | tail -3
REMOTE

echo "=== Step 2: Launch interface ==="
ssh "$LAPTOP" bash <<'REMOTE'
export WAYLAND_DISPLAY=wayland-1
export XDG_RUNTIME_DIR=/run/user/1000
export LIFELOG_GRPC_SERVER_ADDRESS=https://YOUR_SERVER_IP:7182
export LIFELOG_TLS_CA_PATH=$HOME/.config/lifelog/tls/server-ca.pem
export LIFELOG_SERVER_DOMAIN=localhost
cd ~/Projects/lifelog
nohup target/debug/lifelog-server-frontend > /tmp/lifelog-ui.log 2>&1 &
echo "Launched PID: $!"
REMOTE

echo "=== Step 3: Wait for app to load (25s) ==="
sleep 25

echo "=== Step 4: Focus, fullscreen, screenshot ==="
ssh "$LAPTOP" bash <<REMOTE
export WAYLAND_DISPLAY=wayland-1
export XDG_RUNTIME_DIR=/run/user/1000
export HYPRLAND_INSTANCE_SIGNATURE=\$(ls /run/user/1000/hypr/ 2>/dev/null | head -1)
hyprctl dispatch focuswindow "class:lifelog-server-frontend"
sleep 0.5
hyprctl dispatch fullscreen 1
sleep 1
grim /tmp/${SCREENSHOT_NAME}.png
ffmpeg -y -i /tmp/${SCREENSHOT_NAME}.png -vf "scale=iw/2:ih/2" /tmp/${SCREENSHOT_NAME}-half.png 2>/dev/null
hyprctl dispatch fullscreen 1
echo "Screenshot taken"
REMOTE

echo "=== Step 5: Copy screenshot locally ==="
scp "$LAPTOP:/tmp/${SCREENSHOT_NAME}-half.png" "${LOCAL_DIR}/${SCREENSHOT_NAME}-half.png"

echo "=== Step 6: KILL interface (returning laptop to user) ==="
ssh "$LAPTOP" 'killall lifelog-server-frontend 2>/dev/null; echo "Interface killed"'

echo ""
echo "Screenshot saved to: ${LOCAL_DIR}/${SCREENSHOT_NAME}-half.png"
echo "UI log on laptop: /tmp/lifelog-ui.log"
