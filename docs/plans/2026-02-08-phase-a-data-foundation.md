# Phase A: Data Foundation — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the data foundation required by Spec §4, §6, §8 — correct time model, blob CAS separation, and text search indexes — so that Phase B (query engine) can be built on solid ground.

**Architecture:** No proto wire-format changes. The proto messages are the collector→server wire format and stay as-is. All new fields (t_ingest, t_canonical, time_quality, blob_hash) are server-side additions to the DB record types and schema. Clock skew requires one small proto addition (time sample in ControlMessage). The existing `FsCas` in `utils/cas.rs` is wired into the ingest pipeline to store blobs. SurrealDB 2.x `DEFINE ANALYZER` + `SEARCH INDEX` provides full-text search.

**Tech Stack:** Rust, SurrealDB 2.3.1, prost/tonic protobuf, `FsCas` (SHA256 filesystem CAS)

---

## Task 1: Add Time Model Fields to DB Records

Spec §4.2/§4.2.1: Every record must store `t_device`, `t_ingest`, `t_canonical`, and `time_quality`. Currently records only have `timestamp` (which is device time). No proto changes needed — these are server-side DB enrichments.

**Files:**
- Modify: `common/lifelog-types/src/lib.rs` (Record types + ToRecord impls)
- Modify: `server/src/schema.rs` (DDL field + index definitions)
- Modify: `server/src/ingest.rs` (populate new fields at ingest time)
- Modify: `server/src/data_retrieval.rs` (handle new fields when reading back)
- Test: `just test` (existing unit tests must still pass)

### Step 1: Write a failing test for time fields in ScreenRecord

In `common/lifelog-types/src/lib.rs`, add a test that asserts `ScreenRecord` has `t_ingest` and `t_canonical` fields:

```rust
#[cfg(feature = "surrealdb")]
#[test]
fn test_screen_record_has_time_fields() {
    let frame = ScreenFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: Some(::pbjson_types::Timestamp { seconds: 12345, nanos: 0 }),
        width: 1920,
        height: 1080,
        image_bytes: vec![1, 2, 3],
        mime_type: "image/png".to_string(),
    };
    let record = frame.to_record();
    // t_device should equal the frame's timestamp
    assert_eq!(record.t_device, record.timestamp);
    // t_ingest and t_canonical are None until set by server
    assert!(record.t_ingest.is_none());
    assert!(record.t_canonical.is_none());
    assert!(record.time_quality.is_none());
}
```

### Step 2: Run test to verify it fails

Run: `just test`
Expected: Compilation error — `ScreenRecord` has no field `t_device`, `t_ingest`, etc.

### Step 3: Add time fields to all Record types

In `common/lifelog-types/src/lib.rs`, add to `ScreenRecord` (and all other `*Record` types):

```rust
#[cfg(feature = "surrealdb")]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ScreenRecord {
    pub uuid: String,
    pub timestamp: surrealdb::sql::Datetime,      // kept for backward compat / alias
    pub t_device: surrealdb::sql::Datetime,        // raw device time
    pub t_ingest: Option<surrealdb::sql::Datetime>, // server receipt time
    pub t_canonical: Option<surrealdb::sql::Datetime>, // skew-corrected time
    pub time_quality: Option<String>,               // "good"/"degraded"/"unknown"
    pub width: u32,
    pub height: u32,
    pub image_bytes: surrealdb::sql::Bytes,
    pub mime_type: String,
}
```

Update `ToRecord for ScreenFrame`:
```rust
fn to_record(&self) -> Self::Record {
    let ts = to_dt(self.timestamp);
    ScreenRecord {
        uuid: self.uuid.clone(),
        timestamp: ts.into(),
        t_device: ts.into(),
        t_ingest: None,      // Server fills this in
        t_canonical: None,   // Server fills this in
        time_quality: None,  // Server fills this in
        width: self.width,
        height: self.height,
        image_bytes: self.image_bytes.clone().into(),
        mime_type: self.mime_type.clone(),
    }
}
```

