# CLI Detection Report

## Binaries

| Binary | Crate | Type | Description |
|--------|-------|------|-------------|
| `lifelog-server` | server | Long-running daemon | gRPC server, transform pipeline, query engine |
| `lifelog-collector` | collector | Long-running daemon | Device data collection (screen, camera, audio, keys, processes) |
| `lifelog-mcp` | mcp | Service | Model Context Protocol server for AI tool integration |
| `lifelog-export` | export | CLI tool | Data export utility |
| `lifelog-server-frontend` | interface/src-tauri | Desktop app | Tauri GUI (not in default-members) |

## CLI Arguments (known from USAGE.md and source)

### lifelog-server
- Reads config from `lifelog-config.toml` or `~/.config/lifelog/lifelog-config.toml`
- Env overrides: `LIFELOG_HOST`, `LIFELOG_PORT`, `LIFELOG_CAS_PATH`, `LIFELOG_TLS_CERT_PATH`, `LIFELOG_TLS_KEY_PATH`, `LIFELOG_POSTGRES_INGEST_URL`
- Default port: 7182

### lifelog-collector
- `--server-address <url>` - Override server address
- Env: `LIFELOG_COLLECTOR_ID` - Select which collector config section to use
- Env: `LIFELOG_CONFIG_PATH` - Override config file location

### lifelog-export
- (Needs `--help` capture for full flag inventory)

### lifelog-mcp
- (Needs `--help` capture for full flag inventory)

## Just Recipes (Developer CLI)

| Recipe | Purpose |
|--------|---------|
| `just check` | `cargo check --all-targets` (digest mode for agents) |
| `just test` | `cargo nextest run --all-targets` |
| `just test-e2e` | Integration suite (validation, multi-device, sync) |
| `just test-ui` | Frontend Vitest suite |
| `just validate` | Full gate: fmt + check + clippy + test |
| `just validate-all` | Backend + frontend validation |
| `just run-server` | Start lifelog-server |
| `just run-server-tls` | Start with TLS (cert_path, key_path args) |
| `just run-collector` | Start lifelog-collector |
| `just watch` | Bacon continuous check |
| `just test-chaos` | Docker Compose chaos tests |
| `just test-vm` | MicroVM E2E test |
| `just build-vm` | Build MicroVM config |
| `just worktree-create <name>` | Create isolated feature worktree |
| `just worktree-remove <name>` | Clean up worktree |
| `just ship-feature <name>` | Validate + merge + push feature |
| `just work` | Developer dashboard |
| `just status-all` | All worktree statuses |
| `just diff-digest` | Token-efficient diff summary |
| `just summary <file>` | File API summary |

## Systemd Services (deploy/)

| Service | Location |
|---------|----------|
| lifelog-server.service | `deploy/systemd/` and `deploy/systemd-user/` |
| lifelog-collector.service | `deploy/systemd-user/` |
| lifelog-ingest-validate.service | `deploy/systemd-user/` (with timer) |

## Helper Scripts

| Script | Purpose |
|--------|---------|
| `scripts/install_persistent_services.sh` | One-shot systemd installer |
| `scripts/run_collector_service.sh` | Collector wrapper for systemd |
| `scripts/gen-proto-types.sh` | Generate TypeScript proto types |
| `tools/ai/run_and_digest.sh` | Digest noisy command output |
| `tools/ai/check_digest.sh` | Digest cargo check output |
| `tools/ai/scope_changes.sh` | Find symbol usage sites |
| `tools/ai/bulk_replace.sh` | Large-scale rename |
| `tools/ai/summarize_output.sh` | Summarize streaming output |
