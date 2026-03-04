# Lifelog — Agent & Developer Guide

## Build Environment

**Nix is required.** All cargo commands must run inside `nix develop`.
Use `just` recipes (which wrap nix) instead of raw cargo.

**Worktrees:** Feature branches use git worktrees.

- **Standard Location:** `worktrees/feature/<branch-name>` (ignored by git).
- **Alternative:** `~/.config/superpowers/worktrees/lifelog/` (legacy/external).

| Command                       | What it does                                   |
| ----------------------------- | ---------------------------------------------- |
| `just work`                   | Display developer dashboard & active agents    |
| `just status-all`             | List all active worktrees & branch status      |
| `just init-session`           | Prepare workspace for new AI session           |
| `just worktree-list`          | List all active worktrees                      |
| `just worktree-create <name>` | Create a new feature branch and local worktree |
| `just worktree-remove <name>` | Safely remove a worktree and its branch        |
| `just review-feature <name>`  | Read agent handoff report & change digest      |
| `just ship-feature <name>`    | Validate, merge, and prune a feature worktree  |
| `just check`                  | `cargo check --all-targets`                    |
| `just test`                   | `cargo test --all-targets`                     |
| `just test-e2e`               | Integration suite (needs SurrealDB running)    |
| `just validate`               | Full gate: fmt + check + clippy + test         |
| `just run-server`             | Start the lifelog server                       |
| `just run-collector`          | Start the collector daemon                     |

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

## AI Token-Efficient Workflow

For coding agents (Claude Code, Gemini CLI, etc.), the repo includes a small context surface and output digests to reduce token usage. You MUST use these tools to prevent context bloat:

- **`IS_LLM_AGENT` (Env Var)**

  - _Effect:_ When set to `1` (default for new agents), `just check`, `just test`, and `just validate` automatically use the digest tools below. `just init-session` also skips the mandatory build check to speed up startup.
  - _Toggle:_ Run `export IS_LLM_AGENT=0` to disable and get full verbose output.

- **`tools/ai/run_and_digest.sh "<command>"`**
  ...

  - _Why:_ Commands like `cargo build` or `npm run dev` output hundreds of lines of noise, blowing up your context window and causing you to forget previous instructions.
  - _When:_ Use this whenever you need to compile code, run a test suite, or start a server where you only care about the final status or the actual errors.
  - _How:_ `tools/ai/run_and_digest.sh "cargo build"` or `tools/ai/run_and_digest.sh "npm install"`.

- **`just diff-digest`**

  - _Why:_ Raw `git diff` includes unmodified context lines, import statements, and other boilerplate that wastes tokens.
  - _When:_ Use this before committing or when reviewing what changes you've made in your current branch.
  - _How:_ Just run `just diff-digest`.

- **`just summary <file>`**

  - _Why:_ Reading a 1000-line file just to find the name of a struct or a function signature is highly inefficient.
  - _When:_ Use this when exploring a new part of the codebase to get a "map" of a file's public API without reading its implementation details.
  - _How:_ `just summary server/src/query/planner.rs`.

- **`just check-digest`**

  - _Why:_ Standard type checkers produce verbose output. This script distills it down to just the actionable error messages.
  - _When:_ Run this frequently during development to ensure you haven't broken the build, especially after making surgical changes.
  - _How:_ Just run `just check-digest`.

- **`tools/ai/scope_changes.sh <symbol>`**

  - _Why:_ Changing a core symbol requires knowing every usage site to avoid regressions.
  - _When:_ Before refactoring or changing a public struct/function.
  - _How:_ `tools/ai/scope_changes.sh "DataModality"`.

- **`tools/ai/bulk_replace.sh <old> <new>`**

  - _Why:_ Replacing a string in 50 files manually blows up the context window.
  - _When:_ For large-scale renaming or refactoring.
  - _How:_ `tools/ai/bulk_replace.sh "OldType" "NewType"`.

- **`tools/ai/summarize_output.sh`**

  - _Why:_ Long-running servers or noisy logs flood the context.
  - _When:_ When monitoring a process that outputs repetitive lines.
  - _How:_ `npm run dev | tools/ai/summarize_output.sh`.

- `rg` instead of `grep`
  - Always use `rg` instead of `grep`, you will always have access to it and it will significantly improve your efficiency. Only use `grep` if you first tried `rg` and it failed.

## Compaction Rules

When compacting, always preserve:

- List of all modified files in this session
- Current task list and their status
- Any test commands run and their results
- Specific error messages being debugged

## Session Discipline

- One logical task per session. `/clear` between unrelated tasks.
- Use `/rename` to name sessions descriptively for `--resume`.
