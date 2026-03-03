# PostgreSQL Migration Master Plan

This document outlines a rigorous, highly-detailed plan to migrate Lifelog’s hot metadata store from SurrealDB to PostgreSQL. It is designed to ensure zero data loss, guarantee feature parity, and significantly improve query performance while laying the foundation for advanced AI/vector search.

---

## 1. Executive Summary & Thoroughness Assessment

**Thoroughness:** This plan accounts for the full vertical slice of the application. It addresses dependency management, database schema design, asynchronous ingestion concurrency, query AST translation, deployment automation (Systemd/Nix), and the critical path of verification. It transitions the architecture from a dynamic "table-per-device-per-modality" graph model to a strongly typed, relational "unified-modality-table" model.

**Overall Confidence:** **90%**
The path is well-understood, as PostgreSQL is deeply predictable. The 10% uncertainty lies in bridging the gap in the Query Planner (`server/src/query/planner.rs`)—specifically, ensuring that our existing LLQL (Lifelog Query Language) translates flawlessly to PostgreSQL's SQL dialect, particularly for complex temporal joins (`DURING` / `WITHIN`).

---

## 2. Phase 1: Tooling & Schema Foundation
*Confidence Level: 95% - Standard Rust/SQLx integration.*

**Objective:** Set up the connection pooling, remove SurrealDB dependencies, and define the optimal PostgreSQL schema.

- [x] **Task 1.1: Dependency Swap**
  - Remove `surrealdb` and related mobc crates from `Cargo.toml`.
  - Add `sqlx` (with features `["postgres", "runtime-tokio-rustls", "chrono", "uuid", "json"]`).
  - Add `testcontainers` or a similar crate for ephemeral testing in `server/tests/harness`.
- [x] **Task 1.2: Migration Management**
  - Integrate `sqlx-cli` into the `justfile`.
  - Replace the dynamic, on-the-fly DDL in `server/src/schema.rs` with static SQL migration files (`migrations/YYYYMMDD_init.sql`).
- [x] **Task 1.3: Schema Redesign (The Core Architecture)**
  - Transition from `[collector_id]:[modality]` dynamic tables to unified tables (e.g., `screen_records`).
  - Create the `upload_chunks` tracking table with a composite primary key or a deterministic string ID for idempotency.
  - **Temporal Innovation:** Use PostgreSQL's `TSTZRANGE` for the canonical time window. Point records will use a zero-duration range.
  - **Search Innovation:** Implement `TSVECTOR` columns with `GIN` indexes for the `browser`, `ocr`, `shell_history`, and `keystrokes` tables to replace SurrealDB's BM25 analyzer.
- [x] **Task 1.4: Server State Initialization**
  - Update `lifelog-config.toml` to accept a PostgreSQL connection URI instead of a WebSocket URL.
  - Initialize the `sqlx::PgPool` during server startup and run migrations automatically (`sqlx::migrate!().run(&pool)`).

---

## 3. Phase 2: Ingestion Layer Rewrite
*Confidence Level: 90% - High-throughput ingestion in Postgres is a solved problem.*

**Objective:** Ensure the collector can stream gigabytes of data reliably and idempotently.

- [x] **Task 2.1: Refactor `SurrealIngestBackend`**
  - Create `PostgresIngestBackend` implementing the `IngestBackend` trait.
- [x] **Task 2.2: Data Mapping & `ToRecord` Removal**
  - Remove the custom `ToRecord` trait implementations and JSON serialization overhead.
  - Map Protobuf payloads directly into `sqlx::query!` macros. This guarantees compile-time validation of our inserts.
- [x] **Task 2.3: Idempotency Logic**
  - Replace SurrealDB's `UPSERT` with PostgreSQL's `ON CONFLICT (id) DO UPDATE SET indexed = EXCLUDED.indexed`.
  - Ensure the "ACK Gate" (Spec §6.2.1) logic is preserved: `indexed` only becomes `true` when derived transforms (like OCR) are completed, or immediately if disabled.
