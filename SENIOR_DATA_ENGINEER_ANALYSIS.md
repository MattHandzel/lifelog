# Senior Data Engineer (Priya Mehta) Review: Lifelog Data Layer

This review covers the current repository implementation plus `SPEC.md`, with emphasis on: database choice, schema modeling for multi-modal time-indexed data, query execution (cross-modal predicates), ingestion semantics, performance, and storage/retention.

## 1. Repo Reality Check (What Exists Today)

### 1.1 Backend DB: SurrealDB (Remote WS + RocksDB)

The server connects to SurrealDB over WebSockets and uses RocksDB as the storage engine (see `server/src/server.rs`). Tables are created dynamically per `DataOrigin` (device + modality, and nested origins for transforms) using SurrealDB DDL built from modality structs (e.g. `common/data-modalities/src/screen/data.rs`).

Notable current properties:

- **Blob bytes stored inline**: `ScreenFrame` stores `image_bytes` in the DB row (`bytes`), which `SPEC.md` explicitly calls out as a divergence (needs metadata/blob separation).
- **No indexes**: table DDL currently defines fields but does not define time or text indexes (there’s commented-out code suggesting intent).
- **Record identity is the Surreal record ID**: inserts use `create((table, uuid.to_string()))`, so the UUID is effectively used as the record key, but the stored struct does not include `uuid` (it’s reconstructed later by overwriting `uuid` after fetching).

### 1.2 Query Engine: Stubbed

`Server::process_query()` ignores the incoming query and returns **all UUIDs for all origins** by doing:

1. `INFO FOR DB` to list all tables
2. parse table names back into `DataOrigin`
3. for each table: `SELECT VALUE record::id(id)` to fetch all record IDs

So the system currently does not implement:

- parsing the spec’s SQL-like/DSL query syntax
- text predicates (URL/OCR)
- cross-modal correlation operators (WITHIN/OVERLAPS/DURING)
- time range restriction

This is consistent with `SPEC.md` “Known Divergences”: query semantics are stubbed.

### 1.3 Transform Model: Full Table Scans + Set Difference in Memory

Transforms (OCR) are implemented as:

- read **all UUIDs** from source table
- read **all UUIDs** from destination table
- compute `source - destination` in memory

This is an O(N) full scan of both tables every transform pass and becomes a hard wall once you have real lifelog volumes.

### 1.4 Ingestion / Buffering: In-Memory Buffers, No Ordering Guarantees

Collectors currently:

- buffer screen frames in-memory (`Vec<ScreenFrame>` guarded by a mutex in the data source)
- on `GetData` streaming RPC, dump the whole buffer and **clear it**
- do not persist a WAL / disk queue
- do not support offsets, chunk hashes, idempotent retries, or durable ACK semantics beyond the RPC returning successfully

This violates the v1 invariants in `SPEC.md` (durable buffering, resumable upload, idempotency key including session/offset/hash, etc.).

### 1.5 Proto Contract Is Currently Broken / Incomplete

`proto/lifelog.proto` contains a malformed `LifelogDataKey`:

- it includes a stray `timestamp` token with no field number/type

Modality coverage is also incomplete (`DataModality` is only `{Browser,Ocr,Screen}` in `proto/lifelog_types.proto`), while `SPEC.md` expects audio, window/app activity, input events, clipboard, shell history, etc.

## 2. Database Selection: Is SurrealDB the Right Fit?

### 2.1 What Lifelog Needs From a DB (Data-Layer Perspective)

Lifelog’s core workload is:

- high-write append-only ingestion (per device, per modality)
- time-range scans (timeline, replay)
- text search (OCR, URL/title, commands, clipboard)
- cross-modal correlation (join/overlap by time windows)
- incremental transforms (OCR, embeddings) driven by “new data since last cursor”
- local-first reliability (single user, but multiple device ingesters; UI queries concurrently)

You want:

