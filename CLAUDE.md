# Lifelog — Agent & Developer Guide

## Build Environment

**Nix is required.** All cargo commands must run inside `nix develop`.
Use `just` recipes (which wrap nix) instead of raw cargo.

**Worktrees:** Feature branches use git worktrees in `~/.config/superpowers/worktrees/lifelog/`

| Command              | What it does                                |
| -------------------- | ------------------------------------------- |
| `just check`         | `cargo check --all-targets`                 |
| `just test`          | `cargo test --all-targets`                  |
| `just test-e2e`      | Integration suite (needs SurrealDB running) |
| `just validate`      | Full gate: fmt + check + clippy + test      |
| `just run-server`    | Start the lifelog server                    |
| `just run-collector` | Start the collector daemon                  |

## Architecture

**Proto-first:** `.proto` files are the single source of truth for Config, State,
and DataModality types. Rust types generated via `prost`/`tonic-build`.

**Unified Trait Model:** `DataType` and `Modality` provide basic identity. 
`ToRecord` (guarded by `surrealdb` feature in `lifelog-types`) provides 
SurrealDB 2.x friendly record types to avoid generic `serde_json` serialization 
issues with native types like `datetime` and `bytes`.

**Catalog System:** Explicit `catalog` table in SurrealDB tracks registered 
origins. Avoids expensive `INFO FOR DB` discovery during query execution.

See @README.md for project overview.

## Conventions

- **Commit style:** `type: short description` (feat/fix/refactor/docs/tests/build)
- **Error handling:** Unified `LifelogError` in `lifelog-core`. `thiserror` in libraries, `anyhow` only in binary crates.
- **Testing:** 
  - `just test`: Unit tests.
  - `just test-e2e`: Comprehensive integration suite.
  - `server/tests/ocr_pipeline.rs`: End-to-end transformation test.
  - `server/tests/cross_modal_query.rs`: Unified search verification.
  - `server/tests/performance_suite.rs`: Throughput and latency baselines.
- **Seesion End**: At the end of the session, commit, push, and merge your changes into main.

## IMPORTANT: Anti-Patterns

- **NEVER run raw `cargo` without nix** — native deps won't resolve
- **Don't touch proto files unless required** — changes cascade to every crate
- **Don't `unwrap()` in library code** — use `?` or typed errors
- **Don't add interface/src-tauri to default-members** — breaks CI
- **Don't use `unsafe`** — existing unsafe is legacy debt
- **Don't commit broken code** — `just validate` before every commit
- **Build.rs `println!("cargo:...")` is a Cargo directive** — never replace these
- **Never `git add .`** — stage only the specific files you intend to commit
- **Never force push** — if push fails, investigate; never use `--force`
- **No silent failures** — if an operation fails, surface an error or prominent warning; never silently fall back
- **No comments in code** — unless the code is complex or the user asks; mimic the file's existing style
- **Check library availability** — before using a crate or library, verify it's already in `Cargo.toml`; never assume

## Security

- **No secrets in commits** — never commit API keys, tokens, or credentials
- **API keys via environment variables** — if a task needs an external API key, tell the user to set it as an env var; never hardcode

## Observability

- Build in meaningful logging so program state, events, and errors are easy to follow for both humans and automated review.

## Coding Standards

- **Mimic existing style** — before editing a file, understand its conventions (naming, error handling, imports) and match them.
- **Validate after changes** — after making a change, run `just validate` (or write a targeted test) to confirm correctness before finishing.

## Conflict Zones (Coordinate Before Touching)

- `proto/*.proto` — cascades to all crates
- `Cargo.toml` / `Cargo.lock` — workspace-wide
- `server/src/server.rs` — central server logic

## AI Agent Workflow

Use digest tools instead of reading raw output:

| Tool                          | Use instead of        |
| ----------------------------- | --------------------- |
| `tools/ai/check_digest.sh`    | raw cargo check       |
| `tools/ai/run_and_digest.sh`  | raw command execution |
| `tools/ai/scope_changes.sh`   | grep for scoping      |
| `tools/ai/file_summary.sh`    | reading full files    |
| `tools/ai/git_diff_digest.sh` | git diff              |
| `tools/ai/bulk_replace.sh`    | sequential Edit calls |

## Compaction Rules

When compacting, always preserve:

- List of all modified files in this session
- Current task list and their status
- Any test commands run and their results
- Specific error messages being debugged

## Session Discipline

- One logical task per session. `/clear` between unrelated tasks.
- Use `/rename` to name sessions descriptively for `--resume`.
