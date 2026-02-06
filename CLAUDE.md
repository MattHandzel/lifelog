# Lifelog — Agent & Developer Guide

## Build Environment

**Nix is required.** All cargo commands must run inside `nix develop`. Use justfile
recipes (which wrap nix for you) instead of raw cargo.

```bash
# Enter the nix shell (needed only if running cargo directly)
nix develop

# Preferred: use just recipes instead
just check       # cargo check --all-targets
just test        # cargo test --all-targets
just test-e2e    # integration suite (needs SurrealDB running)
just validate    # fmt + check + clippy + test (full gate)
```

## Operating Instructions

1. Open only the minimum files needed.
2. Run noisy commands through `tools/ai/run_and_digest.sh` and share only the digest.
3. Summarize diffs with `tools/ai/git_diff_digest.sh`.

## Command Reference

| Command              | What it does                                     |
| -------------------- | ------------------------------------------------ |
| `just check`         | `cargo check --all-targets` (fast compile check) |
| `just test`          | `cargo test --all-targets` (unit + lib tests)    |
| `just test-e2e`      | Integration suite against live SurrealDB         |
| `just validate`      | Full quality gate (fmt, check, clippy, test)     |
| `just run-server`    | Start the lifelog server backend                 |
| `just run-collector` | Start the collector daemon                       |
| `just clean-tests`   | Remove `/tmp/lifelog-test-*` artifacts           |

## Architecture Overview

**Proto-first:** `.proto` files are the single source of truth for all Config, State,
and DataModality types. Rust types are generated via `prost`/`tonic-build`. No manual
duplicates.

### Dependency Graph

```
proto/lifelog.proto, lifelog_types.proto
    └─> common/lifelog-proto     (generated code)
        └─> common/lifelog-types (re-exports + domain types: DataOrigin, LifelogError, etc.)
            ├─> common/config    (ServerConfig, CollectorConfig, PolicyConfig)
            ├─> common/utils     (CAS, chunking, ingestion)
            ├─> common/data-modalities (ScreenFrame, BrowserFrame, OcrFrame + transforms)
            ├─> common/lifelog-core    (time skew, replay, correlation — pure algorithms)
            └─> server / collector     (binaries)
```

### Workspace Crates

| Crate                    | Purpose                                     |
| ------------------------ | ------------------------------------------- |
| `common/lifelog-proto`   | Protobuf codegen (build.rs generates code)  |
| `common/lifelog-types`   | Core domain types, error types, traits      |
| `common/config`          | Configuration structs (proto-backed)        |
| `common/utils`           | CAS storage, chunk validation, ingestion    |
| `common/data-modalities` | Frame types + OCR transform                 |
| `common/lifelog-core`    | Pure algorithms (no IO)                     |
| `common/macros`          | Procedural macros                           |
| `server`                 | gRPC server, SurrealDB backend, policy loop |
| `collector`              | Device data collection daemon               |
| `interface/src-tauri`    | Desktop UI (excluded from default builds)   |

## Coding Conventions

- **Formatting:** `cargo fmt` (enforced by pre-commit hook)
- **Linting:** `cargo clippy -- -D warnings`
- **Naming:** snake_case for functions/variables, PascalCase for types
- **Commit style:** `type: short description` — types: `feat`, `fix`, `refactor`, `docs`, `tests`, `build`
- **Error handling:** Use `thiserror` in libraries, `anyhow` only in binary crates
- **Proto changes:** Regenerate with `cargo build -p lifelog-proto` after editing `.proto` files

## Conflict Zones (Coordinate Before Touching)

These files are high-contention — changes here cascade widely or block parallel work:

- **`proto/lifelog.proto`** and **`proto/lifelog_types.proto`** — regenerates all crates
- **`server/src/server.rs`** — monolith with gRPC, DB, sync, transforms, query
- **`Cargo.toml` / `Cargo.lock`** — workspace-wide dependency changes
- **`STATUS.md` / `PLAN.md`** — merge conflicts on concurrent updates

## Safe Zones (Low Conflict Risk)

- **`common/lifelog-core/`** — pure algorithms, no external deps
- **`common/utils/`** — self-contained utilities (CAS, chunking)
- **`common/data-modalities/`** — frame type definitions
- **New modules in `server/src/`** — adding new files doesn't conflict

## Worktree Instructions

For parallel agent work, use git worktrees:

```bash
# Create a worktree for a task
just worktree-create my-task-name

# List active worktrees
just worktree-list

# Remove when done
just worktree-remove my-task-name

# Merge agent work back (from main worktree)
just merge-agent my-task-name
```

**Conventions:**

- Branch naming: `agent/<kebab-case-description>`
- All branches fork from current working branch
- Worktrees live at `../lifelog-worktrees/<name>`
- Run `just validate` in the worktree before reporting done

## AI Tools (`tools/ai/`)

| Tool | Use instead of | Example |
|------|---------------|---------|
| `check_digest.sh` | raw `cargo check` | `tools/ai/check_digest.sh nix develop --command cargo check --all-targets` |
| `run_and_digest.sh` | raw command execution | `tools/ai/run_and_digest.sh -- just test` |
| `scope_changes.sh` | grep count for scoping | `tools/ai/scope_changes.sh 'println!' collector/src/` |
| `file_summary.sh` | reading full files | `tools/ai/file_summary.sh server/src/server.rs` |
| `git_diff_digest.sh` | `git diff` | `tools/ai/git_diff_digest.sh` |
| `bulk_replace.sh` | sequential Edit calls | `tools/ai/bulk_replace.sh 'old' 'new' src/*.rs` |

## Anti-Patterns

- **Never run raw `cargo` without nix** — native deps (alsa, glib, tesseract) won't resolve
- **Don't touch proto files unless required** — changes cascade to every crate
- **Don't `unwrap()` in library code** — use `?` or typed errors
- **Don't add the Tauri crate to default-members** — it pulls in GUI deps that break CI
- **Don't use `unsafe` for new code** — existing `unsafe` (sysinfo) is legacy debt
- **Don't commit broken code** — run `just validate` before every commit

## Testing

- **Unit tests:** `just test` — runs all workspace tests
- **Integration tests:** `just test-e2e` — requires SurrealDB running at `127.0.0.1:7183`
- **Start SurrealDB:** `surreal start --user root --pass root --log debug rocksdb://~/lifelog/database --bind "127.0.0.1:7183"`
- **Test harness:** `server/tests/harness/mod.rs` provides `TestContext` for integration tests

## Current State

- **Branch:** `refactor/proto-first-completion`
- **Status:** See `STATUS.md` for current objectives and blockers
- **Plan:** See `PLAN.md` for task tree and phase tracking