- strong/clear transactional semantics for “durable ACK” points
- first-class indexing on time and range overlap
- strong text-search primitives
- predictable query planning (so correlation queries don’t devolve into full scans)

### 2.2 SurrealDB (Current Choice): Pros/Cons

Pros:

- flexible schema evolution
- unified-ish query language (SurrealQL) that is friendlier than raw key-value stores
- easy to spin up locally with RocksDB backend

Cons for this project:

- time-series + cross-modal overlap joins are *not* SurrealDB’s core “happy path”; you’ll end up implementing a lot of window logic yourself, and you need to verify index capabilities for the specific overlap predicates you care about.
- current design trend in this repo (“one table per origin string”) pushes you toward **many tables** and makes global query planning harder (and `INFO FOR DB` table discovery is already used as a “catalog”).
- you still need an external blob store for high-entropy payloads; SurrealDB doesn’t remove that requirement, and storing bytes inline is already a divergence.

Net: SurrealDB can work, but it’s an uphill battle relative to a relational engine for the exact query class you’re targeting (time windows + text predicates + joins).

### 2.3 SQLite: Surprisingly Strong for v1, With Caveats

SQLite with:

- WAL mode
- `FTS5` for text search
- careful schema (time columns as INTEGER epoch micros)
- per-stream indexes

…can absolutely power a local-first v1 for a single user. It’s operationally simple (no daemon), ships well, and is reliable.

Caveats:

- interval overlap joins are doable (`a.start < b.end AND a.end > b.start`) but lack a native `tsrange`/GiST equivalent; you’ll rely on composite B-tree indexes and sometimes precomputed window tables.
- high concurrency across multiple writers is limited (still workable if you funnel writes through a single ingest process).
- “hot/cold” tiering is not built in; you’ll implement it with separate DB files or an external cold store.

### 2.4 Postgres: Best Default for Cross-Modal Joins + Text

Postgres gives you:

- robust concurrency for multi-device ingest + UI queries
- rich indexing (B-tree, GIN for text, GiST for range overlap)
- native interval types via `tsrange` (or `tstzrange`)
- predictable query planner and explainability (critical for performance work)

For your canonical query:

> “Return audio chunks during times when url contains youtube and OCR contains 3Blue1Brown”

Postgres is the cleanest target because you can:

- find match records in browser/OCR via text search indexes
- expand them into time windows
- merge/intersect windows
- range-join audio chunks via GiST on `tsrange`

### 2.5 TimescaleDB: Postgres Plus “Lifelog-Scale” Features

Timescale adds:

- hypertables and time partitioning
- native compression policies
- retention policies
- helpful time-bucket utilities

If you expect “years of data, always-on capture”, Timescale becomes compelling. The tradeoff is operational complexity (extension install) vs SQLite.

### 2.6 DuckDB: Excellent Cold Store, Not a Great Hot Store

DuckDB excels at:

- querying Parquet/columnar cold data
- long-range analytic scans

But it’s not ideal as the live ingestion store for a concurrently queried, always-on server. A strong pattern here is:

- **hot store**: SQLite or Postgres/Timescale
- **cold store**: Parquet partitioned by `(device, modality, day)` queried via DuckDB

### 2.7 Recommendation

Given `SPEC.md` requirements (cross-modal correlation operators, text search, durable ACK semantics), I would recommend:

1. **Postgres (optionally TimescaleDB)** for metadata + indexes + query engine target.
2. **Content-addressed blob store on the filesystem** for images/audio/files.
3. **Optional cold tier**: Parquet + DuckDB for older partitions.

If you are optimizing for “single binary / minimal ops”, then:

1. **SQLite (WAL + FTS5)** for metadata + text
2. filesystem CAS blob store
3. DuckDB later for cold scans

Staying on SurrealDB is viable only if you commit to:

- strict blob separation
- explicit time indexes
- a query planner that avoids multi-table discovery and full scans

## 3. Schema Design: Modeling Multi-Modal Time Data

