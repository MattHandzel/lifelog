# Status

## Current Objective

Phase 6: Query Engine Completion (correlation + replay) and UI integration.

## Last Verified

- `just validate` (fmt + check + clippy + unit tests; integration tests requiring SurrealDB remain `#[ignore]`)

## How To Verify (Target)

- `just validate`
- `nix develop --command cargo test -p lifelog-server --test ocr_pipeline -- --include-ignored`

## What Changed Last

- **Unified Error Hierarchy**: Migrated all local error types to a single `LifelogError` enum in `lifelog-core` for consistent error handling and reporting.
- **Unified Trait Model**: Finalized `DataType`, `Modality`, and `ToRecord` traits. `ToRecord` ensures SurrealDB 2.x compatibility for complex types like `datetime` and `bytes`.
- **Explicit Catalog**: Replaced `INFO FOR DB` table discovery with a dedicated `catalog` table for robust and efficient origin discovery during queries.
- **Robust Transformation Pipeline**: Verified OCR transformation end-to-end, including watermark persistence and idempotent processing.
- **Cross-Modal Search**: Successfully implemented and verified unified search across different data sources (Screen, Browser).
- **Cross-Modal Correlation**: Added `DURING(...)` support as a two-stage query plan (source intervals -> target time-window filter) alongside `WITHIN(...)`.
- **DURING Enhancements**: `DURING(...)` now supports an explicit window expansion for point sources and conjunction of multiple `DURING(...)` terms via interval intersection.
- **Interval Overlap Semantics**: Added `t_end` metadata and updated temporal joins so interval targets (notably Audio) use overlap semantics (`t_canonical`/`t_end`) instead of “start timestamp only”.
- **OVERLAPS Operator**: Added `OVERLAPS(...)` to the typed query AST/LLQL and wired it through planner/executor (currently equivalent to `DURING(...)` execution).
- **Replay Backend**: Added a `Replay` gRPC RPC that returns ordered screen-granularity steps plus aligned context keys (UI integration pending).
- **Canonical Time Wire Fields**: Added `t_device`/`t_ingest`/`t_canonical`/`t_end`/`time_quality` to frame protos and populated them in server `GetData` responses; OCR derived frames now propagate canonical time metadata from source frames.
- **Performance Baselines**: Established throughput and latency benchmarks via `performance_suite.rs`.
- **Improved Test Coverage**: Added unit tests for `DiskBuffer`, `TimeInterval`, `ReplayStep`, and config validation.

## What's Next

- **UI Integration**: Add query builder/templates for LLQL and wire Replay view to call `Replay`.
- **Canonical Query (Spec §10.2)**: Validate end-to-end with real ingest + OCR-derived stream + Audio capture.
- **Security**: Add pairing + auth, and enforce TLS.
- **New Modalities**: Implement missing v1 collectors (clipboard, shell, mouse, window activity), then gate keystrokes behind security controls.

## Blockers

- None.
