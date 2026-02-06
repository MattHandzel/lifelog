# Validation Suite Plan (Lifelog v1)

This plan implements an automated validation suite for `SPEC.md` using `VALIDATION_SUITE.md` as the test inventory.

## Objectives

- Create a runnable, incremental validation test harness.
- Implement the “pure semantics” unit tests first (time model, replay steps, correlation).
- Add integration-test scaffolding for end-to-end requirements (gRPC + storage), but keep them `#[ignore]` until the underlying features exist.

## Task Tree

1. Test Harness Bootstrap
   - Done when: `cargo test` runs and reports at least one “validation suite” test module.
   - Deliverables:
     - `tests/validation_suite/*.rs` integration-test modules (ignored for now).
     - Documentation in `STATUS.md` with exact commands.
   - Status: Partially complete (`tests/validation_suite.rs` skeleton exists; most ITs still ignored).

2. Unit: Time Skew Estimator (UT-010)
   - Done when: a deterministic skew estimator exists in a shared crate and unit tests cover happy path + jitter/outliers.
   - Status: Implemented in `common/lifelog-core/src/time_skew.rs` with unit tests.

3. Unit: Replay Step Builder (UT-011)
   - Done when: a replay-step builder exists and unit tests cover multi-frame and single-frame windows.
   - Status: Implemented in `common/lifelog-core/src/replay.rs` with unit tests.

4. Unit: Correlation Operators (UT-020..UT-024)
   - Done when: WITHIN/OVERLAPS semantics exist for point/interval and unit tests cover worked examples.
   - Status: Implemented minimal WITHIN(point,point) + OVERLAPS(interval,interval) in `common/lifelog-core/src/correlation.rs`.

5. Unit: Chunk/Offset + CAS Primitives (UT-040, UT-041, UT-042, UT-050..UT-052)
   - Done when: byte-offset validation + SHA256 hashing + filesystem CAS put/get/dedupe + idempotent apply + ACK gate are implemented and tested.
   - Status: Implemented in `common/utils/src/cas.rs`, `common/utils/src/chunk.rs`, and `common/utils/src/ingest.rs` with unit tests.

6. Integration Scaffolding (IT-010, IT-060, IT-080, IT-090, IT-081, IT-100, IT-110, IT-130, IT-140, IT-150, IT-160)
   - Done when: there is a consistent way to spin up ephemeral backend components for tests (DB + CAS paths + config).
   - Status: Completed. `TestContext` harness implemented in `server/tests/harness/mod.rs`. `IT-090` (Resumable Upload) is fully implemented and passing.

7. Refactor: Proto-First Type System
   - **Goal:** Make `.proto` files the single source of truth.
   - **Status:** Completed. All Config and State types migrated. Manual type casting eliminated for these types. `pbjson` integrated for automatic Serde support.

## Risks / Mitigations

- Native deps on Linux (alsa, glib/gtk, tesseract, etc.) can make default `cargo test` brittle.
  - Mitigation: keep UI crate out of default workspace builds; ensure tests can run inside `nix develop`.
- Integration tests will need controllable, deterministic time and storage state.
  - Mitigation: build small “test fixtures” modules that seed DB/CAS deterministically.

## Verification Plan

- L1: `nix develop -c cargo check`
- L3: `nix develop -c cargo test`
- L4: AC replay: each implemented test links back to `VALIDATION_SUITE.md` section/test ID in comments.

## Phase 2: Architectural Consolidation

This phase addresses architectural debt and "Dual-Source of Truth" issues identified during the initial refactor.

### 1. Connection Model: Collector-Initiated Control Stream
- **Goal:** Solve NAT/Firewall traversal by eliminating the "Dial-Back" model.
- **Tasks:**
  - [ ] Implement long-lived bidirectional gRPC stream for Control Plane (Collector -> Server).
  - [ ] Move `ReportState` and `RegisterCollector` into the stream initiation.
  - [ ] Implement Server-to-Collector commands (e.g., "Begin Upload") as messages over the long-lived stream.
  - [ ] Remove `CollectorService` gRPC server from the collector process.

### 2. Explicit Database Schema & Migrations
- **Goal:** Eliminate `ensure_table` on-the-fly creation for predictable query planning.
- **Tasks:**
  - [ ] Create a `schema/` directory or module containing explicit SurrealDB DDL for all modalities.
  - [ ] Implement a startup migration/initialization task in the Server that ensures all tables and indices exist.
  - [ ] Move `get_surrealdb_schema()` from the `Modality` trait to a centralized schema registry.

### 3. Error Type Consolidation
- **Goal:** Unify `LifelogError`, `ServerError`, `CollectorError`, etc., into a cohesive hierarchy.
- **Tasks:**
  - [ ] Define a master `LifelogError` in `common/lifelog-types`.
  - [ ] Use `thiserror` to wrap lower-level errors (IO, gRPC, DB) with semantic context.
  - [ ] Refactor modules to use the unified error type where appropriate, reducing reliance on `anyhow::Error`.

### 4. Config & Proto Validation Bridge
- **Goal:** Ensure Proto-first data types are validated against Rust config defaults.
- **Tasks:**
  - [ ] Implement a `Validate` trait for Proto-generated types.
  - [ ] Create a "Safe Config" layer that converts raw Proto configs into validated Rust domain objects with defaults applied.
  - [ ] Ensure all gRPC entry points validate incoming configs before they reach the core logic.

## Phase 3: Final Verification & Performance
- [ ] Implement `IT-010` (Cross-modal query E2E).
- [ ] Implement `IT-110` (OCR Transform Pipeline).
- [ ] Execute `IT-160` (Performance Suite) and establish baselines.