Apply the same pattern to ALL Record types:
- `BrowserRecord`, `OcrRecord`, `ProcessRecord`, `CameraRecord`, `AudioRecord`, `WeatherRecord`, `HyprlandRecord`

Each gets: `t_device`, `t_ingest: Option<_>`, `t_canonical: Option<_>`, `time_quality: Option<String>`.

### Step 4: Update schema DDL to include new fields

In `server/src/schema.rs`, add to every `TableSchema.fields_ddl`:

```sql
DEFINE FIELD t_device     ON `{table}` TYPE datetime;
DEFINE FIELD t_ingest     ON `{table}` TYPE option<datetime>;
DEFINE FIELD t_canonical  ON `{table}` TYPE option<datetime>;
DEFINE FIELD time_quality ON `{table}` TYPE option<string>;
```

Also add a canonical time index:
```sql
DEFINE INDEX `{table}_tcanon_idx` ON `{table}` FIELDS t_canonical;
```

### Step 5: Update ingest to populate time fields

In `server/src/ingest.rs`, after `frame.to_record()`, set the server-side fields:

```rust
let mut record = frame.to_record();
record.t_ingest = Some(chrono::Utc::now().into());
// For now, t_canonical = t_device (no skew correction yet)
record.t_canonical = Some(record.t_device.clone());
record.time_quality = Some("unknown".to_string());
```

This requires the macro to be updated to allow mutation of the record before upsert.

### Step 6: Update data_retrieval.rs for new fields