### 3.1 Key Design Principle: Separate Core Time Index From Modality Payload

Almost every query needs time/device/stream metadata. Only some queries need modality-specific columns. So you want:

- a **core records table** with canonical time semantics and identity
- **modality tables** for payload and modality-specific indexes
- **blob references** instead of inline bytes

This also makes transforms and lineage explicit and incremental.

### 3.2 Canonical Entities

**streams**

- `stream_id` (UUID/ULID)
- `collector_id` (stable device identity)
- `device_id` (if distinct)
- `modality` (enum)
- `logical_name` (e.g., `laptop.screen`, `desktop.microphone`)
- capture metadata (sample rate, frame interval, etc.)

**records** (append-only)

- `record_id` (UUIDv7/ULID)
- `stream_id` (FK)
- `seq` (monotonic per stream, UNIQUE(stream_id, seq))
- `t_device` (timestamp as seen by collector)
- `t_ingest` (timestamp persisted by backend)
- either:
  - point record: `t_start = t_device`, `t_end = NULL`
  - interval record: `t_start`, `t_end`
- `clock_offset_estimate` / `time_quality` (optional, for skew correction)

**lineage**

- `output_record_id`
- `input_record_id`
- `transform_id`
- `transform_version`

**blobs**

- `blob_hash` (sha256/blake3)
- `codec` (png/webp/avif, opus/flac, etc.)
- `size_bytes`
- `storage_path` (or derived from hash prefix)
- `created_at`

**record_blobs**

- `record_id`
- `blob_hash`
- `role` (e.g., `screen_image`, `audio_chunk`, `thumbnail`)

### 3.3 Modality Payload Tables (Examples)

**screen_frames**

- `record_id` (PK/FK -> records)
- `width`, `height`
- `active_window_title` (eventually)
- `mime_type`
- `blob_hash` (FK -> blobs) OR via `record_blobs`

**audio_chunks**

- `record_id`
- `sample_rate`, `channels`, `bits_per_sample`
- `blob_hash`

**browser_events**

- `record_id`
- `url`, `title`, `visit_count`
- (optional) `is_active_tab`, `window_id`, etc.

**ocr_text**

- `record_id`
- `text`
- `source_record_id` (FK -> records) if you prefer explicit backref over lineage join

### 3.4 Normalized vs Denormalized Tradeoffs

Normalized (recommended here):

- pros: stable query patterns, shared time semantics, easier to tier/cold-store, good indexing strategy
- cons: more joins, more tables

Denormalized (single wide table per modality/origin):

- pros: simple inserts, simple “get by id”
- cons: cross-modal joins become messy; lineage and transforms get ad hoc; blob separation is harder; indexes become inconsistent across tables

Given the product’s defining feature is **cross-modal correlation**, normalized core-time + modality tables is the better long-term bet.

## 4. Query Engine: “SQL-like” Cross-Modal Predicates

### 4.1 Don’t Treat This As “Just SQL Parsing”

The spec’s canonical query:

> `SELECT laptop.microphone WHERE 'youtube' in laptop.browser.url AND '3Blue1Brown' in laptop.screen.ocr`

is not standard SQL. It’s a *correlated multi-stream query* that compiles to a plan:

1. identify time windows where predicates over streams are true
2. return records from stream A overlapping those windows

That’s closer to **relational algebra + temporal operators** than “SELECT ... WHERE ...”.

### 4.2 Suggested Internal Representation

Define a typed AST:

- `StreamRef(device, modality, transform_chain?)`
- predicate nodes:
  - `TextContains(stream, field, literal)`
  - `TimeRange(stream, [start,end])`
  - boolean ops: `AND`, `OR`, `NOT`
- correlation ops:
  - `WITHIN(lhs, rhs, delta)`
  - `OVERLAPS(lhs_interval, rhs_interval)`
  - `DURING(target_stream, predicate_over_other_streams)`

Compile to an execution plan that can target Postgres/SQLite.

