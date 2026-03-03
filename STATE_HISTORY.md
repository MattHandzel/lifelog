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
