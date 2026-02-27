#!/usr/bin/env bash
set -euo pipefail

REMOTE_HOST="${REMOTE_HOST:-matth@server.matthandzel.com}"
REMOTE_REPO="${REMOTE_REPO:-/home/matth/Projects/lifelog}"

echo "Installing remote system services on $REMOTE_HOST ..."
ssh "$REMOTE_HOST" "set -euo pipefail; cd '$REMOTE_REPO'; if sudo cp deploy/systemd/lifelog-surrealdb.service /etc/systemd/system/lifelog-surrealdb.service >/dev/null 2>&1; then sudo cp deploy/systemd/lifelog-server.service /etc/systemd/system/lifelog-server.service; sudo systemctl daemon-reload; sudo systemctl enable --now lifelog-surrealdb.service lifelog-server.service; else mkdir -p \"\$HOME/.config/systemd/user\"; cp deploy/systemd-user/lifelog-surrealdb.service \"\$HOME/.config/systemd/user/\"; cp deploy/systemd-user/lifelog-server.service \"\$HOME/.config/systemd/user/\"; systemctl --user daemon-reload; sudo loginctl enable-linger \"\$USER\"; systemctl --user enable --now lifelog-surrealdb.service lifelog-server.service; fi"
ssh "$REMOTE_HOST" "set -euo pipefail; mkdir -p \"\$HOME/.config/lifelog\"; cp '$REMOTE_REPO/deploy/config/lifelog-config.laptop.toml' \"\$HOME/.config/lifelog/lifelog-config.toml\"; cp '$REMOTE_REPO/deploy/config/lifelog-config.laptop.toml' '$REMOTE_REPO/lifelog-config.toml'"

echo "Installing local user services ..."
mkdir -p "$HOME/.config/systemd/user"
cp deploy/systemd-user/lifelog-collector.service "$HOME/.config/systemd/user/"
cp deploy/systemd-user/lifelog-ingest-validate.service "$HOME/.config/systemd/user/"
cp deploy/systemd-user/lifelog-ingest-validate.timer "$HOME/.config/systemd/user/"
mkdir -p "$HOME/.config/lifelog"
cp deploy/config/lifelog-config.laptop.toml "$HOME/.config/lifelog/lifelog-config.toml"
cp deploy/config/lifelog-config.laptop.toml "$HOME/Projects/lifelog/lifelog-config.toml"

systemctl --user daemon-reload
systemctl --user import-environment DISPLAY WAYLAND_DISPLAY XAUTHORITY XDG_SESSION_TYPE XDG_CURRENT_DESKTOP DBUS_SESSION_BUS_ADDRESS
sudo loginctl enable-linger "$USER"
systemctl --user enable --now lifelog-collector.service lifelog-ingest-validate.timer

echo "Done."
