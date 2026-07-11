#!/bin/bash
export WAYLAND_DISPLAY=wayland-0
export XDG_RUNTIME_DIR=/run/user/$(id -u)
cd ~/Projects/lifelog
exec nix develop .#frontend --command ./target/release/lifelog-server-frontend
