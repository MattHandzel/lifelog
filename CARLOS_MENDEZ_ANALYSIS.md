# Carlos Mendez Operational Review (Lifelog)

Date: 2026-02-05

This review focuses on whether Lifelog can run 24/7 across a heterogeneous personal-device fleet with minimal friction and minimal risk of silent data loss. It covers both the **current repo reality** and the **v1 intent** in `SPEC.md`.

## Executive Summary

Operationally, the project is not deployable as a “quiet always-on” system yet.

The biggest blockers are:

1. **No installation/service-management story** (no systemd/launchd/Windows Service integration; `collector/src/install.rs` is a TODO stub).
2. **Networking model is not NAT/roaming-safe today**: the server dials collectors by their `host:port`, requiring inbound reachability to the collector (`server/src/server.rs`), which contradicts `SPEC.md`’s collector-initiated connectivity requirement.
3. **Collector buffering is explicitly in-memory (and unbounded)** for at least screen capture (`collector/src/modules/screen.rs`), contradicting `SPEC.md`’s durable disk buffering guarantee and creating silent-loss risk on crash/reboot.
4. **Config bootstrapping is fragile**: non-dev runs require a pre-existing `~/.config/lifelog/config.toml` and will panic if missing (`common/config/src/lib.rs`), which is high-friction for new device installs.
5. **No monitoring/alerting pipeline**: there is no persistent heartbeat/last-seen model, no alert thresholds, no metrics endpoint, no log shipping, and no self-healing beyond “try to reconnect” loops.

The repo does have building blocks worth keeping:

- A Rust workspace with shared crates (`Cargo.toml` workspace members).
- Nix flake packaging for `lifelog-server` and `lifelog-collector` on a few systems (`flake.nix`).
- A gRPC proto defining Register/State/Config primitives (`proto/lifelog.proto`) and a collector-side periodic `ReportState` loop (`collector/src/collector.rs`).

## What `SPEC.md` Says (Ops-Relevant)

Key operational requirements that matter here:

- “Passive capture, zero maintenance” with failures surfaced as quiet alerts (`SPEC.md` section 1.3).
- NAT-safe model: collectors initiate outbound connections; backend issues commands over control channel (`SPEC.md` section 5.3).
- “Store everything, don’t lose anything”: collectors buffer **durably on disk**, resumable uploads, durable acks (`SPEC.md` section 6).
- Observability: backend exposes collector status/backlog/last pull/errors; collectors expose capture status/buffer fullness/last capture/last ack (`SPEC.md` section 14).

Today’s implementation diverges on the most ops-critical parts (install/run-as-service, NAT-safe connectivity, durable buffering).

## Deployment (New Device Install, Packaging, Cross-Compilation)

### Current State

- Primary “install” path is **build from source** (`README.md` suggests `cargo build --release`).
- Nix provides build outputs for:
  - `x86_64-linux`
  - `x86_64-darwin`
  - `aarch64-darwin`
  (`flake.nix`)
- No packaged artifacts for:
  - Windows (CI matrix comments out `windows-latest`)
  - Debian/RPM/Arch packages
  - Homebrew formula/cask
  - Android/iOS collectors
  - Embedded targets (armv7/aarch64-linux-musl, etc.)

### Operational Issues

- “Build from source + manual config + manual service setup” is not compatible with “invisible, reliable, self-healing”.
- For a fleet, you need a repeatable “enroll this device” flow and repeatable upgrades.

### Recommendations

- Decide v1 packaging targets explicitly:
  - Linux: `deb` + systemd unit, and/or a single static binary with a `systemd --user` unit.
  - macOS: signed/notarized pkg or a Homebrew cask; `launchd` plist.
  - Windows: MSI + Windows Service (or Scheduled Task for per-user).
- Keep Nix for dev and for power users, but do not treat it as the only distribution channel.
- Add CI for cross-build + artifact publishing (GitHub Releases) for the supported targets, even if unsigned initially.

## Service Management (systemd/launchd/Windows Services, Auto-Restart)

### Current State

