# Phase 1: Unified Ingest — SurrealDB Removal

You are executing Phase 1 of the SurrealDB removal plan.
Read the full plan at: `/home/matth/.claude/plans/optimized-squishing-toucan.md`

Phase 0 (Foundation) is already complete: the `frames` table migration and `server/src/frames.rs` conversion layer exist and are tested.

## Your Task

Replace the dual ingest backends with a single `UnifiedIngestBackend` that writes all modalities to the `frames` table via `frames.rs`.

### Steps

1. **Read `server/src/frames.rs`** to understand the `FrameRow`, `from_lifelog_data()`, and `insert_sql()`/`insert_params()` API.

2. **Create `UnifiedIngestBackend`** in `server/src/ingest.rs` (or a new file) that implements `IngestBackend`:
   - Decodes proto payload, calls `frames::from_lifelog_data()` to get a `FrameRow`
   - Applies time skew correction (use existing `get_canonical_time()` pattern)
   - Executes `FrameRow::insert_sql()` with `insert_params()` against the Postgres pool
   - Registers origin in `catalog` table via `INSERT ... ON CONFLICT DO NOTHING`
   - Stores upload chunk metadata in `upload_chunks` (same as today)
   - Handles `indexed` flag: set to `false` when OCR transform is enabled for screen frames

3. **Update `server/src/grpc_service.rs`**: Construct `UnifiedIngestBackend` instead of `HybridIngestBackend`.

4. **Update `server/src/server.rs`**: Make `postgres_pool` required (not `Option`). Keep SurrealDB alive temporarily for read paths.

5. **Run `just validate`** (or `just check` + `just test` if clippy has system issues). Fix any errors.

6. **Commit** with message: `feat: unified ingest backend writing all modalities to frames table`

### Key Constraints
- All 15 modalities must be supported (JSONB makes this trivial via `from_lifelog_data`)
- CAS blob storage for binary modalities (Screen, Audio, Camera, Clipboard) stays the same
- Don't break read paths — SurrealDB reads still work during this phase
- Don't touch proto files

## When Done

After committing, spawn the next phase:

```bash
./tools/ai/surrealdb-removal/run_phase.sh 2 --tmux
```

Then **monitor the new tmux window for 1-2 minutes** to make sure the next phase starts correctly and is making progress (reading files, creating tasks, etc.). If it errors out or stalls on launch, report what went wrong. Only exit once you've confirmed the next phase is working.
