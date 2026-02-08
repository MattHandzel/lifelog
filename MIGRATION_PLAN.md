# Lifelog v1 Migration Plan

This plan outlines the steps to move the current `lifelog` codebase to the architecture and requirements specified in `SPEC.md`. It supersedes `refactoring-plan.md` and expands upon `PLAN.md`.

## Phase 0: Hygiene & Planning Consolidation

**Goal:** Establish a clean baseline and single source of truth for the migration.

- [ ] **Clean Proto Files**: The current `lifelog.proto` mixes old RPCs (`RegisterCollector`) with the new `ControlStream`.
  - Remove old RPCs
  - Ensure `ControlStream` is the _only_ path for collector registration and state reporting.
- [ ] **Verify Build Chain**: Ensure `just build` or `cargo build` correctly regenerates Rust code from `.proto` files and that `pbjson` is working for all types.

## Phase 1: Protocol & Schema Authority

**Goal:** Make `.proto` files the absolute source of truth for all data types and enforce strict typing.

- [ ] **Complete Modality Definitions**:
  - Verify `lifelog_types.proto` covers all v1 modalities:
    - [ ] Audio (Chunked, Interval)
    - [ ] Keystrokes (Point, Text content)
    - [ ] Clipboard (Text + Binary)
    - [ ] Shell History (Command, CWD, Exit Code)
    - [ ] Window/App Activity (Interval/State)
- [ ] **Standardize IDs**: Ensure `collector_id`, `stream_id`, `session_id`, and `chunk_hash` are consistently defined and used across all protos.
- [ ] **Config Validation**: Implement a "Safe Config" layer (as per `PLAN.md`) that validates Proto-generated config structs against logic constraints (e.g., "interval > 0").

## Phase 2: Reliability & Data Plane (Critical Path)

**Goal:** Implement "Store everything, lose nothing" guarantees.

### 2.1 Collector Buffering (WAL)

**Current:** In-memory `Vec<Frame>` (Data loss on crash).
**Target:** Disk-backed Write-Ahead Log.

- [ ] **WAL Implementation**: Create a `DiskBuffer` trait/struct in `common/`.
  - Must support: `append(item)`, `peek_chunk(size)`, `commit_offset(offset)`.
  - Implementation options: `squeaky`, `sled`, or a simple append-only file + read pointer.
- [ ] **Integrate into Collector**: Replace `Vec` buffer in `DataSource` with `DiskBuffer`.

### 2.2 Content Addressed Storage (CAS)

**Current:** Images stored inline in DB or filesystem (ad-hoc).
**Target:** Dedicated CAS for all blobs.

- [ ] **CAS Module**: Implement `BlobStore` in `server/`.
  - API: `put(bytes) -> hash`, `get(hash) -> bytes`, `has(hash) -> bool`.
  - Backend: Flat file system structure (e.g., `objects/ab/cd/ef...`).
- [ ] **Ingest Integration**: `UploadChunks` RPC must write payload to CAS, verify hash, and _then_ create metadata.

### 2.3 Durable Ingest & ACKs

**Current:** Basic ingest.
**Target:** ACK only when fully indexed.

- [ ] **SurrealDB Indexing**: Define explicit schema/indices for `timestamp`, `origin`, `text_content`.
- [ ] **Transaction Logic**:
  1. Write Blob to CAS (if applicable).
  2. Write Metadata to SurrealDB.
  3. Update Text/Time Indices.
  4. **Only then** return `Ack` to collector.
- [ ] **Resumable Upload**: Implement `GetUploadOffset` correctly (returning the last durably indexed byte offset).

## Phase 3: Query Engine & Transforms

**Goal:** Enable "Recall Anything" via deterministic queries.

- [ ] **Schema Definition**: Create `server/src/schema.rs` that defines the SurrealDB tables and fields explicitly (no lazy `ensure_table`).
- [ ] **Query Compiler**:
  - Implement AST for the Query Language (Section 10 in SPEC).
  - Implement `Planner` that converts AST -> SurrealQL (or internal iterator plan).
  - Support operators: `WITHIN`, `OVERLAPS`, `DURING`.
- [ ] **OCR Pipeline Refactor**:
  - Move from "scan all UUIDs" to "subscription/cursor" model.
  - OCR Job: Read un-OCR'd screen records -> Run Tesseract -> Write Text Record -> Update Cursor.

## Phase 4: Frontend & Connectivity

**Goal:** A unified, secure interface.

- [ ] **Unified API Surface**:
  - Decision: Use **gRPC-Web** or a **gRPC-JSON Transcoder** (like `tonic-web`) on the _same_ port/process as the main backend.
  - Remove reliance on any separate REST server.
- [ ] **TLS Everywhere**:
  - Configure `tonic` to require TLS for all connections (Collector & UI).
  - Add logic for loading certs from config.
- [ ] **UI Implementation**:
  - Update `interface/` to generate TypeScript clients from `.proto`.
  - Implement the Timeline View using `Query` RPCs.

## Phase 5: Operational Readiness

- [ ] **Health/Status Reporting**: Ensure `ReportState` accurately reflects buffer sizes, upload lag, and last-seen times.
- [ ] **Packaging**: Create `systemd` unit files and a `Justfile` install target.
