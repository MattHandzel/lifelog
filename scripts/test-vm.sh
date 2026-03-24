#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(builtin cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(builtin cd "$SCRIPT_DIR/.." && pwd)"
VM_NAME="lifelog-test-vm"

log() { echo "[test-vm] $*"; }
fail() { log "FAIL: $*"; exit 1; }

cleanup() {
    log "Shutting down VM..."
    microvm -s "$VM_NAME" 2>/dev/null || true
}

# Build the VM
log "Building VM configuration..."
nix build "${PROJECT_DIR}#nixosConfigurations.${VM_NAME}.config.microvm.declaredRunner" \
    -o "${PROJECT_DIR}/result-vm" || fail "VM build failed"

# Start the VM
log "Starting VM..."
trap cleanup EXIT
"${PROJECT_DIR}/result-vm/bin/microvm-run" &
VM_PID=$!

# Wait for VM to boot (check for SSH or serial console readiness)
log "Waiting for VM to boot..."
TRIES=0
MAX_TRIES=30
while [ $TRIES -lt $MAX_TRIES ]; do
    if kill -0 "$VM_PID" 2>/dev/null; then
        sleep 2
        TRIES=$((TRIES + 1))
        log "Waiting... ($TRIES/$MAX_TRIES)"
    else
        fail "VM process exited unexpectedly"
    fi
done

log "VM boot wait complete (${MAX_TRIES}x2s = $((MAX_TRIES * 2))s)"

# Health checks would go here once we have a way to communicate with the VM
# For now, verify the VM process is still running
if kill -0 "$VM_PID" 2>/dev/null; then
    log "PASS: VM is running"
else
    fail "VM process is not running"
fi

log "All basic checks passed."
log ""
log "NOTE: Full health checks (server starts, collector connects, frames in DB)"
log "require serial console or SSH access to the VM. This skeleton verifies"
log "the VM boots and stays running."
log ""
log "To add full checks, configure a vsock or serial console for command execution."
