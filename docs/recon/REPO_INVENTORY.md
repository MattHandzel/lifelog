# Repo Inventory

## Workspace Members (Cargo.toml)

| Crate | Path | Binary | Description |
|-------|------|--------|-------------|
| lifelog-server | `server/` | `lifelog-server` | gRPC backend, transforms, query engine |
| lifelog-collector | `collector/` | `lifelog-collector` | Device-side data collection daemon |
| lifelog-mcp | `mcp/` | `lifelog-mcp` | Model Context Protocol server |
| lifelog-export | `export/` | `lifelog-export` | Data export CLI |
| lifelog-config | `common/config/` | (library) | Unified TOML config parsing |
| lifelog-core | `common/lifelog-core/` | (library) | Error types, shared logic |
| lifelog-types | `common/lifelog-types/` | (library) | Proto-generated Rust types |
| data-modalities | `common/data-modalities/` | (library) | Modality definitions and traits |
| lifelog-utils | `common/utils/` | (library) | CAS, hashing, helpers |
| interface (src-tauri) | `interface/src-tauri/` | `lifelog-server-frontend` | Tauri desktop app (Rust side) |

## Frontend (interface/)

- **Framework:** Vite + React 18 + TypeScript
- **Desktop wrapper:** Tauri v2 (WebKitGTK on Linux)
- **UI library:** Radix UI + Tailwind CSS + shadcn/ui components
- **State:** React hooks (no Redux/Zustand)
- **Routing:** react-router-dom v6 (client-side, view-based navigation)
- **API layer:** gRPC-web via Tauri invoke commands

## Proto Definitions

- `proto/lifelog.proto` - Service RPCs
- `proto/lifelog_types.proto` - Message/enum types

## Infrastructure

- **Build:** Nix flake + `justfile` recipes
- **Database:** PostgreSQL 16+ (unified `frames` table)
- **Deployment:** NixOS modules, systemd services, Docker Compose (chaos tests)
- **CI:** Nix-based validation gate (`just validate`)

## Key Documentation

| Doc | Purpose |
|-----|---------|
| `README.md` | Project overview, build instructions |
| `USAGE.md` | Full usage guide (config, run, test, deploy) |
| `CLAUDE.md` | AI agent workflow instructions |
| `docs/REPO_MAP.md` | Navigation surface for developers |
| `docs/server.md` | Server architecture |
| `docs/collector.md` | Collector architecture |
| `docs/interface.md` | Interface design |
| `docs/CONFIG.md` | Configuration reference |
| `docs/SETUP_TLS.md` | TLS setup guide |
| `docs/vision.md` | Project vision |
| `docs/features-roadmap.md` | Feature roadmap |
| `docs/plans/` | Implementation plans (12 files) |
| `docs/ops/` | Operations runbooks (migration, backup, log rotation) |
