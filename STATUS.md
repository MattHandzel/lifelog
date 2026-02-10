# Status

## Current Objective

Phase 6: Query Engine Completion (correlation + replay) and UI integration.

## Last Verified

- `just check`
- `just test` (nextest; integration tests requiring SurrealDB remain `#[ignore]`)

## How To Verify (Target)

- `just validate`
- `nix develop --command cargo test -p lifelog-server --test ocr_pipeline -- --include-ignored`
- `nix develop --command cargo test -p lifelog-server --test canonical_llql_example -- --include-ignored`

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
- **Replay Backend + UI**: Added a `Replay` gRPC RPC that returns ordered screen-granularity steps plus aligned context keys, and wired an interface Replay view to it.
- **Canonical Time Wire Fields**: Added `t_device`/`t_ingest`/`t_canonical`/`t_end`/`time_quality` to frame protos and populated them in server `GetData` responses; OCR derived frames now propagate canonical time metadata from source frames.
- **Clock Skew Estimation Wired**: Added periodic clock sync over `ControlStream` so collectors report `(device_now, backend_now)` samples; server computes per-collector skew estimates and applies them at ingest to populate `t_canonical` and `time_quality`.
- **Default Correlation Window**: Added `ServerConfig.default_correlation_window_ms` and wired temporal operators to fall back to it when a query omits a per-predicate window (LLQL supports omitting `window`).
- **Query Resource Limits**: Added default resource bounds to query execution: `LIMIT 1000` on UUID-returning queries and a `10s` SurrealDB query timeout.
- **Timeline Query Mode**: Timeline UI now submits `Query.text` as a string array and supports an LLQL mode (`llql:` / `llql-json:`) for cross-modal queries.
- **Canonical LLQL Example Verified**: Added an ignored integration test that seeds Browser/OCR/Audio and runs the Spec §10.2 canonical query end-to-end via LLQL JSON.
- **Temporal Conjunction Queries**: Temporal correlation operators can now be mixed under `AND` (including multiple `WITHIN(...)` terms) by intersecting interval sets at execution time.
- **Schema DDL Validation**: Startup migrations and table schema creation now call `.check()` so SurrealDB DDL/index errors surface immediately.
- **Performance Baselines**: Established throughput and latency benchmarks via `performance_suite.rs`.
- **Improved Test Coverage**: Added unit tests for `DiskBuffer`, `TimeInterval`, `ReplayStep`, and config validation.

## What's Next

- **UI Integration**: Add query builder/templates for LLQL (and add richer previews in search/replay).
- **Canonical Query (Spec §10.2)**: Extend validation to real ingest + OCR transform + Audio capture on a live collector (beyond the integration test seed data).
- **Security**: Add pairing + auth, and enforce TLS.
- **New Modalities**: Implement missing v1 collectors (audio, mouse, window activity fallback), then gate keystrokes behind security controls.

## Blockers

- None.
