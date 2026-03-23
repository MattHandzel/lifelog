# Contributing to Lifelog

See [README.md](README.md) for a project overview.

## Prerequisites

- **Nix with flakes enabled** — required; all build commands run inside `nix develop`
- **PostgreSQL 16+** — required for integration tests and server runtime
- **Linux desktop environment** — required for most collector modalities (screen, audio, keystrokes)
- **Tesseract** — optional, needed for OCR-related tests

## Development Setup

```bash
git clone <repo-url> lifelog
cd lifelog
nix develop          # enter build environment
just check           # verify everything compiles
```

Start PostgreSQL for integration tests:

```bash
export LIFELOG_POSTGRES_INGEST_URL=postgresql://lifelog@127.0.0.1:5432/lifelog
just test-e2e
```

## Common Commands

| Command           | What it does                              |
| ----------------- | ----------------------------------------- |
| `just check`      | Type-check all targets                    |
| `just test`       | Run unit tests                            |
| `just test-e2e`   | Run integration suite (needs PostgreSQL)  |
| `just validate`   | Full gate: fmt + check + clippy + test    |
| `just run-server` | Start the lifelog gRPC server             |

Always run `just validate` before submitting a PR.

## Architecture Overview

**Proto-first:** `.proto` files are the single source of truth for all types. Do not modify proto files unless you intend to cascade the change through every crate.

**Unified frames table:** All data modalities are stored in a single `frames` table in PostgreSQL. Modality-specific data goes in a `payload JSONB` column. Binary data is stored in a content-addressable store (CAS) referenced by `blob_hash`.

**Catalog system:** A `catalog` table tracks registered origins (collectors and data sources).

**Privacy tiers:** Modalities are classified as sensitive (keystrokes, audio, clipboard), moderate (screenshots, browser), or low (weather, processes). Transforms and exports must respect per-tier policies.

## Commit Style

```
type: short description
```

Types: `feat`, `fix`, `refactor`, `docs`, `tests`, `build`

Keep the subject line under 72 characters. No period at the end.

## Pull Request Process

1. Fork and create a feature branch.
2. Make your changes. Run `just validate` — it must pass cleanly.
3. Write or update tests for any changed behavior.
4. Open a PR against `main` with a clear description of what and why.
5. Address any review feedback.

## Anti-Patterns to Avoid

- **Never run raw `cargo`** without `nix develop` — native deps won't resolve.
- **Don't `unwrap()` in library code** — use `?` or typed errors.
- **Don't add env var reads** — all configuration comes from `lifelog-config.toml`; env vars are optional overrides only.
- **Don't touch proto files** unless required — changes cascade to every crate.
- **No secrets in commits** — never commit API keys, tokens, or credentials.
- **Don't add `interface/src-tauri` to `default-members`** — breaks CI.
- **No `unsafe`** — existing unsafe is legacy debt being removed.

## Testing

- `just test` — unit tests, no external dependencies needed
- `just test-e2e` — full integration suite, requires a running PostgreSQL instance
- `server/tests/ocr_pipeline.rs` — end-to-end OCR transform test
- `server/tests/cross_modal_query.rs` — unified search verification

## Questions

Open an issue or start a discussion on the repository.
