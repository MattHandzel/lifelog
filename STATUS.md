# Status

## Current Objective

Expanding automated validation coverage and finishing remaining data modality migrations.

## Where To Resume

- Branch: `refactor/proto-first-completion`

## Last Verified

- `just check` (passes)
- `just test` (passes)
- `just test-e2e` (passes: IT-090 verified)

## How To Verify (Target)

- `just check`
- `just test-e2e`

## What Changed Last

- **IT-081 Verified**: ACK Gate implemented and verified. Backend now only ACKs chunks if they are marked as indexed.
- **ChunkIngester Fixed**: Corrected logic to check `is_chunk_indexed` on the start offset.
- **SurrealIngestBackend Improved**: Now uses `CREATE` instead of `UPSERT` to be idempotent without overwriting `indexed` status.
- **Integration Test Scaffolding**: `TestContext` now exposes `db_addr` for direct DB manipulation in tests.
- **Proto-First Refactor Completed**: All Config and State types now use `lifelog_proto` generated structs. Manual type conversion layers removed.
- **IT-090 Verified**: Resumable chunked upload protocol is now verified by a real integration test.

## What's Next

- Wire up real SurrealDB metadata persistence in `SurrealIngestBackend` (parse chunk data and insert actual records).
- Implement `IT-100` (Blob Separation) and `IT-110` (OCR Transform).
- Migrate Modality types (`ScreenFrame`, `BrowserFrame`) to re-exports.

## Blockers

- None.
