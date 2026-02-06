# Tier 3: Real Device Simulation — Design Document

## Overview

Tier 3 testing covers scenarios that cannot be faithfully reproduced in containers:
hardware-level behaviors like sleep/wake cycles, real network switching (WiFi ↔ cellular),
clock skew from NTP drift, and actual OS-level process lifecycle management.

## Prerequisites

- QEMU/libvirt or Vagrant for VM provisioning
- Android emulator (for mobile collector testing)
- Dedicated test runner machine (CI self-hosted runner or manual)

## Scenarios

### 1. Clock Skew Simulation

**Goal:** Verify that the server correctly compensates for device clocks that drift.

**Setup:**
- VM-1: NTP synchronized (reference clock)
- VM-2: NTP disabled, clock offset by +5 minutes
- VM-3: NTP disabled, clock offset by -2 minutes

**Procedure:**
1. Start server on VM-1
2. Start collectors on VM-2 and VM-3
3. Upload timestamped data from both collectors
4. Verify server-side timestamps are within tolerance of real time
5. Check `SkewEstimate` quality reported by each collector

**Expected:** Server detects skew, applies correction, and data is queryable
with canonical timestamps.

### 2. Sleep/Wake Cycle

**Goal:** Verify collectors resume cleanly after system suspend.

**Setup:**
- VM with collector connected to the server
- Use `systemctl suspend` to trigger real ACPI sleep (requires KVM)

**Procedure:**
1. Collector uploads 5 chunks, verify offset
2. Suspend VM for 10 seconds
3. Resume VM
4. Collector uploads 5 more chunks
5. Verify final offset = 10 chunks worth of data

**Expected:** Collector detects connection loss on wake, calls `GetUploadOffset`,
and resumes from last acked position.

### 3. Network Switching (WiFi ↔ Ethernet)

**Goal:** Verify collectors handle IP address changes gracefully.

**Setup:**
- VM with two network interfaces (bridged + NAT)
- Script to toggle interfaces during upload

**Procedure:**
1. Start upload on interface A
2. Mid-upload, disable interface A and enable interface B
3. Collector should reconnect on new IP
4. Upload completes with correct final offset

**Expected:** gRPC channel reconnects, `GetUploadOffset` returns correct
resume position, no data loss.

### 4. Android Emulator Integration

**Goal:** Verify the collector works on Android with realistic constraints.

**Setup:**
- Android emulator (API 34+) with collector APK
- `adb` for scripting interactions

**Procedure:**
1. Install and start collector
2. Upload screen captures via the collector
3. Toggle airplane mode mid-upload
4. Disable airplane mode
5. Verify data integrity on server

**Expected:** Android-specific lifecycle events (Doze mode, battery
optimization) don't cause data corruption.

## Manual Test Protocols

For scenarios too complex to automate:

### Protocol A: Multi-Day Soak Test
1. Deploy server + 3 collectors on real machines
2. Run continuously for 48 hours
3. Periodically check:
   - Offset drift (should be monotonically increasing)
   - Memory usage (should be stable)
   - CAS disk usage (should grow linearly)
4. At end: verify all data is queryable

### Protocol B: Chaos Monkey (Manual)
1. Deploy server + 2 collectors
2. Every 5 minutes, randomly:
   - Kill and restart a collector process
   - Temporarily block network with `iptables`
   - Restart the server process
3. After 1 hour: verify data integrity

## Infrastructure Requirements

| Resource | Specification |
|----------|--------------|
| VM host | 8+ cores, 32GB RAM, KVM support |
| VMs | 4x (1 server, 3 collectors), 2 cores / 4GB each |
| Network | Controllable bridge (for partition testing) |
| Storage | 100GB for CAS data during soak tests |
| CI runner | Self-hosted, labeled `tier3-testing` |

## Implementation Order

1. Vagrant/QEMU provisioning scripts for the VM fleet
2. Clock skew scenario (highest value, most unique to Tier 3)
3. Sleep/wake scenario
4. Network switching scenario
5. Android emulator (lowest priority, most infrastructure)

## Status

This is a **design document only**. Implementation requires dedicated hardware
and is tracked separately from the automated Tier 0-2 test suite.