### 4.3 Efficient Execution Strategy (Window-First)

For most queries, do not join “all audio to all OCR” directly. Do:

1. **Filter predicate streams first** using their best indexes (FTS, url indexes, etc.)
2. Convert predicate matches to **time windows**:
   - point -> `[t - Δ, t + Δ]`
   - interval -> `[start - Δ, end + Δ]`
3. **Merge windows** per predicate (union of overlapping windows)
4. **Intersect** windows across predicates (AND semantics)
5. Range-join the target stream against the resulting window set

This turns “cross-modal join” into “range join with a small window table”, which is the difference between milliseconds and minutes.

### 4.4 Concrete SQL Shape (Postgres)

Conceptually:

- CTE `browser_hits` from `browser_events` using trigram/ILIKE or URL normalization
- CTE `ocr_hits` from `ocr_text` using `tsvector`/GIN
- CTE `windows`:
  - project to `tstzrange(...)`
  - union + merge (you can do merging with window functions or custom range aggregation)
- final:
  - `SELECT ... FROM audio_chunks a JOIN windows w ON a.range && w.range`

Key index:

- `audio_chunks(range)` GiST
- `ocr_text(text_tsv)` GIN
- `browser_events(url)` (GIN trigram) or `url_host` derived column

SQLite variant:

- store `t_start_us`, `t_end_us` as integers
- windows as temp table or CTE of intervals
- overlap join via `a.t_start_us < w.end_us AND a.t_end_us > w.start_us`

### 4.5 “SQL-like Interface” Recommendation

Externally, you can expose:

1. a small custom DSL (as spec suggests)
2. and/or a JSON typed query (AST) for the UI

Internally, compile to SQL for the chosen DB. Avoid implementing a whole database inside your server: lean on proven planners.

## 5. Cross-Modal Joins: Overlapping Time Windows

The hard requirement is joining point and interval streams with clock skew.

### 5.1 Represent Time Correctly

You need both:

- `t_device` and `t_ingest` (spec)
- `seq` per stream (monotonic ordering)

For intervals (audio chunks), store:

- `t_start`, `t_end` (or `duration_ms`)

For points (screen frames), treat as:

- `t_start = t_device`
- optionally assign a small “support” interval for replay (`[t, t + frame_interval)`), but keep the raw capture time too.

### 5.2 Overlap Predicates

Core overlap:

- `A` overlaps `B` iff `A.start < B.end AND A.end > B.start`

WITHIN for point-point:

- `abs(a.t - b.t) <= Δ`

WITHIN for point-interval:

- `a.t BETWEEN (b.start - Δ) AND (b.end + Δ)`

### 5.3 Indexing For Overlap

Postgres:

- use `tstzrange(t_start, t_end, '[)')` + GiST

SQLite:

- index `(stream_id, t_start_us)` and `(stream_id, t_end_us)`
- queries typically constrain by time range first, then apply overlap predicate

At scale, you generally prefilter by coarse time buckets (day/hour) to bound overlap checks.

## 6. Ingestion: Buffering, Dedup, Ordering, Batch vs Streaming

### 6.1 Collector-Side: Disk WAL/Queue Per Stream

To meet `SPEC.md` invariants, collectors need:

- append-only segment files per stream (e.g., `stream_id/000001.log`)
- a manifest of committed segments
- a monotonic `seq` assigned at capture time
- periodic fsync boundary (or group commit)

### 6.2 Upload Protocol: Offsets + Idempotency Key

The spec’s idempotency key shape is correct:

- `(collector_id, stream_id, session_id, offset, chunk_hash)`

Backend stores an upload cursor per `(collector_id, stream_id, session_id)` and responds with “last durable offset”.

### 6.3 Backend ACK Semantics

ACK only after:

1. metadata inserted (transaction committed)
2. blobs persisted + hash verified
3. baseline indexes updated (time index + text index for extracted fields)

This implies an ingest transaction boundary that includes:

