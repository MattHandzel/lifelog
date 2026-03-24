# Lifelog Usage Guide

This guide covers installation, configuration, running the system, and testing.

## 1. Prerequisites

- Nix with flakes enabled
- Git
- Linux desktop environment (for most collector modalities)
- PostgreSQL (required for migrated ingest/query paths)
- Optional for OCR-heavy tests/features: Tesseract

Clone and enter the repo:

```bash
git clone <your-fork-or-repo-url> lifelog
cd lifelog
```

## 2. Enter Development Environment

All project commands should run inside `nix develop` (or via `just`, which already wraps nix).

```bash
nix develop
```

Quick sanity check:

```bash
just check
```

## 3. Configure Server and Collector (Unified File)

Preferred config file:

- `lifelog-config.toml` (repo root in dev runs)
- or `~/.config/lifelog/lifelog-config.toml` (installed deployments)

This file can include all server, collector, transform, and device-alias config.

Required shape (strict; missing sections cause startup failure):

```toml
[runtime]
collectorId = "laptop"

[collectors.laptop]
id = "laptop"
host = "127.0.0.1"
port = 7190
timestampFormat = "%Y-%m-%d_%H-%M-%S.%3f%Z"

# Include full modality tables (not only `enabled`) for:
# browser, screen, camera, microphone, processes, hyprland
# See `lifelog-config.toml` in repo root for a complete example.

[server]
host = "0.0.0.0"
port = 7182

transforms = [
  { id = "ocr", enabled = true, sourceOrigin = "*:screen", language = "eng" }
]

[deviceAliases]
"44A3BB2C216C" = "laptop"
```

You can force a path with:

```bash
export LIFELOG_CONFIG_PATH=/path/to/lifelog-config.toml
```

For multi-device collector binaries, select the active collector config via:

```bash
export LIFELOG_COLLECTOR_ID=laptop
```

If `LIFELOG_COLLECTOR_ID` is unset, `runtime.collectorId` must be present. There is no fallback selection.

The backend supports these environment variables for runtime overrides/secrets:

- `LIFELOG_POSTGRES_INGEST_URL` (recommended default): PostgreSQL DSN, e.g. `postgresql://lifelog@127.0.0.1:5432/lifelog`
- `LIFELOG_POSTGRES_INGEST_MAX_CONNECTIONS` (optional, default `16`)
- Optional:
  - `LIFELOG_HOST` (default `127.0.0.1`)
  - `LIFELOG_PORT` (default `7182`)
  - `LIFELOG_CAS_PATH` (default `~/lifelog/cas`)
  - `LIFELOG_TLS_CERT_PATH`, `LIFELOG_TLS_KEY_PATH` (enable TLS only when both are set)

Example:

```bash
export LIFELOG_POSTGRES_INGEST_URL=postgresql://lifelog@127.0.0.1:5432/lifelog
export LIFELOG_POSTGRES_INGEST_MAX_CONNECTIONS=16
```

## 4. Start PostgreSQL (Default)

If using NixOS, prefer enabling PostgreSQL via the flake module:

```nix
{
  imports = [ inputs.lifelog.nixosModules.lifelog-postgres ];
  services.lifelog.postgres.enable = true;
}
```

For local non-NixOS dev, start PostgreSQL in a separate terminal:

```bash
nix develop --command bash -lc '
  mkdir -p /tmp/lifelog-pg &&
  if [ ! -f /tmp/lifelog-pg/PG_VERSION ]; then
    initdb -D /tmp/lifelog-pg
  fi &&
  pg_ctl -D /tmp/lifelog-pg -o "-p 5432" -l /tmp/lifelog-pg.log start &&
  psql -h 127.0.0.1 -p 5432 -d postgres -c "DO \$\$BEGIN IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = '\''lifelog'\'') THEN CREATE ROLE lifelog LOGIN; END IF; END\$\$;" &&
  psql -h 127.0.0.1 -p 5432 -d postgres -c "SELECT '\''CREATE DATABASE lifelog OWNER lifelog'\'' WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = '\''lifelog'\'')\\gexec"
'
```

## 5. Run Server

In another terminal:

```bash
just run-server
```

This starts `lifelog-server` (gRPC + gRPC-web).

## 6. Configure and Run Collector

Run collector:

```bash
just run-collector
```

You can also override server address at launch:

```bash
nix develop --command cargo run -p lifelog-collector --bin lifelog-collector -- --server-address http://127.0.0.1:7182
```

### 6.1 Enable Desktop Microphone and Keystrokes

Edit the selected collector section in `lifelog-config.toml` and set:

```toml
[microphone]
enabled = true

[keyboard]
enabled = true
```

Notes:

- Microphone captures chunked audio frames.
- Keystrokes are sensitive data; enable only when you understand the privacy impact.

## 7. Run Interface (Web UI)

From the repo root:

```bash
nix develop --command sh -lc 'cd interface && npm ci'
nix develop --command sh -lc 'cd interface && npm run dev'
```

Set `interface/.env` as needed (see `interface/.env.example`), especially `VITE_API_BASE_URL` if your server is not default.

## 8. Test and Validation

### 8.1 Main project checks

```bash
just check
just test
just validate
```

`just validate` is the full gate (`fmt`, `check`, `clippy`, `test`).

### 8.2 Integration/E2E suite

```bash
just test-e2e
```

### 8.3 New realistic dataflow tests

All modalities simulated end-to-end ingest/query/get-data:

```bash
nix develop --command cargo test -p lifelog-server --test all_modalities_dataflow -- --nocapture
```

Collector + server multi-modality flow with real collector upload manager:

```bash
nix develop --command cargo test -p lifelog-server --test collector_multi_modality_flow -- --nocapture
```

Server binary smoke E2E:

```bash
nix develop --command cargo test -p lifelog-server --test smoke_server_bin -- --nocapture
```

### 8.4 Frontend tests

```bash
just test-ui
```

## 9. Optional Systemd Installation

Build and install binaries:

```bash
just install
```

Install service units:

```bash
just install-services
```

Then start services:

```bash
sudo systemctl start postgresql
sudo systemctl start lifelog-server
sudo systemctl start lifelog-collector
```

## 10. Troubleshooting

- Server fails at startup with PostgreSQL connection error:
  - Ensure `LIFELOG_POSTGRES_INGEST_URL` points to a running PostgreSQL instance.
- Collector starts but does not upload:
  - Verify server is reachable at `http://<host>:<port>` and config `id` is set.
- Interface cannot query:
  - Check `VITE_API_BASE_URL` and confirm server is running.
- E2E tests are flaky when run concurrently:
  - Use the provided exclusive recipe (`just test-e2e-exclusive`) when needed.

## 11. Persistent Distributed Deployment (Server + Laptop Collector)

This section is for a split setup:
- Server backend on home server (`matth@server.matthandzel.com`)
- Collector on laptop
- Transport over Tailscale (`http://100.118.206.104:7182`)

### 11.1 Files Added for Persistence

- Remote system services:
  - `postgresql.service` (system PostgreSQL)
  - `deploy/systemd/lifelog-server.service`
- Remote user-service fallback (for hosts where `/etc/systemd/system` is read-only):
  - `deploy/systemd-user/lifelog-server.service`
- Local user services:
  - `deploy/systemd-user/lifelog-collector.service`
  - `deploy/systemd-user/lifelog-ingest-validate.service`
  - `deploy/systemd-user/lifelog-ingest-validate.timer`
- Collector wrapper:
  - `scripts/run_collector_service.sh`
- One-shot installer:
  - `scripts/install_persistent_services.sh`
- Unified laptop config template:
  - `deploy/config/lifelog-config.laptop.toml`

### 11.2 Install / Enable

Run from repo root:

```bash
nix develop --command bash -lc 'scripts/install_persistent_services.sh'
```

What this does:
- Installs and enables remote boot services (`postgresql` when present, `lifelog-server`)
  - Uses system-level units when writable, otherwise user-level units + linger
- Installs local user collector service + validation timer
- Enables linger (`loginctl enable-linger`) so user services survive reboot/login boundaries
- Installs unified config to:
  - `~/.config/lifelog/lifelog-config.toml`
  - `<repo>/lifelog-config.toml`

### 11.3 Service Operations

Remote server host:

```bash
ssh matth@server.matthandzel.com 'sudo systemctl status postgresql lifelog-server --no-pager'
ssh matth@server.matthandzel.com 'sudo systemctl restart postgresql lifelog-server'
ssh matth@server.matthandzel.com 'sudo journalctl -u postgresql -u lifelog-server -f'
```

If using remote user-level fallback:

```bash
ssh matth@server.matthandzel.com 'systemctl --user status lifelog-server --no-pager'
ssh matth@server.matthandzel.com 'systemctl --user restart lifelog-server'
ssh matth@server.matthandzel.com 'journalctl --user -u lifelog-server -f'
```

Laptop collector (user service):

```bash
systemctl --user status lifelog-collector lifelog-ingest-validate.timer --no-pager
systemctl --user restart lifelog-collector
journalctl --user -u lifelog-collector -f
journalctl --user -u lifelog-ingest-validate.service -n 100 --no-pager
```

### 11.4 Validation / Health Checks

Manual end-to-end ingest validation:

```bash
DURATION_SECS=35 scripts/validate_remote_ingest.sh
```

Timer validation (non-invasive):
- `lifelog-ingest-validate.timer` runs every 10 minutes.
- It uses:
  - `SKIP_COLLECTOR_RUN=1`
  - `scripts/validate_remote_ingest.sh`
- This checks remote `upload_chunks` growth for `stream_id=processes` without spawning a second collector process.