In `server/src/data_retrieval.rs`, the `get_data_by_key` function reads records and converts them back to proto frames. The new fields (`t_ingest`, etc.) are DB-only — the proto frame still uses `timestamp`. No changes needed to data_retrieval for the proto conversion, but the Record deserialization must handle the new fields (which it will, since they're `Option`).

**Important:** Existing DB records won't have the new fields. `Option<_>` with `#[serde(default)]` handles this gracefully.

Add `#[serde(default)]` to the Option fields:

```rust
#[serde(default)]
pub t_ingest: Option<surrealdb::sql::Datetime>,
#[serde(default)]
pub t_canonical: Option<surrealdb::sql::Datetime>,
#[serde(default)]
pub time_quality: Option<String>,
```

### Step 7: Run tests, verify passing

Run: `just test`
Expected: All tests pass.

### Step 8: Commit

```bash
git add common/lifelog-types/src/lib.rs server/src/schema.rs server/src/ingest.rs server/src/data_retrieval.rs
git commit -m "feat: add time model fields (t_device, t_ingest, t_canonical, time_quality) to DB records

Spec §4.2/§4.2.1: Every record now stores device time, ingest time,
canonical time (skew-corrected), and time quality indicator.
t_canonical defaults to t_device until clock skew integration (Task 2)."
```

---

## Task 2: Integrate Clock Skew Estimation

Spec §4.2.1: Backend maintains skew estimate per collector. `t_canonical = t_device + skew_estimate`. The algorithm already exists in `common/lifelog-core/src/time_skew.rs`.

**Files:**
- Modify: `proto/lifelog.proto` (add time sample to ControlMessage)
- Modify: `server/src/grpc_service.rs` (collect samples, compute skew)
- Modify: `server/src/server.rs` (store skew estimates per collector)
- Modify: `server/src/ingest.rs` (apply skew when computing t_canonical)
- Test: `just test`

### Step 1: Add clock sample to ControlMessage proto

In `proto/lifelog.proto`, add a new message and oneof variant:

```protobuf
message ClockSample {
  google.protobuf.Timestamp device_now = 1;
}
```

In `ControlMessage.oneof msg`, add:
```protobuf
ClockSample clock_sample = 6;
```

### Step 2: Store skew estimates in Server

In `server/src/server.rs`, add to the `Server` struct:

```rust
pub(crate) skew_estimates: Arc<RwLock<HashMap<String, lifelog_core::time_skew::SkewEstimate>>>,
skew_samples: Arc<RwLock<HashMap<String, Vec<(DateTime<Utc>, DateTime<Utc>)>>>>,
```

### Step 3: Collect clock samples in gRPC service

In `server/src/grpc_service.rs`, when processing `ControlMessage` with `clock_sample` variant:

```rust
ControlMessage { msg: Some(control_message::Msg::ClockSample(sample)), collector_id, .. } => {
    let device_now = /* convert sample.device_now to DateTime<Utc> */;
    let backend_now = chrono::Utc::now();
    // Store sample
    let mut samples = server.skew_samples.write().await;
    let entry = samples.entry(collector_id.clone()).or_default();
    entry.push((device_now, backend_now));
    // Keep only last 20 samples
    if entry.len() > 20 {
        entry.drain(..entry.len() - 20);
    }
    // Recompute estimate
    let estimate = lifelog_core::time_skew::estimate_skew(entry);
    server.skew_estimates.write().await.insert(collector_id, estimate);
}
```

### Step 4: Apply skew in ingest pipeline

In `server/src/ingest.rs`, the `persist_metadata` method needs access to the skew estimate for the collector. Pass the skew estimate (or a lookup function) and use it:

```rust
let t_device = to_dt(frame.timestamp);
let t_canonical = match skew_estimate {
    Some(est) => est.apply(t_device),
    None => t_device,  // No skew data yet
};
let time_quality = match skew_estimate {
    Some(est) => format!("{:?}", est.time_quality).to_lowercase(),
    None => "unknown".to_string(),
};
record.t_canonical = Some(t_canonical.into());
record.time_quality = Some(time_quality);
```

### Step 5: Collector sends clock samples

In `collector/src/collector.rs`, periodically send a `ClockSample` in the control stream (alongside heartbeats):

```rust
// Every heartbeat interval, also send a clock sample
let clock_sample = ControlMessage {
    collector_id: self.id.clone(),
    msg: Some(control_message::Msg::ClockSample(ClockSample {
        device_now: Some(chrono::Utc::now().into()),
    })),
};
```

### Step 6: Run tests, verify passing

Run: `just test`
Expected: All tests pass. Proto regeneration may require `just check` first.

### Step 7: Commit

```bash
git add proto/lifelog.proto server/src/server.rs server/src/grpc_service.rs server/src/ingest.rs collector/src/collector.rs
git commit -m "feat: integrate clock skew estimation into ingest pipeline

Spec §4.2.1: Collectors send periodic ClockSample messages.
Server estimates skew per collector using median+MAD algorithm.
t_canonical = t_device + skew_estimate at ingest time.
time_quality reflects estimation confidence."
```

---

## Task 3: Blob CAS Separation

Spec §8.2.1: Blobs must be stored in filesystem CAS, not inline in SurrealDB. Metadata records store `(blob_hash, blob_size, blob_codec)` references. The `FsCas` in `utils/cas.rs` already exists and is tested. The `Server` struct already has a `cas: FsCas` field.

**Files:**
- Modify: `common/lifelog-types/src/lib.rs` (change blob Record types)
- Modify: `server/src/schema.rs` (field type changes: bytes → string for hash)
- Modify: `server/src/ingest.rs` (store blob in CAS, store hash in DB)
- Modify: `server/src/data_retrieval.rs` (reconstruct frames from CAS + metadata)
- Modify: `server/src/transform.rs` (read blobs from CAS for OCR)
- Modify: `common/utils/src/ingest.rs` (IngestBackend trait needs CAS access)
- Test: `just test`

### Step 1: Write a failing test for blob-separated ScreenRecord

```rust
#[cfg(feature = "surrealdb")]
#[test]
fn test_screen_record_uses_blob_hash() {
    let record = ScreenRecord {
        uuid: "test".to_string(),
        timestamp: chrono::Utc::now().into(),
        t_device: chrono::Utc::now().into(),
        t_ingest: None,
        t_canonical: None,
        time_quality: None,
        width: 1920,
        height: 1080,
        blob_hash: "abc123".to_string(),
        blob_size: 1024,
        mime_type: "image/png".to_string(),
    };
    assert_eq!(record.blob_hash, "abc123");
    assert_eq!(record.blob_size, 1024);
}
```

### Step 2: Change Record types — replace inline bytes with blob references

For `ScreenRecord`:
```rust
pub struct ScreenRecord {
    pub uuid: String,
    pub timestamp: surrealdb::sql::Datetime,
    pub t_device: surrealdb::sql::Datetime,
    #[serde(default)]
    pub t_ingest: Option<surrealdb::sql::Datetime>,
    #[serde(default)]
    pub t_canonical: Option<surrealdb::sql::Datetime>,
    #[serde(default)]
    pub time_quality: Option<String>,
    pub width: u32,
    pub height: u32,
    pub blob_hash: String,     // SHA256 hex hash → CAS key
    pub blob_size: u64,        // original byte count
    pub mime_type: String,
}
```

Same pattern for:
- `CameraRecord`: `image_bytes` → `blob_hash` + `blob_size`
- `AudioRecord`: `audio_bytes` → `blob_hash` + `blob_size`
- `ClipboardRecord` (if binary_data is large): keep `text` inline, `binary_data` → optional `blob_hash`

### Step 3: Update ToRecord to NOT include blob bytes

`ToRecord for ScreenFrame` no longer copies `image_bytes` into the record. Instead, the blob is stored separately. But `to_record()` doesn't have CAS access, so it stores a placeholder:

```rust
fn to_record(&self) -> Self::Record {
    let ts = to_dt(self.timestamp);
    ScreenRecord {
        uuid: self.uuid.clone(),
        timestamp: ts.into(),
        t_device: ts.into(),
        t_ingest: None,
        t_canonical: None,
        time_quality: None,
        width: self.width,
        height: self.height,
        blob_hash: String::new(),  // Filled by ingest pipeline
        blob_size: self.image_bytes.len() as u64,
        mime_type: self.mime_type.clone(),
    }
}
```

### Step 4: Update schema DDL

In `server/src/schema.rs`, for Screen:
```sql
-- Remove: DEFINE FIELD image_bytes ON `{table}` TYPE bytes;
-- Add:
DEFINE FIELD blob_hash ON `{table}` TYPE string;
DEFINE FIELD blob_size ON `{table}` TYPE int;
```

Same for Camera (`image_bytes` → `blob_hash` + `blob_size`) and Audio (`audio_bytes` → `blob_hash` + `blob_size`).

### Step 5: Update ingest to store blobs in CAS

In `server/src/ingest.rs`, the ingest macro needs CAS access. The `SurrealIngestBackend` needs a `cas: FsCas` field.

For Screen ingestion:
```rust
"screen" => {
    if let Ok(frame) = ScreenFrame::decode(payload) {
        // Store blob in CAS
        let blob_hash = cas.put(&frame.image_bytes).map_err(|e| e.to_string())?;

        let mut record = frame.to_record();
        record.blob_hash = blob_hash;
        record.blob_size = frame.image_bytes.len() as u64;
        record.t_ingest = Some(chrono::Utc::now().into());
        record.t_canonical = Some(record.t_device.clone());

        // ... upsert to DB (without blob bytes)
    }
}
```

### Step 6: Update data_retrieval to reconstruct from CAS

In `server/src/data_retrieval.rs`, `get_data_by_key` must read the blob from CAS:

```rust
DataModality::Screen => {
    let record: ScreenRecord = db.select((&table, &*id)).await...;
    let image_bytes = cas.get(&record.blob_hash)
        .map_err(|e| LifelogError::Database(format!("CAS read: {e}")))?;

    let frame = ScreenFrame {
        uuid: record.uuid,
        timestamp: to_pb_ts(record.timestamp.0),
        width: record.width,
        height: record.height,
        image_bytes,
        mime_type: record.mime_type,
    };
    Ok(LifelogData { payload: Some(Payload::Screenframe(frame)) })
}
```

`get_data_by_key` signature changes to accept `&FsCas` parameter.

### Step 7: Update transform.rs for CAS-based blob access

`transform_data_single` calls `get_data_by_key` which now needs CAS. Pass `cas` through.

### Step 8: Run tests, verify passing

Run: `just test`
Expected: All tests pass. CAS unit tests already exist.

### Step 9: Commit

```bash
git add common/lifelog-types/src/lib.rs server/src/schema.rs server/src/ingest.rs \
        server/src/data_retrieval.rs server/src/transform.rs server/src/server.rs \
        common/utils/src/ingest.rs
git commit -m "feat: separate blobs to filesystem CAS, store hash references in DB

Spec §8.2.1: Screen, camera, and audio blobs now stored in FsCas.
DB records contain (blob_hash, blob_size, mime_type) references.
data_retrieval reconstructs full frames by reading from CAS.
Eliminates multi-MB inline bytes in SurrealDB rows."
```

---

## Task 4: Text Search Indexes

Spec §6.2.1/§8.3: Full-text search required for OCR text, browser URL/title, clipboard text, shell commands, keystroke text. SurrealDB 2.x supports `DEFINE ANALYZER` + `SEARCH INDEX` with BM25 scoring.

**Files:**
- Modify: `server/src/schema.rs` (add analyzer + search indexes)
- Test: Integration test (requires SurrealDB running)

### Step 1: Define a text analyzer

In `server/src/schema.rs`, add a startup migration for the analyzer:

```rust
static TEXT_ANALYZER_DDL: &str = r#"
    DEFINE ANALYZER lifelog_text TOKENIZERS blank, class FILTERS lowercase, ascii, snowball(english);
"#;
```

### Step 2: Add search indexes to text-bearing modalities

Update `indexes_ddl` for each modality that has searchable text:

**Ocr:**
```sql
DEFINE INDEX `{table}_ts_idx` ON `{table}` FIELDS timestamp;
DEFINE INDEX `{table}_tcanon_idx` ON `{table}` FIELDS t_canonical;
DEFINE INDEX `{table}_text_search` ON `{table}` FIELDS text SEARCH ANALYZER lifelog_text BM25;
```

**Browser:**
```sql
DEFINE INDEX `{table}_ts_idx` ON `{table}` FIELDS timestamp;
DEFINE INDEX `{table}_tcanon_idx` ON `{table}` FIELDS t_canonical;
DEFINE INDEX `{table}_url_search` ON `{table}` FIELDS url SEARCH ANALYZER lifelog_text BM25;
DEFINE INDEX `{table}_title_search` ON `{table}` FIELDS title SEARCH ANALYZER lifelog_text BM25;
```

**ShellHistory:**
```sql
DEFINE INDEX `{table}_command_search` ON `{table}` FIELDS command SEARCH ANALYZER lifelog_text BM25;
```

**Keystrokes:**
```sql
DEFINE INDEX `{table}_text_search` ON `{table}` FIELDS text SEARCH ANALYZER lifelog_text BM25;
```

**Clipboard:**
```sql
DEFINE INDEX `{table}_text_search` ON `{table}` FIELDS text SEARCH ANALYZER lifelog_text BM25;
```

**WindowActivity:**
```sql
DEFINE INDEX `{table}_title_search` ON `{table}` FIELDS window_title SEARCH ANALYZER lifelog_text BM25;
```

### Step 3: Update run_startup_migrations to create analyzer

```rust
pub(crate) async fn run_startup_migrations(db: &Surreal<Client>) -> Result<(), LifelogError> {
    // Create text analyzer first (idempotent)
    db.query(TEXT_ANALYZER_DDL)
        .await
        .map_err(|e| LifelogError::Database(format!("text analyzer: {}", e)))?;

    // ... existing migration code
}
```

### Step 4: Update query planner to use `@@` for text search

In `server/src/query/planner.rs`, the `Contains` expression currently uses `~` (case-insensitive regex). For fields with search indexes, use `@@` (full-text search operator):

```rust
Expression::Contains(field, text) => {
    // Use @@ for full-text search on indexed fields
    format!("{} @@ {}", field, Self::quote_string(text))
}
```

**Note:** The `@@` operator only works on fields with a SEARCH INDEX. For non-indexed fields, fall back to `~` or `CONTAINS`. For v1, all text fields that we search are indexed, so `@@` is safe.

### Step 5: Run tests

Run: `just test`
Expected: Unit tests pass. Full-text search needs integration test.

### Step 6: Commit

```bash
git add server/src/schema.rs server/src/query/planner.rs
git commit -m "feat: add full-text search indexes for text-bearing modalities

Spec §6.2.1/§8.3: DEFINE ANALYZER lifelog_text with BM25 scoring.
Search indexes on: OCR text, browser URL/title, clipboard text,
shell commands, keystroke text, window titles.
Query planner uses @@ operator for full-text search."
```

---

## Task 5: Durable ACK Enhancement

Spec §6.2.1: ACK means "fully queryable" — metadata persisted, blobs persisted, baseline indexes updated. Currently ACK fires after metadata write but doesn't verify index readiness.

**Files:**
- Modify: `server/src/ingest.rs` (verify after upsert)
- Test: `just test`

### Step 1: Verify blob + metadata persistence before marking indexed

In `server/src/ingest.rs`, the `indexed` flag should only be set true when:
1. Blob is in CAS (if applicable)
2. Metadata record is persisted in DB
3. Record is queryable (SurrealDB search indexes update synchronously with writes for SEARCH indexes)

The current flow already sets `indexed = true` only after successful upsert. With CAS separation (Task 3), we also verify the blob was stored:

```rust
// In the ingest macro, after CAS put and DB upsert:
let blob_stored = cas.contains(&blob_hash).unwrap_or(false);
let db_stored = result.is_some();
indexed = blob_stored && db_stored;
```

**Note:** SurrealDB SEARCH indexes are updated synchronously during `UPSERT`, so if the upsert succeeds, the search index is already updated. No async index lag to worry about.

### Step 2: Add a verification query (optional hardening)

For extra safety, after upsert, run a quick verification:

```rust
// Verify the record is queryable by timestamp (proves time index works)
let verify_sql = format!(
    "SELECT count() FROM `{}` WHERE timestamp = $ts GROUP ALL",
    table
);
let verify_result = db.query(verify_sql).bind(("ts", record.timestamp.clone())).await;
// If verify fails, don't set indexed = true
```

This is optional and adds latency. For v1, trusting SurrealDB's synchronous index updates is sufficient.

### Step 3: Run tests

Run: `just test`
Expected: All tests pass.

### Step 4: Commit

```bash
git add server/src/ingest.rs
git commit -m "fix: strengthen durable ACK to verify blob + metadata persistence

Spec §6.2.1: ACK implies fully queryable. indexed flag now requires
both CAS blob storage and DB metadata upsert to succeed."
```

---

## Execution Dependencies

```
Task 1 (Time Fields) ──→ Task 2 (Clock Skew)
                    ╲
                     ╲──→ Task 5 (ACK)
Task 3 (Blob CAS) ──╱
Task 4 (Text Search)╱
```

Tasks 1, 3, and 4 can be developed in parallel (no dependencies between them).
Task 2 depends on Task 1 (needs time fields to populate).
Task 5 depends on Tasks 3 and 4 (needs CAS and indexes to exist).

## Validation Gate

After all 5 tasks, run: `just validate` (fmt + check + clippy + test)

The following should work:
- New records stored with t_device, t_ingest, t_canonical, time_quality
- Screen/camera/audio blobs stored in `cas/` directory, not in DB
- Full-text search queries return results via `@@` operator
- Clock samples flow from collector → server → skew estimate → t_canonical
- ACK only fires when blob + metadata + indexes are all confirmed