- No systemd unit, no launchd plist, no Windows service wrapper.
- `collector/src/install.rs` is a TODO placeholder.
- Collector process tries to avoid duplicates by counting processes with `sysinfo` (`collector/src/setup.rs` + `collector/src/main.rs`), which is not a robust single-instance mechanism and doesn’t help after crashes.

### Risks

- Silent downtime: if the collector crashes, nothing restarts it unless the user notices.
- Sleep/resume edge cases: there’s a TODO about restarting loggers on suspend/resume (`collector/src/main.rs`).

### Recommendations

- Implement `lifelog-collector install` that:
  - Writes config (or points to it).
  - Installs a service entry:
    - Linux: systemd unit with `Restart=always`, `RestartSec=…`, `WatchdogSec=…` (if you implement a watchdog ping), hardening options, and resource limits.
    - macOS: LaunchAgent with `KeepAlive`, `ThrottleInterval`.
    - Windows: Service or Scheduled Task with restart-on-failure.
- Add a lockfile-based single-instance guard (cross-platform) rather than process enumeration.

## Configuration (Central vs Per-Device, Rollouts)

### Current State

- Collector config is loaded from:
  - dev mode: `./dev-config.toml`
  - non-dev: `~/.config/lifelog/config.toml`
  (`common/config/src/lib.rs`)
- Config bootstrap is currently high-friction:
  - In non-dev mode, if the file doesn’t exist, the code panics before it can write a default (`common/config/src/lib.rs`).
- There are gRPC methods for `GetConfig`/`SetConfig` on server and collector (`proto/lifelog.proto`), and UI code that expects to edit config.
- Server `SetConfig` is stubbed (it does not apply/forward config) (`server/src/server.rs`).

### Risks

- New device provisioning requires manually creating the config file in exactly the right location.
- No notion of config versioning, staged rollout, or per-device overrides vs global defaults.
- “Config pushes” are not enforced or acknowledged end-to-end (spec wants `PushConfig` with acknowledgement semantics; proto doesn’t model that yet).

### Recommendations

- Fix bootstrapping:
  - On first run, create a default config and continue (never panic).
  - Make config path explicit via CLI flag and/or `LIFELOG_HOME_DIR`.
- Split config into:
  - Device identity + enrollment secrets
  - Capture settings (per modality)
  - Network/server discovery settings
  - Resource budgets
- Add config versioning + “applied config hash” reporting in collector state so the backend can confirm rollout.

## Monitoring (Heartbeats, Health Checks, Silent Failure Detection)

### Current State

- Collector reports state periodically and retries handshake when state reporting fails (`collector/src/collector.rs`).
- Server stores reported collector states in an in-memory `SystemState` map (`server/src/server.rs` via `report_collector_state`).
- There is no durable “last seen” persistence, no explicit heartbeat timeout handling, and no alerting mechanism.
- No metrics endpoint (Prometheus/OpenTelemetry) and no structured logging guidance for production.

### Risks

- Silent data loss: a collector can stop capturing (permissions revoked, screen capture command missing, audio device changed) and the user will not be notified unless they manually inspect logs.
- Server restarts lose visibility history (state is memory-only).

### Recommendations

- Add an explicit heartbeat model:
  - Collector sends `state.timestamp`, `last_successful_capture_ts` per modality, `buffer_bytes`, `buffer_oldest_ts`, `buffer_newest_ts`.
  - Server persists per-collector health rows and computes `last_seen`.
  - Define alert thresholds (e.g., “no screen frames for > 10 minutes while enabled”).
- Expose a local health endpoint:
  - Collector: `GET /healthz` or gRPC Health Checking service.
  - Server: health + `/metrics`.
- Add a “quiet alerts” surface in the UI and optionally OS notifications.

## Resource Management (CPU/Mem/Disk Budgets)

### Current State

- Server has a policy config with max CPU/memory/thread targets (`common/config/src/policy_config.rs`), but it is not enforcing collector budgets and appears unused for hard limiting.
- Collector screen capture buffers full PNG bytes in memory in a growing `Vec` with no cap (`collector/src/modules/screen.rs`).
- Screen capture uses external programs (`grim` / `screencapture`) which introduces dependency and runtime failure modes and likely higher CPU overhead.

### Risks

