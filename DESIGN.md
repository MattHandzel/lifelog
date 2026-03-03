# Design Notes

## Network Topology Dashboard (2026-03-01)

### Scope

- Replaced the legacy Devices page with an interactive Network dashboard.
- Added a visual topology model showing one server node and all connected collector nodes.
- Added collector health/detail controls directly from node selection.

### Architecture Decisions

- Implemented topology rendering with native React + SVG instead of adding a new graph library.
  - Rationale: no additional dependency churn and enough flexibility for glowing edges + pulse animation.
- Used existing Tauri commands as the backend integration surface:
  - `get_system_state`, `get_collector_ids`,
  - `get_component_config`, `set_component_config`.
- Kept alias/icon customization local to the interface (localStorage override) because current backend config RPCs do not expose alias/icon fields.
- Implemented force-sync action as an explicit attempted RPC call (`force_collector_sync`) with surfaced warning when unavailable.

### Data Flow

1. UI loads collector state via `get_system_state` (fallback `get_collector_ids`).
2. UI loads per-collector modality configs for managed modalities.
3. Topology graph renders:
   - edge glow based on online/offline status,
   - pulse animation based on active source state inference.
4. Selecting a node opens controls for:
   - per-modality enable/disable,
   - pause/resume all known modalities,
   - local alias/icon override save.

### Validation

- Added `NetworkTopologyDashboard` unit test for node render + modality update command dispatch.
- Verified via `just test-ui` and `just validate-all`.

## Search Previews (2026-03-01)

### Scope

- Enhance search results with:
  - text snippets around query terms,
  - highlighted term matches,
  - lightweight thumbnails for image modalities.

### Architecture Decisions

- Keep `query_timeline` as the primary key retrieval path.
- Add an interface backend enrichment command:
  - `get_frame_data_thumbnails(keys)` returns frame metadata plus downscaled image previews.
- Perform snippet construction in the frontend from enriched frame fields.
  - Rationale: avoids proto churn and allows UI-level tuning of snippet length and highlight behavior.

### Data Flow

1. UI calls `query_timeline` with text query.
2. UI calls `get_frame_data_thumbnails` for returned keys.
3. UI builds `SearchResult` models with:
   - `snippet`,
   - `highlightTerms`,
   - `preview` (thumbnail data URL for image frames).
4. `ResultCard` renders lazy thumbnail + highlighted snippet.

### Validation

- Added `SearchDashboard` UI tests for:
  - snippet highlighting,
  - thumbnail rendering.
- Verified with `just test-ui` and `just validate`.

## Security Hardening (2026-03-01)

### Scope

- Enforced mandatory TLS for server and collector gRPC traffic.
- Enforced mandatory bearer authentication token checks on server RPC entrypoints.
- Wired collector pairing usage (`PairCollector`) into collector handshake when only enrollment token is present.
- Unified collector identity usage across control/state traffic to configured collector id.

### Architecture Decisions

- Server startup now fails if TLS cert/key are not configured via:
  - `LIFELOG_TLS_CERT_PATH`,
  - `LIFELOG_TLS_KEY_PATH`.
- Server startup now fails if token env vars are missing:
  - `LIFELOG_AUTH_TOKEN`,
  - `LIFELOG_ENROLLMENT_TOKEN`.
- Collector and upload manager now reject non-`https://` server addresses.
- Pairing keeps configured collector identity as source of truth for stream/upload consistency.

### Validation

- Verified compilation with `tools/ai/run_and_digest.sh "just check"` after changes.
- `just test` was started under digest wrapper and did not complete in this environment within the session window.

## Retention Controls (2026-03-01)

### Scope

- Added coarse-grained retention controls for automatic data lifecycle management.
- Added a server-side pruning worker and UI controls in Settings.

### Architecture Decisions

- `ServerConfig` now carries `retention_policy_days: map<string,uint32>`.
  - Keys are modality buckets (for now: `screen`, `audio`, `text`; optional `all` fallback in pruning logic).
  - `0` means no automatic deletion.
- Retention worker runs as a dedicated server background task.
  - Default schedule is daily.
  - `LIFELOG_RETENTION_INTERVAL_SECS` can override interval for testing/ops.
- Prune semantics:
  - Use `t_canonical` when present, otherwise `timestamp`.
  - Delete stale rows from origin tables by modality policy.
  - Gather `blob_hash` values from deleted rows and remove only orphaned CAS blobs after cross-table reference checks.