Remote DB quick check:

```bash
ssh matth@server.matthandzel.com 'psql "$LIFELOG_POSTGRES_INGEST_URL" -c "SELECT modality, count(*) FROM frames GROUP BY modality ORDER BY count DESC;"'
```

### 11.5 Known Limitations

- `ControlStream` reconnect/close events still occur; ingest can continue despite this.
- `screen` capture depends on desktop session environment and available screenshot binary.
  - Current config uses `program = "grim"` (not `"grim -t png"`).
  - If `screen` errors in service logs, set `[collectors.<collector_id>.screen].enabled = false` in `lifelog-config.toml` and keep `processes` enabled.

### 11.6 Deployment-Specific Values You Should Change Before Sharing

These values are intentionally specific to one environment and should be parameterized before wider rollout:

- Remote identity and paths:
  - `REMOTE_HOST=matth@server.matthandzel.com`
  - `REMOTE_REPO=/home/matth/Projects/lifelog`
  - service `WorkingDirectory` paths under `/home/matth/Projects/lifelog`
  - files:
    - `scripts/install_persistent_services.sh`
    - `deploy/systemd/lifelog-server.service`
    - `deploy/systemd-user/lifelog-server.service`

- Collector endpoint:
  - `SERVER_ADDR=http://100.118.206.104:7182`
  - file: `deploy/systemd-user/lifelog-collector.service`

- Local data/output paths and laptop id:
  - `id = "laptop"`
  - output directories under `/home/matth/lifelog/data/...`
  - file: `deploy/config/lifelog-config.laptop.toml`

- Database credentials:
  - `LIFELOG_POSTGRES_INGEST_URL=postgresql://lifelog@127.0.0.1:5432/lifelog`
  - change this for any non-dev/shared deployment.

- Service style fallback:
  - Installer currently tries system services first, then falls back to user services if `/etc/systemd/system` is not writable.
  - file: `scripts/install_persistent_services.sh`

- Validation cadence defaults:
  - `DURATION_SECS=45`
  - timer every `10m`
  - files:
    - `deploy/systemd-user/lifelog-ingest-validate.service`
    - `deploy/systemd-user/lifelog-ingest-validate.timer`

- Collector runtime mode:
  - Service currently runs `target/debug/lifelog-collector`.
  - For production sharing, switch to a pinned release binary path and versioned build artifact.

### 11.7 Auth Token Sync (Server ↔ Collector)

When per-request auth tokens are enabled, both the server and the collector must
use the same shared secret. Use the `environmentFile` option on both NixOS
modules to inject the token without hardcoding it in the Nix config.

**Generate a token:**

```bash
openssl rand -hex 32 > /etc/lifelog/auth_token
chmod 600 /etc/lifelog/auth_token
```

**Server env file** (e.g. `/etc/lifelog/server.env`, mode 600, owned by root):

```
LIFELOG_AUTH_TOKEN=<paste token here>
LIFELOG_TLS_CERT_PATH=/etc/lifelog/tls/cert.pem
LIFELOG_TLS_KEY_PATH=/etc/lifelog/tls/key.pem
LIFELOG_POSTGRES_INGEST_URL=postgresql://lifelog@127.0.0.1:5432/lifelog
```

Wire it in `configuration.nix`:

```nix
services.lifelog.server.environmentFile = "/etc/lifelog/server.env";
```

**Collector env file** (e.g. `~/.config/lifelog/collector.env`, mode 600, owned by the user):

```
LIFELOG_AUTH_TOKEN=<same token>
```

Wire it in the laptop's `configuration.nix` or `home-manager`:

```nix
services.lifelog.collector.environmentFile = "/home/<user>/.config/lifelog/collector.env";
```

**Distributing the token securely:**

Copy over SSH — never commit the raw token to git:

```bash
scp /etc/lifelog/auth_token <user>@<laptop>:.config/lifelog/auth_token
```

If you use [agenix](https://github.com/ryantm/agenix), encrypt the secret with
the host keys for both machines and reference the resulting `.age` file as the
`environmentFile`. This ensures the token is decrypted at boot on each host
without ever appearing in plain text in the Nix store.

### 11.8 PostgreSQL Collation Version Refresh

After a glibc or ICU upgrade the database may log a collation mismatch warning.
Run this once on the server to silence it and prevent index corruption:

```bash
psql "$LIFELOG_POSTGRES_INGEST_URL" -c "ALTER DATABASE lifelog REFRESH COLLATION VERSION;"
```

You can check whether a refresh is needed with:

```bash
psql "$LIFELOG_POSTGRES_INGEST_URL" -c "SELECT datname, datcollversion FROM pg_database WHERE datname = 'lifelog';"
```

If `datcollversion` differs from what `pg_collation_actual_version()` returns,
run the `REFRESH` statement above. This is a one-time manual operation per
system upgrade — it does not affect data.