- [x] **Task 2.4: Blob/CAS Integrity**
  - The CAS (Content-Addressed Store) logic remains on the filesystem. Ensure the `blob_hash` is correctly written as a `VARCHAR` in Postgres and linked to the metadata.

---

## 4. Phase 3: Query Execution & AST Translation
*Confidence Level: 85% - This is the most complex algorithmic work.*

**Objective:** Translate the complex temporal overlap logic currently handled in Rust into native PostgreSQL engine operations.

- [x] **Task 3.1: Table Queries & Full-Text Search**
  - Update `ExecutionPlan::TableQuery` to generate PostgreSQL queries.
  - Translate text searches from `SEARCH ANALYZER lifelog_text BM25` to `to_tsvector('english', col) @@ to_tsquery('english', 'query')`.
- [x] **Task 3.2: Temporal `DuringQuery` Translation (Massive Performance Win)**
  - **Current State:** `executor.rs` pulls thousands of intervals into Rust, merges them, and generates a massive `OR` string.
  - **New State:** Push this to the database using PostgreSQL's range overlap operator (`&&`).
  - Example: A query finding Audio during specific Browser activity becomes an `INNER JOIN` on `audio.time_range && browser.time_range`.
- [x] **Task 3.3: Replay/Timeline Alignment**
  - Update the `Replay` RPC logic to query Postgres. Use `ORDER BY lower(time_range) ASC` to ensure strict chronological ordering.

---

## 5. Phase 4: Operations, Deployment, & Migration
*Confidence Level: 95% - Standard infrastructure orchestration.*

**Objective:** Ensure a smooth transition for existing development environments and production deployments.

- [x] **Task 4.1: NixOS & Systemd Updates**
  - Remove `deploy/systemd/lifelog-surrealdb.service`.
  - Update `flake.nix` to provision PostgreSQL (`services.postgresql.enable = true`) and automatically create the `lifelog` database/user.
- [x] **Task 4.2: Legacy Data Migration (Optional/Strategic)**
  - Decide if a data migration tool is needed for existing SurrealDB data, or if v1 will launch fresh. If needed, write a one-off Rust script to iterate SurrealDB records and `INSERT` into PostgreSQL.
- [x] **Task 4.3: Health & Metrics**
  - Ensure `ReportState` and observability endpoints correctly reflect PostgreSQL pool metrics (active connections, idle connections).

---

## 6. Verification Strategy: Proving Parity and Superiority

To guarantee that this refactor works exactly as before (or better), we will employ the following verification strategy:

### 6.1. The Test Harness Pivot (Critical Path)
Currently, `server/tests/harness/mod.rs` spins up an ephemeral SurrealDB instance.
**Action:** We will integrate `testcontainers-rs` (or a local `initdb` script) to automatically provision an isolated PostgreSQL instance for `just test-e2e`. The exact same test suite (`ocr_pipeline.rs`, `cross_modal_query.rs`, `sync_scenarios.rs`) must pass without modifying the *assertions*, proving behavioral parity.

### 6.2. Performance Benchmarking
We have `server/tests/performance_suite.rs`.
**Action:** 
1. Run the suite on the current `main` branch (SurrealDB) and capture the baseline metrics (Ingestion Throughput, Query Latency).
2. Run the suite on the Postgres branch.
3. **Success Criteria:** 
   - Temporal query latency (`DURING`) drops by >50% due to native Range indexes.
   - Ingestion throughput increases due to reduced serialization overhead and no WebSocket layer.

### 6.3. Idempotency & Data Loss Validation
**Action:** Run a chaotic ingest test where the collector network connection is artificially dropped mid-upload. Verify that:
1. PostgreSQL `ON CONFLICT` prevents duplicate rows.
2. The `upload_chunks` offset tracking matches exactly.
3. The final record count equals the expected total.

### 6.4. Query AST Snapshot Testing
**Action:** Before removing SurrealDB, capture 20 complex LLQL queries and their resulting UUID sets. Run those identical queries against the new Postgres schema. The resulting UUID set must be identical (order-agnostic unless `Replay` is used).