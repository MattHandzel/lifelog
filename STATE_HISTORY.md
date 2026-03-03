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

