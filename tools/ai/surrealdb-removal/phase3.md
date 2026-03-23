# Phase 3: SurrealDB Code Removal — SurrealDB Removal

You are executing Phase 3 of the SurrealDB removal plan.
Read the full plan at: `/home/matth/.claude/plans/optimized-squishing-toucan.md`

Phases 0-2 are complete: all reads and writes use the `frames` table. SurrealDB is no longer used at runtime.

## Your Task

Delete all SurrealDB code, types, and dependencies. This is mechanical but touches many files.

### Steps

1. **Delete entire files**:
   - `server/src/schema.rs` (~510 lines of SurrealDB DDL)
   - `server/src/db.rs` (SurrealDB helpers)
   - `server/src/sync.rs` (stub taking Surreal param)

2. **Clean up ingest** (`server/src/ingest.rs`):
   - Remove `SurrealIngestBackend` entirely
   - Remove `HybridIngestBackend` enum if it still exists
   - Keep only `UnifiedIngestBackend`

3. **Delete ToRecord types** (`common/lifelog-types/src/lib.rs`):
   - Remove entire `#[cfg(feature = "surrealdb")]` block (~400 lines)
   - Remove `ToRecord` trait + all `*Record` structs + trait impls

4. **Remove Cargo dependencies**:
   - `Cargo.toml` workspace: remove `surrealdb` from `[workspace.dependencies]`
   - `server/Cargo.toml`: remove `surrealdb`, remove `features = ["surrealdb"]` from lifelog-types dep
   - `common/lifelog-types/Cargo.toml`: remove optional `surrealdb` dep and feature
   - `collector/Cargo.toml`: remove unused `surrealdb`, `mobc`, `mobc-surrealdb` if present

5. **Update `server/src/lib.rs`**: Remove `mod db`, `mod schema`, `mod sync` declarations. Remove `test_support::reset_table_cache()` if it references deleted code.

6. **Update test harness** (`server/tests/harness/mod.rs`):
   - Remove SurrealDB process spawn (`Command::new("surreal")`)
   - Remove sleep wait for DB startup
   - Remove `LIFELOG_DB_USER`, `LIFELOG_DB_PASS` env vars
   - Update `TestContext` to remove surreal fields

7. **Update all test files**: Remove SurrealDB references, use Postgres only.

8. **Verify**:
   - `just check` + `just test` pass
   - `nix develop --command cargo tree | rg -i surrealdb` returns nothing
   - Build should be noticeably faster

9. **Commit**: `refactor: remove all SurrealDB code and dependencies`

## When Done

After committing, spawn the next phase:

```bash
./tools/ai/surrealdb-removal/run_phase.sh 4 --tmux
```

Then **monitor the new tmux window for 1-2 minutes** to make sure the next phase starts correctly and is making progress (reading files, creating tasks, etc.). If it errors out or stalls on launch, report what went wrong. Only exit once you've confirmed the next phase is working.
