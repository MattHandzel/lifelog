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
- Transitional SurrealDB variables (still required for non-migrated paths during hybrid phase):
  - `LIFELOG_DB_ENDPOINT` (default from code: `127.0.0.1:7183`)
  - `LIFELOG_DB_USER` (required in hybrid mode)
  - `LIFELOG_DB_PASS` (required in hybrid mode)
- Optional:
  - `LIFELOG_HOST` (default `127.0.0.1`)
  - `LIFELOG_PORT` (default `7182`)
  - `LIFELOG_CAS_PATH` (default `~/lifelog/cas`)
  - `LIFELOG_TLS_CERT_PATH`, `LIFELOG_TLS_KEY_PATH` (enable TLS only when both are set)

Example:

```bash
export LIFELOG_POSTGRES_INGEST_URL=postgresql://lifelog@127.0.0.1:5432/lifelog
export LIFELOG_POSTGRES_INGEST_MAX_CONNECTIONS=16
export LIFELOG_DB_ENDPOINT=127.0.0.1:7183
export LIFELOG_DB_USER=root
export LIFELOG_DB_PASS=root
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

### 4.1 SurrealDB (Transitional Hybrid Mode)

Current server still keeps a SurrealDB path for non-migrated modalities. Run SurrealDB if your deployment still uses hybrid mode:

```bash
nix develop --command surreal start --bind 127.0.0.1:7183 file:/tmp/lifelog-surreal.db
```

## 5. Run Server

In another terminal:

```bash
just run-server
```

This starts `lifelog-server-backend` (gRPC + gRPC-web).

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
- Server fails at startup with Surreal auth error in hybrid mode:
  - Ensure `LIFELOG_DB_USER` and `LIFELOG_DB_PASS` are exported in the server terminal.
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
  - `deploy/systemd/lifelog-surrealdb.service`
  - `deploy/systemd/lifelog-server.service`
- Remote user-service fallback (for hosts where `/etc/systemd/system` is read-only):
  - `deploy/systemd-user/lifelog-surrealdb.service`
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
- Installs and enables remote boot services (`postgresql` when present, `lifelog-surrealdb`, `lifelog-server`)
  - Uses system-level units when writable, otherwise user-level units + linger
- Installs local user collector service + validation timer
- Enables linger (`loginctl enable-linger`) so user services survive reboot/login boundaries
- Installs unified config to:
  - `~/.config/lifelog/lifelog-config.toml`
  - `<repo>/lifelog-config.toml`

### 11.3 Service Operations

Remote server host:

```bash
ssh matth@server.matthandzel.com 'sudo systemctl status postgresql lifelog-surrealdb lifelog-server --no-pager'
ssh matth@server.matthandzel.com 'sudo systemctl restart postgresql lifelog-surrealdb lifelog-server'
ssh matth@server.matthandzel.com 'sudo journalctl -u postgresql -u lifelog-surrealdb -u lifelog-server -f'
```

If using remote user-level fallback:

```bash
ssh matth@server.matthandzel.com 'systemctl --user status lifelog-surrealdb lifelog-server --no-pager'
ssh matth@server.matthandzel.com 'systemctl --user restart lifelog-surrealdb lifelog-server'
ssh matth@server.matthandzel.com 'journalctl --user -u lifelog-surrealdb -u lifelog-server -f'
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
ssh matth@server.matthandzel.com 'cd /home/matth/Projects/lifelog && printf "SELECT stream_id, count() AS n FROM upload_chunks GROUP BY stream_id;\n" | nix develop --command surreal sql --endpoint http://127.0.0.1:7183 --user root --pass root --namespace lifelog --database main --hide-welcome --json'
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
    - `deploy/systemd/lifelog-surrealdb.service`
    - `deploy/systemd-user/lifelog-server.service`
    - `deploy/systemd-user/lifelog-surrealdb.service`

- Collector endpoint:
  - `SERVER_ADDR=http://100.118.206.104:7182`
  - file: `deploy/systemd-user/lifelog-collector.service`

- Local data/output paths and laptop id:
  - `id = "laptop"`
  - output directories under `/home/matth/lifelog/data/...`
  - file: `deploy/config/lifelog-config.laptop.toml`

- Database credentials:
  - `LIFELOG_POSTGRES_INGEST_URL=postgresql://lifelog@127.0.0.1:5432/lifelog`
  - `LIFELOG_DB_USER=root`
  - `LIFELOG_DB_PASS=root`
  - change all values for any non-dev/shared deployment.

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
