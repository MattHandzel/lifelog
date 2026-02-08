# Codebase Audit Report: Lifelog (Final Deep Dive)
**Auditor:** Rena Kazakov
**Date:** 2026-02-07
**Scope:** Full Architecture, Concurrency, Data Integrity, and Wiring.

## 🔴 CRITICAL DEFECTS (Data Integrity & Architecture)

### 1. Data Ingestion Black Hole
*   **Location:** `server/src/ingest.rs`
*   **Defect:** The `persist_metadata` function contains a hardcoded check: `if stream_id.eq_ignore_ascii_case("screen")`.
*   **Impact:** ANY data stream that is not "screen" (e.g., "browser", "audio", "processes") is **received but never indexed**. The raw bytes are stored in CAS (Content-Addressable Store), but no database record is created to map the UUID to the data. This makes the data unqueryable and effectively lost.
*   **Root Cause:** Incomplete implementation of the ingestion dispatcher.

### 2. Browser History Collector is a No-Op
*   **Location:** `collector/src/modules/browser_history.rs`
*   **Defect:** The `run()` loop is an empty `sleep` loop. `get_data()` is defined but **never called**.
*   **Impact:** The "browser" module does absolutely nothing when enabled.
*   **Root Cause:** Forgotten implementation stub.

### 3. Processes Collector Violates Architecture (Direct DB Access)
*   **Location:** `collector/src/modules/processes.rs`
*   **Defect:** This module accepts a `&Surreal<C>` handle and writes directly to the database using `db.upsert("screen")`.
    1.  **Wrong Table:** It writes process data to the "screen" table!
    2.  **Security/Networking:** Collectors are designed to be standalone binaries communicating via gRPC. Requiring a direct DB connection breaks this model (won't work if server is remote) and exposes database credentials to the collector.
*   **Impact:** Data corruption (polluting "screen" table) and deployment failure in non-local setups.

### 4. "Detached" Modules (Weather, Hyprland, Camera)
*   **Defect:** These modules write to local SQLite files (`weather.db`, `hyprland.db`) or JPEG files (`camera_output/`). They have **zero** integration with the `DiskBuffer` or `UploadManager`.
*   **Impact:** Data collected by these modules stays on the client disk forever. It is never uploaded to the server.

### 5. `FsCas` Concurrency Risk
*   **Location:** `common/utils/src/cas.rs`
*   **Defect:** `put` performs a check `if final_path.exists()` then writes to a temp file and renames. While `rename` is atomic on POSIX, the check-then-write pattern is racy. If two threads try to write the same blob (same hash) simultaneously:
    1.  Thread A checks exists -> false.
    2.  Thread B checks exists -> false.
    3.  Thread A writes tmp -> renames.
    4.  Thread B writes tmp -> renames (overwriting A).
    *   *Mitigation:* Since CAS is content-addressed, overwriting identical content is benign *unless* file permissions/metadata matter or on non-POSIX filesystems where rename isn't atomic/replacing. However, it wastes I/O.
    *   *Note:* The tests specifically check for concurrent puts and pass, suggesting the benign overwrite behavior is acceptable for now, but it's wasteful.

## 🟡 QUERY & STORAGE ISSUES

### 1. Query Planner Naivety
*   **Location:** `server/src/query/planner.rs`
*   **Defect:** The planner blindly maps `StreamSelector::StreamId(id)` to a table name.
    *   If a collector sends stream_id="browser", the planner queries table `browser`.
    *   However, `ingest.rs` creates tables based on `DataOrigin`. If `DataOrigin` construction logic diverges (e.g. including device ID in table name like `device_id:browser`), the planner will query the wrong table.
*   **Impact:** Queries might return empty results if table naming conventions aren't strictly aligned between Ingest and Query.

### 2. DiskBuffer Corruption Risk
*   **Location:** `common/utils/src/buffer.rs`
*   **Defect:** `append` uses `OpenOptions::new().append(true)`. If a write is partial (e.g. power failure), the `wal.log` might contain a torn frame (incomplete length or data).
*   **Impact:** `peek_chunk` reads `u32` length. If it reads garbage length from a torn write, it will try to allocate a massive buffer (OOM risk) or desync from the stream.
*   **Fix:** Use a checksum/CRC per record or a marker byte.

## 🟠 CONFIGURATION & SECURITY

### 1. Hardcoded Endpoints
*   **Multiple Locations:** `http://127.0.0.1:7182` is hardcoded in `collector/src/main.rs` (CLI default) and `common/config/src/lib.rs`.
*   **Impact:** Confusing configuration hierarchy (CLI arg vs Config file vs Hardcoded default).

## ✅ RECOMMENDATIONS (Execution Plan)

1.  **Refactor `ingest.rs`**: Add a `match` block to handle deserialization for `BrowserFrame`, `AudioFrame` (if exists), etc., ensuring they get their own tables.
2.  **Fix `processes.rs`**:
    *   Remove `Surreal` dependency.
    *   Implement `DataSource` properly with `DiskBuffer`.
    *   Define `ProcessFrame` proto if missing.
3.  **Fix `browser_history.rs`**: Implement the run loop.
4.  **Wire up `camera.rs`**: Stop writing files; write to `DiskBuffer`.
5.  **Fix `query/executor.rs`**: Ensure it handles UUID parsing failures gracefully (currently logs nothing if ID isn't a UUID, just silently skips).

This concludes the audit. The system is currently a "Screen Logger" with broken appendages.
