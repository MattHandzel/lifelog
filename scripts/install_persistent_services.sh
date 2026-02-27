#!/usr/bin/env bash
set -euo pipefail

REMOTE_HOST="${REMOTE_HOST:-matth@server.matthandzel.com}"
REMOTE_REPO="${REMOTE_REPO:-/home/matth/Projects/lifelog}"

echo "Installing remote system services on $REMOTE_HOST ..."
ssh "$REMOTE_HOST" "cd '$REMOTE_REPO' && sudo cp deploy/systemd/lifelog-surrealdb.service /etc/systemd/system/lifelog-surrealdb.service && sudo cp deploy/systemd/lifelog-server.service /etc/systemd/system/lifelog-server.service && sudo systemctl daemon-reload && sudo systemctl enable --now lifelog-surrealdb.service lifelog-server.service"

echo "Installing local user services ..."
mkdir -p "$HOME/.config/systemd/user"
cp deploy/systemd-user/lifelog-collector.service "$HOME/.config/systemd/user/"
cp deploy/systemd-user/lifelog-ingest-validate.service "$HOME/.config/systemd/user/"
cp deploy/systemd-user/lifelog-ingest-validate.timer "$HOME/.config/systemd/user/"
mkdir -p "$HOME/.config/lifelog"
cp deploy/config/collector.laptop.toml "$HOME/.config/lifelog/config.toml"

systemctl --user daemon-reload
systemctl --user import-environment DISPLAY WAYLAND_DISPLAY XAUTHORITY XDG_SESSION_TYPE XDG_CURRENT_DESKTOP DBUS_SESSION_BUS_ADDRESS
sudo loginctl enable-linger "$USER"
systemctl --user enable --now lifelog-collector.service lifelog-ingest-validate.timer

echo "Done."
