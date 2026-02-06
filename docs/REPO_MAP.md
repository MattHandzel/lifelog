# Repo Map (Lifelog)

This repo is a Rust workspace plus a Vite/React UI. The system collects device data (collector), stores/processes it (server + common crates), and exposes an interface (web/desktop UI).

Use this file as the stable navigation surface. For deeper design notes, browse `docs/` selectively.

## Top-Level Layout

- `common/`: shared Rust crates (types, config, utils, proto bindings, algorithms).
- `collector/`: device-side data collection binary (`lifelog-collector`).
- `server/`: backend crate (builds `lifelog-server-backend`).
- `interface/`: Vite + React + TypeScript UI; Tauri app under `interface/src-tauri/`.
- `proto/`: `.proto` definitions; Rust bindings generated via build scripts.
- `docs/`: design notes and system documentation.
- `tests/`: Rust integration-test scaffolding (limited, evolving).
- `tools/ai/`: output-digest scripts to prevent log/token blowups.

## Entry Points (Commonly Touched)

Rust workspace:

- `Cargo.toml`: workspace members and dependencies.
- `justfile`: common workflows (wraps `nix develop --command ...`).

Server:

- `server/src/server.rs`: main backend orchestration.
- `server/src/grpc_service.rs`: gRPC service layer.
- `server/tests/validation_suite.rs`: end-to-end validation test.

Collector:

- `collector/src/main.rs`: collector entry.
- `collector/src/collector.rs`: collector core.

Interface:

- `interface/src/App.tsx`: UI entry.
- `interface/src/lib/api.ts`: API client wiring.
- `interface/src/components/*Dashboard.tsx`: major screens.

Protos:

- `proto/lifelog.proto`, `proto/lifelog_types.proto`: schema source of truth.

## Config and Docs

- `AGENTS.md`: repo guidelines (structure, commands, style, secrets).
- `CLAUDE.md`: minimal agent workflow + strict context budgeting rules (always-loaded by some tools).
- `docs/`: architecture and design notes (open on-demand only).

## Common Commands

Preferred: use `justfile` targets.

- Fast Rust typecheck: `just check`
- Rust tests: `just test`
- Full gate: `just validate`
- E2E suite (noisy): `just test-e2e`
- Run backend: `just run-server`
- Run collector: `just run-collector`

Digest wrappers (recommended in agent workflows):

- `tools/ai/run_and_digest.sh -- just check`
- `tools/ai/run_and_digest.sh -- just validate`
- `tools/ai/run_and_digest.sh -- just test-e2e`
- `tools/ai/git_diff_digest.sh` (or `--cached`)

## Ownership and Hot Paths

If you are trying to make progress quickly, these are the usual "hot" areas:

- Data model/schema changes: `proto/` then generated bindings under `common/`.
- Storage/ingest plumbing: `common/utils/`, `server/`.
- Device capture modules: `collector/src/modules/`.
- UI dashboards: `interface/src/components/`.

Areas that are easy to accidentally load and waste tokens:

- `target/` (huge build artifacts, never open in an LLM context).
- Large design/spec markdown files at repo root (`SPEC*.md`, `VALIDATION_SUITE.md`, etc.).
