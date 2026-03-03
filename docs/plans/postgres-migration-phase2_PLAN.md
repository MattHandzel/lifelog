# PostgreSQL Migration Phase 2: Ingestion Layer Rewrite

**Objective:** Ensure the collector can stream gigabytes of data reliably and idempotently into PostgreSQL.

- [ ] **Task 2.1: Refactor IngestBackend**
  - Create `PostgresIngestBackend` implementing the `IngestBackend` trait.
- [ ] **Task 2.2: Data Mapping & ToRecord Removal**
  - Remove the custom `ToRecord` trait implementations and JSON serialization overhead if no longer needed.
  - Map Protobuf payloads directly into Postgres parameterized queries (using `tokio-postgres` as established in Phase 1). This guarantees strict type mapping.
- [ ] **Task 2.3: Idempotency Logic**
  - Replace SurrealDB's `UPSERT` with PostgreSQL's `ON CONFLICT (id) DO UPDATE SET indexed = EXCLUDED.indexed`.
  - Ensure the "ACK Gate" (Spec §6.2.1) logic is preserved: `indexed` only becomes `true` when derived transforms (like OCR) are completed, or immediately if disabled.
- [ ] **Task 2.4: Blob/CAS Integrity**
  - The CAS (Content-Addressed Store) logic remains on the filesystem. Ensure the `blob_hash` is correctly written as a `VARCHAR` in Postgres and linked to the metadata.