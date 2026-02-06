# Lifelog — Agent & Developer Guide

## Build Environment

**Nix is required.** All cargo commands must run inside `nix develop`.
Use `just` recipes (which wrap nix) instead of raw cargo.

| Command              | What it does                                 |
| -------------------- | -------------------------------------------- |
| `just check`         | `cargo check --all-targets`                  |
| `just test`          | `cargo test --all-targets`                   |
| `just test-e2e`      | Integration suite (needs SurrealDB running)  |
| `just validate`      | Full gate: fmt + check + clippy + test       |
| `just run-server`    | Start the lifelog server                     |
| `just run-collector` | Start the collector daemon                   |

## Architecture

**Proto-first:** `.proto` files are the single source of truth for Config, State,
and DataModality types. Rust types generated via `prost`/`tonic-build`.

See @README.md for project overview.

## Conventions

- **Commit style:** `type: short description` (feat/fix/refactor/docs/tests/build)
- **Error handling:** `thiserror` in libraries, `anyhow` only in binary crates
- **Testing:** `just test` for unit, `just test-e2e` for integration (needs SurrealDB at `127.0.0.1:7183`)

## IMPORTANT: Anti-Patterns

- **NEVER run raw `cargo` without nix** — native deps won't resolve
- **Don't touch proto files unless required** — changes cascade to every crate
- **Don't `unwrap()` in library code** — use `?` or typed errors
- **Don't add interface/src-tauri to default-members** — breaks CI
- **Don't use `unsafe`** — existing unsafe is legacy debt
- **Don't commit broken code** — `just validate` before every commit
- **Build.rs `println!("cargo:...")` is a Cargo directive** — never replace these

## Conflict Zones (Coordinate Before Touching)

- `proto/*.proto` — cascades to all crates
- `Cargo.toml` / `Cargo.lock` — workspace-wide
- `server/src/server.rs` — central server logic

## AI Agent Workflow

Use digest tools instead of reading raw output:

| Tool                 | Use instead of        |
| -------------------- | --------------------- |
| `tools/ai/check_digest.sh`  | raw cargo check |
| `tools/ai/run_and_digest.sh`| raw command execution |
| `tools/ai/scope_changes.sh` | grep for scoping |
| `tools/ai/file_summary.sh`  | reading full files |
| `tools/ai/git_diff_digest.sh` | git diff |
| `tools/ai/bulk_replace.sh`  | sequential Edit calls |

## Compaction Rules

When compacting, always preserve:
- List of all modified files in this session
- Current task list and their status
- Any test commands run and their results
- Specific error messages being debugged

## Session Discipline

- One logical task per session. `/clear` between unrelated tasks.
- Use `/rename` to name sessions descriptively for `--resume`.
