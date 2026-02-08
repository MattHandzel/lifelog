# Lifelog Development Plan

## Phase 1: Fundamentals (COMPLETED)
- [x] Time Skew Estimator (UT-010)
- [x] Replay Step Builder (UT-011)
- [x] Correlation Operators (UT-020)
- [x] Chunk/Offset + CAS Primitives (UT-040)
- [x] Integration Scaffolding & Test Harness (IT-090)
- [x] Proto-First Type System Migration

## Phase 2: Critical Functional Repairs (COMPLETED)

### 1. Collector Upload Implementation (DONE)
- [x] **Upload Manager Actor:** Background actor in `collector` pumping data to server.
- [x] **Cursor Management:** Local WAL offset tracking in `DiskBuffer`.
- [x] **Data Pump:** Binary streaming via `UploadChunks` RPC.
- [x] **Durable ACK Gate:** Local cursor only advances on server confirmation.

### 2. Server Control Plane & Device Persistence (DONE)
- [x] **Improved Sync Protocol:** Functional `UploadChunks` and `GetUploadOffset` implementation.
- [x] **Robust Metadata Ingestion:** Resolved serialization issues between Protobuf and SurrealDB.

## Phase 3: Architecture & Performance (COMPLETED)

### 1. Transformation Pipeline Optimization (DONE)
- [x] **Watermark Polling:** Replaced $O(N)$ full-table diffing in OCR pipeline with an efficient cursor-based approach.
- [x] **Persistence:** Added `watermarks` table to track progress per transform.

### 2. Proto Crate Decoupling (DONE)
- [x] **Feature-Based Split:** Refactored `lifelog-types` to use cargo features, allowing lightweight clients to pull generated code without heavy dependencies.

### 3. Explicit Database Schema & Migrations (DONE)
- [x] **Schema Registry:** Centralized SurrealDB DDL in `server/src/schema.rs`.
- [x] **Strict Typing:** Enabled `SCHEMAFULL` tables for improved data integrity.

## Phase 4: Final Validation & Cleanup (COMPLETED)

- [x] **Error Consolidation:** Unified `LifelogError` hierarchy across all crates.

- [x] **IT-010:** Cross-modal query E2E test.

- [x] **IT-110:** Full OCR Transform Pipeline integration test.

- [x] **IT-160:** Performance Suite & established baselines.



## Phase 5: Polish & Architectural Cleanup (COMPLETED)

- [x] **Unified Trait Model:** Finalized `DataType`, `Modality`, and `ToRecord` traits.

- [x] **Catalog Refactor:** Moved away from `INFO FOR DB` to explicit `catalog` table.

- [x] **Robust Key Retrieval:** Fixed serialization issues with SurrealDB native types in query results.

- [x] **Code Coverage:** Increased test coverage for core modules.