- metadata write
- blob write is outside the DB, but you can:
  - write blob to temp path
  - fsync + rename into CAS location
  - then commit DB transaction referencing it

### 6.4 Batch Inserts

Do micro-batches:

- batch by stream and by time ordering
- use `COPY` (Postgres) or prepared batched inserts (SQLite)
- avoid per-record roundtrips

### 6.5 Dedup Strategy

Two layers:

- **record dedup**: UNIQUE(stream_id, seq) and/or deterministic record_id (e.g., UUIDv7 + seq)
- **blob dedup**: content-addressed by hash, so duplicates become no-ops

## 7. Performance: Indexing, Partitioning, Query Optimization

### 7.1 Partitioning

Postgres:

- partition `records` and large modality tables by time (daily/monthly) and optionally by device
- Timescale hypertables if using Timescale

SQLite:

- consider “one DB per month” or “one DB per device” to keep b-trees smaller
- or keep one DB but be strict about indexes and vacuum strategy

### 7.2 Indexing Baseline

You need at minimum:

- `(stream_id, t_start)` B-tree for time scans per stream
- `t_start` BRIN (Postgres) for large append-only tables
- text search indexes:
  - Postgres GIN on `tsvector(text)`
  - SQLite FTS5 virtual tables
- overlap/range index if using Postgres (`GiST` on range)

### 7.3 Query Optimization Tactics

- Always constrain by time early when possible (UI typically has a visible window).
- Turn text predicates into candidate time windows first (window-first plan).
- Materialize merged windows as a temp table for multi-stage queries (especially replay).
- Maintain transform cursors to avoid full scans.

## 8. Storage: Compression, Hot/Cold, Retention

### 8.1 Blob Store Layout

Use a filesystem CAS:

- path by hash prefix: `blobs/ab/cd/<hash>`
- sidecar metadata: codec, duration, width/height, etc. either in DB or small JSON

### 8.2 Compression Choices (Pragmatic Defaults)

- screen frames:
  - store as `webp` or `avif` (huge win vs PNG/raw)
  - generate thumbnails for UI
- audio:
  - store as `opus` in `.ogg` container for voice; `flac` if you truly need lossless
- text:
  - keep plain in DB; optionally compress large fields with TOAST (Postgres) or application-level zstd for big documents

### 8.3 Hot/Cold Tiering

Hot:

- last N days/weeks in the DB optimized for interactive queries

Cold:

- export older partitions to Parquet (partitioned by day/device/modality)
- query cold store via DuckDB when needed

Retention:

- coarse policies per modality (audio can be huge; screenshots huge; text smaller)
- implement “delete by partition” not “delete row-by-row”

## 9. Specific Gaps and Fixes for This Repo

1. **Stop storing bytes inline in SurrealDB** (`ScreenFrame.image_bytes` in DB row). Replace with blob refs.
2. **Implement durable buffering** on collectors (disk WAL) before any serious ingestion work.
3. **Fix proto contracts**:
   - repair `LifelogDataKey` definition in `proto/lifelog.proto`
   - add interval record semantics and modalities needed by `SPEC.md`
4. **Replace transform full scans** with a cursor/watermark model:
   - per `(transform, source_stream)` store `last_seq` or `last_t_device`
5. **Add indexes**:
   - time indexes per table/stream
   - text search index for OCR and browser
6. **Query language**:
   - define AST + planner
   - compile to SQL on the chosen DB
   - implement correlation ops as window-first plans

## 10. Bottom Line

The current implementation (SurrealDB + per-origin tables + inline blobs + stubbed query + memory buffers) is a good prototype scaffold but is not aligned with the v1 spec’s reliability and cross-modal query requirements.

If you want the spec’s “overlapping time window cross-modal queries” to be fast and explainable, a relational metadata store (Postgres/Timescale or SQLite) with strong time/range/text indexing plus a filesystem blob store is the most defensible data-layer architecture.

