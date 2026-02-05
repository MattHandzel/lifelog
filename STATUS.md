# Status

## Current Objective

Implement chunk offset validation tests (UT-040) for the validation suite.

## Where To Resume

- Branch to continue from: `validation/planning`
- Prior branch with build-only changes: `validation/scaffold`

## Last Verified

- `nix develop -c cargo check` (passes)
- `nix develop -c cargo test -p utils` (passes all 7 tests, including UT-041/042)

## How To Verify (Target)

- `nix develop -c cargo check`
- `nix develop -c cargo test -p utils`

## What Changed Last

- Added `UploadChunks` and `GetUploadOffset` RPCs to `lifelog.proto`.
- Implemented `ChunkIngester` in `common/utils` with async `IngestBackend` trait.
- Added unit tests UT-041 (Idempotent Apply) and UT-042 (Durable ACK Gate) to `utils`.
- Implemented `SurrealIngestBackend` stub and `Server` integration in `lifelog-server`.
- **Proto-First Refactor**:
  - Disabled automatic overwriting of `.proto` files by `lifelog-macros`.
  - Configured `lifelog-proto` to generate `serde::Serialize`/`Deserialize` implementations using `pbjson`.
  - Replaced `common/config::ServerConfig` with `lifelog_proto::ServerConfig` to eliminate type duplication and manual conversion.
  - Updated `lifelog-types` and `data-modalities` to compatibility with `pbjson_types`.

## What's Next

- Migrate remaining config structs (`CollectorConfig`, etc.) to use `lifelog_proto` types.
- Implement integration test scaffolding (IT-090) in `tests/validation_suite.rs`.
- Wire up real SurrealDB metadata persistence in `SurrealIngestBackend`.

## Blockers

- None, assuming `nix develop` is available for native deps on Linux.