- Live config updates:
  - `SetSystemConfigRequest` now carries full `SystemConfig`.
  - Server `SetConfig` applies server retention/default-window updates and forwards collector updates over `ControlStream` via `UpdateConfig`.

### Validation

- `tools/ai/run_and_digest.sh "just check"`: pass.
- `tools/ai/run_and_digest.sh "just test"`: pass.
- `tools/ai/run_and_digest.sh "cd interface && npm run build"`: pass.

## PostgreSQL Migration Phase 1 Scaffold (2026-03-03)

### Scope

- Added initial PostgreSQL migration assets and runtime helper module in `server/`.
- Kept SurrealDB runtime paths intact to avoid partial cutover regressions while landing scaffolding.

### Architecture Decisions

- Added a new server-local PostgreSQL helper module (`server/src/postgres.rs`) with:
  - URI detection helper (`postgres://` / `postgresql://`).
  - Connection pooling via `deadpool-postgres`.
  - Ordered SQL migration execution from `server/migrations/`.
- Added first static migration file (`20260303143000_init_postgres.sql`) containing:
  - `upload_chunks` idempotency/resume metadata table.
  - `catalog` and `transform_watermarks` metadata tables.
  - Initial unified modality tables (`screen_records`, `browser_records`, `ocr_records`, `audio_records`, `clipboard_records`, `shell_history_records`, `keystroke_records`).
  - `TSTZRANGE` + GIST temporal indexes and TSVECTOR + GIN text indexes for search-capable modalities.
- Added `just` recipes for SQL migration workflow:
  - `just sqlx-migrate-add <name>`
  - `just sqlx-migrate-run <database_url>`

### Validation

- `just check-digest`: pass.
- `tools/ai/run_and_digest.sh "nix develop --command cargo test -p lifelog-server postgres::tests --lib"`: pass.

## PostgreSQL Migration Phase 2 (Ingestion Rewrite) (2026-03-03)

### Scope

- Rewrote the server ingest path to support PostgreSQL as an ingest backend for chunk streaming.
- Preserved existing Surreal ingest/query runtime as fallback to avoid partial cutover regressions.

### Architecture Decisions

- Added `PostgresIngestBackend` implementing `utils::ingest::IngestBackend`.
- Added `HybridIngestBackend` delegator (`Surreal` or `Postgres`) so `UploadChunks` can route at runtime without changing the collector protocol.
- PostgreSQL ingest writes decoded frame payloads directly via parameterized SQL (no `ToRecord` in Postgres path).
- Idempotency strategy for chunk metadata:
  - `INSERT ... ON CONFLICT (id) DO UPDATE SET indexed = (upload_chunks.indexed OR EXCLUDED.indexed)`.
- ACK gate semantics preserved:
  - `screen` chunks remain pinned (`indexed=false`) when OCR transform is enabled.
  - other mapped streams set `indexed=true` once persisted.
- CAS remains filesystem-backed; Postgres records store `blob_hash` and `blob_size` references.
- Postgres ingest activation is explicit through `LIFELOG_POSTGRES_INGEST_URL` (+ optional `LIFELOG_POSTGRES_INGEST_MAX_CONNECTIONS`).

### Validation

- `just check-digest`: pass.
- `tools/ai/run_and_digest.sh "just test"`: pass.

## PostgreSQL Migration Phase 4 (Operations & Deployment) (2026-03-03)

### Scope

- Switched deployment defaults to PostgreSQL-first operation while preserving SurrealDB during hybrid transition.
- Added PostgreSQL pool observability to server state responses.
- Updated operational docs/config examples to reflect PostgreSQL as the primary dependency.

### Architecture Decisions

- Added a NixOS module in `flake.nix` (`nixosModules.lifelog-postgres`) that:
  - enables `services.postgresql`,
  - auto-creates `lifelog` database and `lifelog` role (configurable).
- Updated server systemd units to:
  - depend on `postgresql.service`,
  - export `LIFELOG_POSTGRES_INGEST_URL` and `LIFELOG_POSTGRES_INGEST_MAX_CONNECTIONS`.
- Kept `lifelog-surrealdb.service` dependency/env in place for transition safety.
- Extended `ServerState` with PostgreSQL pool metrics sourced from `deadpool-postgres` status:
  - enabled flag,
  - max size,
  - current pool size,
  - available connections,
  - waiting requests.

### Validation

- `tools/ai/run_and_digest.sh "just check-digest"`: pass.
