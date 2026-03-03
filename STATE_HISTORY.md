<state_snapshot>
      <overall_goal>
      Implement Phase 1 PostgreSQL migration foundation in the server worktree without breaking the existing SurrealDB runtime paths.
      </overall_goal>

      <what_to_do>
          - Added PostgreSQL helper module and migration SQL scaffolding.
          - Added migration-oriented just recipes.
          - Added Nix devshell tools needed for migration workflow (`sqlx-cli`, `postgresql`).
          - Kept existing SurrealDB execution flow unchanged to avoid partial cutover regressions.
      </what_to_do>
      <why>
          - Phase 1 requires static schema/migration groundwork before replacing ingest/query runtime.
          - Current codebase is deeply Surreal-coupled; safe incremental migration avoids destabilizing unrelated features.
          - Hypothesis: landing schema + pool + migration runner first reduces risk and enables iterative backend porting.
      </why>

      <how>
          - Added `server/src/postgres.rs` with postgres URI detection, pool construction, and ordered migration execution.
          - Added first SQL migration `server/migrations/20260303143000_init_postgres.sql` with unified modality tables, upload chunk metadata, catalog/watermarks, and indexes.
          - Added `just sqlx-migrate-add` and `just sqlx-migrate-run` recipes.
          - Added devshell packages in `flake.nix` for migration tooling.
          - Added module export in `server/src/lib.rs`.
      </how>

      <validation_steps>
           - `just check-digest` (pass).
           - `tools/ai/run_and_digest.sh "nix develop --command cargo test -p lifelog-server postgres::tests --lib"` (pass).
      </validation_steps>

</state_snapshot>

<state_snapshot>
      <overall_goal>
      Implement Phase 2 PostgreSQL migration for ingestion so collector chunk uploads are persisted idempotently in PostgreSQL with ACK gating preserved.
      </overall_goal>

      <what_to_do>
          - Added a PostgreSQL-backed ingest implementation for `UploadChunks`.
          - Removed Surreal-specific `ToRecord` mapping from the new Postgres ingest path by decoding protobuf frames directly and executing typed SQL inserts.
          - Preserved durable ACK behavior via `upload_chunks.indexed` and idempotent upserts.
          - Kept Surreal ingest/query runtime intact as default path for non-Postgres sessions.
      </what_to_do>
      <why>
          - Phase 2 requires moving ingest semantics (throughput/idempotency/ACK correctness) onto PostgreSQL before full query/transform cutover.
          - Hypothesis: a dual backend switch at gRPC ingest boundary enables incremental migration with low regression risk.
          - Assumption tested: server can safely prefer Postgres ingest only when explicitly configured, and retain existing Surreal path otherwise.
      </why>

      <how>
          - Added `PostgresIngestBackend` in `server/src/ingest.rs` implementing `IngestBackend`.
          - Added `HybridIngestBackend` delegator (`Surreal` | `Postgres`) for runtime backend selection in gRPC upload handling.
          - Implemented parameterized SQL inserts for `screen_records`, `browser_records`, `ocr_records`, `audio_records`, `clipboard_records`, `shell_history_records`, `keystroke_records`.
          - Implemented idempotent chunk metadata persistence in PostgreSQL with:
            `INSERT ... ON CONFLICT (id) DO UPDATE SET indexed = (upload_chunks.indexed OR EXCLUDED.indexed)`.
          - Preserved CAS linkage by storing `blob_hash`/`blob_size` in Postgres modality tables.
          - Added optional Postgres ingest pool bootstrap in `Server::new` from `LIFELOG_POSTGRES_INGEST_URL` plus migration run.
          - Updated `GetUploadOffset` RPC to query PostgreSQL when ingest pool is enabled.
      </how>

      <validation_steps>
           - `just check-digest` (pass).
           - `tools/ai/run_and_digest.sh "just test"` (pass).
      </validation_steps>

</state_snapshot>

<state_snapshot>
      <overall_goal>
      Implement Phase 3 PostgreSQL migration for query execution by translating AST/plans to native PostgreSQL operations, adding temporal overlap SQL execution, and routing query/replay between SurrealDB and PostgreSQL.
      </overall_goal>

      <what_to_do>
          - Added PostgreSQL execution for `ExecutionPlan::TableQuery` with native filtering and FTS translation.
          - Added PostgreSQL-native temporal execution for `ExecutionPlan::DuringQuery` using range overlap checks in SQL (`time_range && ...`) with source-term `EXISTS` joins.
          - Added hybrid query routing in server query path based on plan compatibility and configured Postgres pool.
          - Added hybrid origin discovery (Surreal catalog + Postgres distinct collector/origin scan).
          - Updated replay to use PostgreSQL range queries for migrated modalities (screen/context) with Surreal fallback for non-migrated modalities.
          - Refactored planner plans to carry backend-agnostic filters (`Expression`) alongside existing Surreal SQL strings.
      </what_to_do>
      <why>
          - Phase 3 requires moving temporal overlap work from Rust interval materialization into PostgreSQL execution for performance and reduced memory overhead.
          - Hypothesis: keeping the existing planner shape while adding AST-bearing plan fields enables dual-engine execution with minimal regression risk.
          - Assumption tested: a plan-level compatibility check allows safe per-query/per-plan backend routing for incremental migration.
      </why>

      <how>
          - Reworked `ExecutionPlan` variants in `server/src/query/planner.rs` to include filter expressions and explicit limits used by Postgres execution.
          - Implemented `execute_postgres(...)` and `plan_is_postgres_compatible(...)` in `server/src/query/executor.rs`.
          - Implemented Postgres expression compilation:
            - `Expression::TimeRange` -> `time_range && tstzrange(...)`
            - `Expression::Contains` -> `search_document @@ websearch_to_tsquery('english', ...)` on supported tables.
          - Implemented temporal translation with SQL `EXISTS` source subqueries and overlap checks against expanded source ranges.
          - Updated `process_query` in `server/src/server.rs` to route plans to Postgres when compatible; otherwise use Surreal executor.
          - Added Postgres-origin introspection and merged origin availability for selector resolution.
          - Updated replay screen/context fetches to query Postgres tables ordered by `lower(time_range)` where supported.
      </how>

      <validation_steps>
           - `just check-digest` (pass).
           - `tools/ai/run_and_digest.sh "nix develop --command cargo test -p lifelog-server plans_temporal_or_via_dnf_union --lib"` (pass).
           - `tools/ai/run_and_digest.sh "just test"` (pass).
      </validation_steps>

</state_snapshot>
