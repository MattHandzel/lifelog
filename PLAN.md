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

5. Unit: Chunk/Offset + CAS Primitives (UT-040, UT-050..UT-052)
   - Done when: byte-offset validation + SHA256 hashing + filesystem CAS put/get/dedupe are implemented and tested.
   - Status: CAS primitive implemented in `common/utils/src/cas.rs`; chunk/offset validation not started.

6. Integration Scaffolding (IT-010, IT-060, IT-080, IT-090, IT-081, IT-100, IT-110, IT-130, IT-140, IT-150, IT-160)
   - Done when: there is a consistent way to spin up ephemeral backend components for tests (DB + CAS paths + config), even if tests remain ignored.

## Risks / Mitigations

- Native deps on Linux (alsa, glib/gtk, tesseract, etc.) can make default `cargo test` brittle.
  - Mitigation: keep UI crate out of default workspace builds; ensure tests can run inside `nix develop`.
- Integration tests will need controllable, deterministic time and storage state.
  - Mitigation: build small “test fixtures” modules that seed DB/CAS deterministically.

## Verification Plan

- L1: `nix develop -c cargo check`
- L3: `nix develop -c cargo test`
- L4: AC replay: each implemented test links back to `VALIDATION_SUITE.md` section/test ID in comments.
