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

- `cargo check`: fast compile/typecheck for the Rust workspace (matches CI intent).
- `cargo build --release`: build optimized binaries into `target/release/`.
- `cargo run -p lifelog-server --bin lifelog-server-backend`: run the server backend.
- `cargo run -p lifelog-collector`: run the collector.
- `cargo test`: run Rust unit/integration tests (add tests as features stabilize).
- `nix develop`: enter a dev shell with native deps (e.g., protobuf/tesseract on Linux).
- `nix build .#lifelog-server` / `nix build .#lifelog-collector`: reproducible builds on Linux.
- `cd interface && npm ci && npm run dev`: run the web UI locally.
- `cd interface && npm run build`: typecheck (`tsc`) and build the UI bundle.

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
