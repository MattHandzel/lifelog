#!/usr/bin/env bash
# Capture a screenshot from the laptop via SSH + hyprctl
# Usage: ./scripts/capture_laptop_screen.sh [output_path]
set -euo pipefail

OUTPUT="${1:-/tmp/laptop-screen.jpg}"
REMOTE="matth@nixos"

ssh "$REMOTE" "hyprctl --instance 0 dispatch exec 'grim -t jpeg -q 60 -s 0.5 /tmp/_screen_capture.jpg'"
sleep 2
scp "$REMOTE:/tmp/_screen_capture.jpg" "$OUTPUT"
echo "Screenshot saved to $OUTPUT"
