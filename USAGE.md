# Lifelog Usage Guide

This guide covers installation, configuration, running the system, and testing.

## 1. Prerequisites

- Nix with flakes enabled
- Git
- Linux desktop environment (for most collector modalities)
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

## 3. Configure Server

The current backend uses these environment variables:

- `LIFELOG_DB_ENDPOINT` (default from code: `127.0.0.1:7183`)
- `LIFELOG_DB_USER` (required)
- `LIFELOG_DB_PASS` (required)
- Optional:
  - `LIFELOG_HOST` (default `127.0.0.1`)
  - `LIFELOG_PORT` (default `7182`)
  - `LIFELOG_CAS_PATH` (default `~/lifelog/cas`)
  - `LIFELOG_TLS_CERT_PATH`, `LIFELOG_TLS_KEY_PATH` (enable TLS only when both are set)

Example:

```bash
export LIFELOG_DB_ENDPOINT=127.0.0.1:7183
export LIFELOG_DB_USER=root
export LIFELOG_DB_PASS=root
```

## 4. Start SurrealDB

Run SurrealDB in a separate terminal:

```bash
nix develop --command surreal start --bind 127.0.0.1:7183 memory
```

Or for persistent local storage:

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

Collector config file path:

- `~/.config/lifelog/config.toml` (auto-created on first run)

Run collector:

```bash
just run-collector
```

You can also override server address at launch:

```bash
nix develop --command cargo run -p lifelog-collector --bin lifelog-collector -- --server-address http://127.0.0.1:7182
```

### 6.1 Enable Desktop Microphone and Keystrokes

Edit `~/.config/lifelog/config.toml` and set:

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
sudo systemctl start surrealdb
sudo systemctl start lifelog-server
sudo systemctl start lifelog-collector
```

## 10. Troubleshooting

- Server fails at startup with DB auth error:
  - Ensure `LIFELOG_DB_USER` and `LIFELOG_DB_PASS` are exported in the server terminal.
- Collector starts but does not upload:
  - Verify server is reachable at `http://<host>:<port>` and config `id` is set.
- Interface cannot query:
  - Check `VITE_API_BASE_URL` and confirm server is running.
- E2E tests are flaky when run concurrently:
  - Use the provided exclusive recipe (`just test-e2e-exclusive`) when needed.