- Memory ballooning and OOM kills (worst case: takes down the collector, losing buffered data because it’s in-memory).
- Disk usage is unmanaged: output directories exist but no retention/rotation model for raw blobs, WALs, or transform artifacts.

### Recommendations

- Define per-modality budgets:
  - Max CPU%, max RAM, max disk for buffer/WAL, max bandwidth per hour/day.
- Implement bounded buffers:
  - Prefer disk-backed WAL/ring segments for large payloads (screen/audio).
  - Keep only small metadata in memory.
- Add backpressure and “when full” behavior (block, drop, degrade quality) as explicit policy knobs, as discussed in `docs/buffering.md`.

## Updates (Binary Rollout Across Fleet)

### Current State

- No updater mechanism (no self-update, no server-managed updates).
- CI builds but does not publish signed artifacts; no release process is defined (`.github/workflows/ci.yml` only builds).

### Risks

- You will accumulate a fleet of drifted versions with incompatible proto/config behavior.
- Security fixes won’t land reliably on devices.

### Recommendations

- Pick one update strategy for v1:
  - OS-native package updates (brew/apt/msi) and keep the app passive.
  - Or a simple auto-updater (collector checks backend for signed release manifest).
- Report versions in `RegisterCollector` and in periodic state so server can detect drift.

## Build System (Workspace, CI/CD, Release)

### Current State

- Cargo workspace is sensible and already split into reusable crates (`Cargo.toml`).
- CI runs on ubuntu and macOS, but:
  - Windows is disabled in matrix.
  - Tests are commented out.
  - Linux uses `nix build` for server/collector only; macOS does `cargo build --release` for the whole workspace.
  (`.github/workflows/ci.yml`)

### Recommendations

- Turn tests on (at least unit + basic integration).
- Add packaging jobs per OS with reproducible artifacts.
- Add a proto compatibility check (buf lint or prost compile in CI) and enforce API stability rules.

## Networking (Discovery, NAT/Firewall, Enrollment)

### Current State

- Collector calls `RegisterCollector` to server, but server then creates a `CollectorServiceClient` by dialing back to `http://{collector_config.host}:{collector_config.port}` (`server/src/server.rs`).
- Collector notes: “server expects to be able to connect to the collector” (`collector/src/main.rs`).
- This is the opposite of the `SPEC.md` NAT-safe model, and it will fail for:
  - laptops on coffee shop Wi-Fi,
  - mobile hotspots,
  - CGNAT,
  - devices with local firewalls denying inbound ports,
  - IPv6-only / weird LANs.
- There is no mDNS/Bonjour discovery, no reverse tunnel, no relay, and no explicit pairing mechanism.
- Device identity is currently derived from MAC address in the collector handshake (`collector/src/collector.rs`), which will not work reliably on some platforms and is a poor long-term identity primitive.

### Recommendations

- Implement the `SPEC.md` connectivity model:
  - Collector establishes a long-lived outbound control channel (gRPC stream, WebSocket, or QUIC).
  - Backend sends “begin upload session” commands over that channel.
  - Data upload is collector-initiated and uses resumable chunking with durable offsets/acks.
- Enrollment/pairing:
  - Replace MAC-based identity with a generated device keypair + backend-issued cert (mTLS) or a PSK token during enrollment.
- Optional: LAN discovery via mDNS to simplify first-time pairing, but don’t rely on it for correctness.

## Highest-Leverage Fix List (In Priority Order)

P0 (blocks “always-on”):

- Make collector buffering durable (disk WAL) for screen/audio and cap memory usage.
- Flip networking to NAT-safe collector-initiated control plane; stop requiring server->collector dial-back.
- Add service units + auto-restart + install/uninstall commands for Linux/macOS first.
- Fix config bootstrapping so first-run works without manual file creation.

P1 (prevents silent loss / reduces friction):

- Persist collector health and implement “missing data” alerts.
- Add per-modality resource budgets and enforcement.
- Add version reporting + a basic update/release pipeline.

P2 (fleet polish):

- Cross-platform installers (Windows, more Linux distros), signing/notarization, mDNS-assisted pairing, and UI-driven enrollment.

