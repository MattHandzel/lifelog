# Phase 4: Data Migration & Cleanup — SurrealDB Removal (Final)

You are executing the final phase of the SurrealDB removal plan.
Read the full plan at: `/home/matth/.claude/plans/optimized-squishing-toucan.md`

Phases 0-3 are complete: all code uses the `frames` table, SurrealDB deps are removed.

## Your Task

Migrate existing per-modality Postgres data into the unified `frames` table, then drop old tables.

### Steps

1. **Create migration** `server/migrations/YYYYMMDD_migrate_to_frames.sql` (use today's date):
   ```sql
   -- Migrate each old per-modality table into frames
   INSERT INTO frames (id, collector_id, stream_id, modality, time_range,
       t_device, t_ingest, t_canonical, t_end, time_quality,
       blob_hash, blob_size, indexed, payload)
   SELECT id, collector_id, stream_id, 'Screen', time_range,
       t_device, t_ingest, t_canonical, t_end, time_quality,
       blob_hash, blob_size::integer, true,
       jsonb_build_object('width', width, 'height', height, 'mime_type', mime_type)
   FROM screen_records ON CONFLICT (id) DO NOTHING;
   -- Repeat pattern for: browser_records, ocr_records, audio_records,
   -- clipboard_records, shell_history_records, keystroke_records, transcription_records
   ```

2. **Create migration** `server/migrations/YYYYMMDD_drop_legacy_tables.sql`:
   - Drop all old per-modality tables: `screen_records`, `browser_records`, `ocr_records`,
     `audio_records`, `clipboard_records`, `shell_history_records`, `keystroke_records`,
     `transcription_records`

3. **Update config**: Change example/default `databaseEndpoint` from SurrealDB WebSocket to Postgres URI format.

4. **Update docs**: Update CLAUDE.md to remove SurrealDB references (startup command, env vars, etc.).

5. **Update memory files**: Update relevant memory files in `/home/matth/.claude/projects/-home-matth-Projects-lifelog/memory/` — remove SurrealDB gotchas, update architecture notes.

6. **Verify**: `just check` + `just test` pass.

7. **Commit**: `feat: migrate legacy tables to unified frames, drop old tables`

## Final Verification

After committing, print a summary of what was accomplished across all 5 phases (0-4):
- Lines removed vs added
- Build time improvement (if measurable)
- Files deleted
- Dependencies removed

This is the final phase. No further chaining needed.
