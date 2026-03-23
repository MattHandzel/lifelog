# Phase 2: Unified Query & Retrieval — SurrealDB Removal

You are executing Phase 2 of the SurrealDB removal plan.
Read the full plan at: `/home/matth/.claude/plans/optimized-squishing-toucan.md`

Phases 0-1 are complete: `frames` table + conversion layer + unified ingest backend all exist.

## Your Task

Move all read paths to the `frames` table. After this phase, SurrealDB is no longer needed at runtime.

### Steps

1. **Add retrieval functions to `server/src/frames.rs`**:
   - `get_by_id(pool, cas, id) -> LifelogData` — SELECT from frames, reconstruct via `to_lifelog_data()`
   - `get_keys_after(pool, origin, after, limit) -> Vec<key>` — watermark-based cursor query
   - `get_origins(pool) -> Vec<origin>` — `SELECT DISTINCT collector_id, modality FROM frames`

2. **Rewrite query executor** (`server/src/query/executor.rs`):
   - Remove `execute()` (SurrealDB path)
   - Rewrite `execute_postgres()` to target `frames` table
   - Filter by `modality` column instead of routing to per-modality tables
   - Full-text search via `search_doc @@ websearch_to_tsquery('english', $N)`
   - Remove `plan_is_postgres_compatible()` and `pg_table_for_origin()`

3. **Update transform pipeline**:
   - `writer.rs`: Replace SurrealDB upsert with `INSERT INTO frames ... ON CONFLICT DO UPDATE`
   - `worker.rs`: Remove `db: Surreal<Client>` field, use Postgres pool only
   - `watermark.rs`: Remove `SurrealWatermarkStore`, keep `PostgresWatermarkStore`

4. **Simplify data retrieval** (`server/src/data_retrieval.rs`):
   - Remove `get_data_by_key_surreal()` and hybrid fallback
   - Delegate to `frames::get_by_id()`

5. **Update retention** (`server/src/retention.rs`):
   - `DELETE FROM frames WHERE modality = $1 AND t_canonical < $2`

6. **Update `server/src/server.rs`**:
   - Remove `db: Surreal<Client>` field and SurrealDB connection logic
   - Remove `LIFELOG_DB_USER`, `LIFELOG_DB_PASS` env vars
   - `get_available_origins()` uses Postgres only

7. **Run `just check` + `just test`**. The server should start WITHOUT SurrealDB running.

8. **Commit**: `feat: all reads from unified frames table, SurrealDB disconnected`

### Key Constraints
- Query/Replay/GetData must all return data from `frames` table
- Transform pipeline reads from and writes to `frames`
- Don't touch proto files

## When Done

After committing, spawn the next phase:

```bash
./tools/ai/surrealdb-removal/run_phase.sh 3 --tmux
```

Then **monitor the new tmux window for 1-2 minutes** to make sure the next phase starts correctly and is making progress (reading files, creating tasks, etc.). If it errors out or stalls on launch, report what went wrong. Only exit once you've confirmed the next phase is working.
