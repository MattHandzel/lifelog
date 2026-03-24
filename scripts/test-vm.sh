#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(builtin cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(builtin cd "$SCRIPT_DIR/.." && pwd)"
VM_NAME="lifelog-test-vm"
SSH_KEY="${PROJECT_DIR}/nix/vm-test-key"
SSH_PORT=2222
SCREENSHOT_PATH="/tmp/vm-e2e-screenshot.png"

log() { echo "[test-vm] $(date '+%H:%M:%S') $*"; }
fail() { log "FAIL: $*"; cleanup; exit 1; }

cleanup() {
    log "Shutting down VM..."
    if [ -n "${VM_PID:-}" ] && kill -0 "$VM_PID" 2>/dev/null; then
        kill "$VM_PID" 2>/dev/null || true
        wait "$VM_PID" 2>/dev/null || true
    fi
}

ssh_cmd() {
    ssh -o ConnectTimeout=5 \
        -o StrictHostKeyChecking=no \
        -o UserKnownHostsFile=/dev/null \
        -o LogLevel=ERROR \
        -i "$SSH_KEY" \
        -p "$SSH_PORT" \
        root@localhost "$@"
}

scp_cmd() {
    scp -o ConnectTimeout=5 \
        -o StrictHostKeyChecking=no \
        -o UserKnownHostsFile=/dev/null \
        -o LogLevel=ERROR \
        -i "$SSH_KEY" \
        -P "$SSH_PORT" \
        "$@"
}

# --- Pre-flight checks ---
if [ ! -f "$SSH_KEY" ]; then
    log "Generating SSH key pair..."
    ssh-keygen -t ed25519 -f "$SSH_KEY" -N "" -C "lifelog-vm-test"
fi
chmod 600 "$SSH_KEY"

# --- Build VM ---
log "Building VM configuration..."
nix build "${PROJECT_DIR}#nixosConfigurations.${VM_NAME}.config.microvm.declaredRunner" \
    -o "${PROJECT_DIR}/result-vm" || fail "VM build failed"

# --- Start VM ---
log "Starting VM..."
trap cleanup EXIT
"${PROJECT_DIR}/result-vm/bin/microvm-run" &
VM_PID=$!

# --- Wait for SSH ---
log "Waiting for SSH access..."
SSH_READY=false
for i in $(seq 1 60); do
    if ! kill -0 "$VM_PID" 2>/dev/null; then
        fail "VM process exited unexpectedly"
    fi
    if ssh_cmd "echo ready" 2>/dev/null; then
        SSH_READY=true
        log "SSH connection established (attempt $i)"
        break
    fi
    sleep 2
done
$SSH_READY || fail "SSH connection timed out after 120s"

# --- Check services ---
log "Checking systemd services..."
for svc in postgresql lifelog-server lifelog-collector xvfb; do
    STATUS=$(ssh_cmd "systemctl is-active $svc" 2>/dev/null || echo "inactive")
    if [ "$STATUS" = "active" ]; then
        log "  $svc: active"
    else
        log "  $svc: $STATUS (checking journal...)"
        ssh_cmd "journalctl -u $svc --no-pager -n 20" 2>/dev/null || true
        fail "Service $svc is not active ($STATUS)"
    fi
done

# --- Wait for frames in database ---
log "Waiting for data in database..."
DATA_FOUND=false
for i in $(seq 1 30); do
    COUNT=$(ssh_cmd "sudo -u lifelog psql -h /run/postgresql -U lifelog -d lifelog -tAc 'SELECT count(*) FROM frames;'" 2>/dev/null || echo "0")
    COUNT=$(echo "$COUNT" | tr -d '[:space:]')
    if [ "$COUNT" -gt "0" ] 2>/dev/null; then
        DATA_FOUND=true
        log "Frames in database: $COUNT (attempt $i)"
        break
    fi
    if [ "$((i % 5))" -eq 0 ]; then
        log "  Still waiting for frames... (attempt $i/30)"
    fi
    sleep 2
done
$DATA_FOUND || fail "No frames appeared in database after 60s"

# --- Check interface service ---
log "Checking interface service..."
IFACE_STATUS=$(ssh_cmd "systemctl is-active lifelog-interface" 2>/dev/null || echo "inactive")
if [ "$IFACE_STATUS" = "active" ]; then
    log "  lifelog-interface: active"
else
    log "  lifelog-interface: $IFACE_STATUS — attempting manual launch..."
    ssh_cmd "DISPLAY=:99 GDK_BACKEND=x11 LIFELOG_SERVER_ADDRESS=http://127.0.0.1:7182 lifelog-server-frontend &" 2>/dev/null || true
fi

log "Waiting for interface to render data..."
sleep 15

# --- Take screenshot ---
log "Taking screenshot..."
ssh_cmd "DISPLAY=:99 import -window root /tmp/vm-screenshot.png" 2>/dev/null \
    || fail "Screenshot command failed"

scp_cmd "root@localhost:/tmp/vm-screenshot.png" "$SCREENSHOT_PATH" \
    || fail "Failed to copy screenshot from VM"

# --- Verify screenshot has content ---
if [ ! -f "$SCREENSHOT_PATH" ]; then
    fail "Screenshot file not found at $SCREENSHOT_PATH"
fi

SIZE=$(stat -c%s "$SCREENSHOT_PATH" 2>/dev/null || echo "0")
log "Screenshot size: $SIZE bytes"

if [ "$SIZE" -lt "10000" ]; then
    fail "Screenshot appears blank or too small ($SIZE bytes)"
fi

# --- Collect additional diagnostics ---
log "Collecting diagnostics..."
ssh_cmd "sudo -u lifelog psql -h /run/postgresql -U lifelog -d lifelog -c 'SELECT id, modality, created_at FROM frames ORDER BY created_at DESC LIMIT 5;'" 2>/dev/null || true

log ""
log "=== E2E TEST PASSED ==="
log "Screenshot saved: $SCREENSHOT_PATH"
log "Frames in DB: $COUNT"
log ""
