# Repository Guidelines

## Project Structure

- `common/`: shared Rust crates (config, types, proto bindings, utilities).
- `collector/`: device-side data collection binary (`lifelog-collector`).
- `server/`: backend crate (builds `lifelog-server-backend`).
- `interface/`: Vite + React + TypeScript UI, plus Tauri app in `interface/src-tauri/` (`lifelog-interface`).
- `proto/`: `.proto` definitions used by the Rust build scripts (requires `protoc`/protobuf tooling).
- `docs/`: design notes and system documentation.
- `tests/`: Rust integration-test scaffolding (currently TODO stubs).

## Build, Test, and Development Commands

**ALL COMMANDS MUST RUN WITHIN `nix develop`.** Prefer using `just` recipes which wrap the necessary nix context.

- `just check`: fast compile/typecheck for the Rust workspace (matches CI intent).
- `just validate`: full gate including fmt, check, clippy, and unit tests.
- `just run-server`: run the server backend.
- `just run-collector`: run the collector.
- `just test`: run Rust unit tests.
- `just test-e2e`: run full integration suite (requires SurrealDB).
- `nix develop --command cargo test -p <pkg> --test <test_name> -- --include-ignored`: run specific ignored integration tests.
- `cd interface && npm run dev`: run the web UI locally (ensure server is running).

## Coding Style & Naming Conventions

- Rust: format with `cargo fmt`; keep modules/files `snake_case.rs`, types `PascalCase`, functions/vars `snake_case`.
- TypeScript/React: `strict` TS is enabled; components `PascalCase.tsx`, hooks `useXyz.ts`.
- Prefer small, composable modules under the crate/component they belong to; avoid duplicating types already in `common/`.

## Testing Guidelines

- Use `cargo test` for Rust; place unit tests next to code (`mod tests`) and integration tests under `tests/`.
- When adding new behavior, include at least one test covering the happy path and one edge case.

## Commit & Pull Request Guidelines

- History shows short, informal subjects (often imperative) plus merge commits. Keep subjects brief and specific (e.g., `server: fix buffer clear`, `interface: add search tab`).
- PRs: describe what changed, how to test (`cargo check`, `cargo test`, `npm run build`), link issues if applicable, and include screenshots for UI changes.

## Configuration & Secrets

- Use `.env.example` files (`server/`, `interface/`) as the source of truth for required variables.
- Do not commit real secrets or local `.env` values; keep credentials out of logs and fixtures.
