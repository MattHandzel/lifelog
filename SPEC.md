# Lifelog v1 Technical Specification

This document specifies the v1 system you are rebuilding: a local-first, multi-device lifelog platform centered on **recall** (timeline + search + replay) and **cross-modal retrieval** (use one stream to filter/locate another).

This spec is written to minimize rework by pinning down: goals, invariants, data/time semantics, network contracts, query language requirements, storage boundaries, and reliability guarantees.

---

## 0. Definitions

- **Backend**: the central authority that schedules ingestion, stores data durably, runs transforms, and serves the UI/API.
- **Collector**: a device program that captures raw streams, buffers them durably, and follows backend control.
- **Stream**: an ordered sequence of records from a single source and modality (e.g., “laptop screen”, “desktop audio”).
- **Record**: an immutable item in a stream. Records are either point events or time intervals.
- **Blob**: large payload content (image/audio) stored separately from metadata, referenced by content address.
- **Transform**: a deterministic function that produces a derived stream from one or more input streams (e.g., OCR from screen).
- **Query**: a deterministic, typed expression compiled to an executable plan, supporting cross-stream correlation.

---

## 1. Product Goals (What v1 must do)

### 1.1 Primary Goal: Recall Anything

The system must allow the user to recall any piece of information from any point in time, by:

- browsing a timeline,
- searching across modalities,
- replaying a time window step-by-step.

### 1.2 Cross-Modal Reconstruction (Core Capability)

The system must support queries of the form:

> Return items from stream A during times where conditions over streams B/C/D were true.

Example target capability:

> Retrieve microphone audio during times where browser URL contains “youtube” and screen OCR contains “3Blue1Brown”.

### 1.3 Passive Capture, Zero Maintenance

After initial setup, no manual “organize/tag/save” is required. Failures are surfaced as quiet alerts (health/status), not as interactive debugging tasks.

### 1.4 Local-First Privacy

All ingestion, storage, processing, and queries execute on user-controlled machines by default. No cloud sync in v1.

---

## 2. Non-Goals (v1 explicitly does not do)

- Cloud sync, cloud indexing, or remote storage targets.
- A “quantified-self analytics” UI (dashboards, long-term stats). Recall UI only.
- Third-party “agent ecosystem” as a supported product surface (can be designed for later).
- Perfect privacy filtering (incognito/password field detection) beyond minimal safety rails (see Security).
- Real-time event-driven capture optimization (v1 uses fixed-interval capture).

---

## 3. v1 Data Modalities (Required)

Collectors must produce at least these streams:

### 3.1 Desktop/Laptop (Primary)

- Screen capture (fixed interval) + metadata (resolution, active window/app).
- Desktop microphone audio capture (fixed interval chunking).
- Browser activity (URLs/titles/visit events; optionally “active tab” snapshots).
- App/window activity (active window title, process, workspace).
- Keystrokes (content policy TBD; minimum required: key events + timestamps).
- Mouse events (minimum: activity indicators + timestamps).
- Clipboard history (text + timestamps; binary clipboard optional).
- Shell history (commands + timestamps + working directory if feasible).

Implementation note: in this repo these are configured via `CollectorConfig.clipboard` and
`CollectorConfig.shell_history` (both default to disabled).

### 3.2 Secondary Devices (v1-ready but not required day 1)

- Phone: optional capture streams; must at least be a UI client (web UI).
- Wearable glasses/camera: ingest path must exist conceptually (blob + metadata), even if capture implementation is later.
- Watch: periodic health metrics ingestion later; backend must support interval time-series records.

---

## 4. Time Model and Correlation Semantics (Hard Requirement)

Cross-modal retrieval depends on consistent time semantics.

### 4.0 Canonical Timeline Time (Decision)

The canonical time used for UI timeline ordering and for query evaluation is:

- **Device-reported time corrected by an estimated clock skew** per collector.

Backend must still store raw device time and ingest time, but query/UI semantics are defined in terms of the corrected device time.

### 4.1 Record Types

- **Point record**: occurs at a timestamp `t`.
- **Interval record**: valid over `[t_start, t_end)`.

Examples:

- Screen capture: point record at capture time.
- Audio chunk: interval record over chunk duration.
- Active window: interval record over time window is active (if captured that way), or point samples (v1 can start as point).
- Keystrokes: point records.

### 4.2 Clock and Ordering

- Each collector must maintain a monotonic local ordering for each stream (sequence numbers).
- Backend must tolerate clock skew across devices.
- Backend must store both:
  - **device time** (as reported),
  - **ingest time** (backend receipt/persistence time).

#### 4.2.1 Clock Skew Model (Required)

- Backend maintains an estimated clock skew per collector: `skew_estimate = backend_now - device_now` sampled over time.
- Mechanism: backend periodically sends a clock-sync command over the control channel containing `backend_now`; collector replies with a clock sample containing both `backend_now` (echoed) and `device_now` (local).
- Backend stores, per record:
  - `t_device` (raw),
  - `t_ingest` (raw),
  - `t_canonical = t_device + skew_estimate_at_ingest` (used for query/UI).
- Wire format: all modality frame messages include optional time fields (`t_device`, `t_ingest`, `t_canonical`, `t_end`, `time_quality`) in addition to the legacy `timestamp` field. `timestamp` is treated as the device-reported time for backward compatibility.
- If skew cannot be estimated reliably (collector offline, inconsistent samples), the backend must mark records with a `time_quality` level so the UI can surface “time may be off”.

### 4.3 Correlation Operators

Queries must support correlation across streams using explicit operators:

- `WITHIN(A, B, ±Δt)`: records in A match conditions in B if timestamps are within `Δt`.
- `OVERLAPS(intervalA, intervalB)`: interval overlap.
- `DURING(A, predicateOverOtherStreams)`: restrict records from A to times where predicate holds.

v1 must define a global default correlation window `Δt_default` (configurable), and allow per-predicate overrides.

Implementation note: this repo represents `Δt_default` as `ServerConfig.default_correlation_window_ms`. Temporal operators (`WITHIN`/`DURING`/`OVERLAPS`) use the per-term window when provided, otherwise they fall back to `Δt_default`.

---

## 5. System Architecture (Backend-first, distributed)

### 5.1 Core Entities

1. **Collectors** run on devices; they capture and buffer locally.
2. **Backend** is the authority:
   - decides when to pull data,
   - stores durably,
   - runs transforms/indexing,
   - serves API and UI.
3. **Interface** is a web UI served by the backend, usable from desktop and phone.

### 5.2 Control Plane vs Data Plane

The system has two logical channels:

- **Control Plane (long-lived)**:

  - collector registration/enrollment,
  - config distribution,
  - health/state reporting,
  - backend-issued commands (e.g., “begin upload session”, “pause/resume capture”).

- **Data Plane (bulk transfer)**:
  - chunked upload of buffered data + blobs,
  - offset-based resume,
  - durable acknowledgements.

### 5.3 Connectivity Model (NAT-safe)

Collectors must initiate outbound connections to the backend (works behind NAT and on roaming devices). Backend “pulls” by sending commands over the established control channel.

### 5.4 API Surface (Decision)

v1 uses **gRPC-only** for all external interfaces:

- Collector <-> Backend: gRPC.
- UI client <-> Backend: gRPC.

No separate REST “logger server” is part of the v1 architecture. If an HTTP gateway is added later, it must be a façade over the same canonical backend, not a second backend with its own storage.

---

## 6. Reliability Guarantees (Invariants)

v1 is required to implement “store everything, don’t lose anything” as explicit guarantees.

### 6.1 Collector Buffering

- Collectors must buffer durably on disk.
- Buffer is append-only (WAL/queue semantics).
- Buffer retention is based on backend acknowledgements.

### 6.2 End-to-End Durable Acknowledgement

The backend must only acknowledge data as “delivered” when:

- metadata is persisted, and
- blobs are persisted (or content-addressed and verified), and
- **all baseline indexes required for full queryability are updated** (see Section 6.2.1).

#### 6.2.1 Baseline Index Set (Decision: ACK implies “fully queryable”)

v1 treats ACK as “fully queryable”. Therefore, ACK requires completion of:

- **Time-range index** for all ingested records (per stream).
- **Text search index** for all configured searchable text fields, including at minimum:
  - screen OCR text (derived stream),
  - browser URL + title,
  - clipboard text,
  - shell command text,
  - keystroke captured text (if enabled; see Section 12.4).

Vector/embedding indexes are explicitly not baseline for ACK in v1.

### 6.3 Resumable Upload

- Uploads are resumable at chunk boundaries via offsets/checkpoints.
- Backend must expose “what offset do I have durably stored for this session/stream?”.

### 6.4 Idempotency and Deduplication

All chunk writes must be idempotent:

- identity key includes `(collector_id, stream_id, session_id, offset, chunk_hash)`.
- backend must safely accept retries without duplicating logical records.

---

## 7. Protocol Requirements (API contracts, not implementation)

This spec is transport-agnostic, but v1 should use gRPC/Protobuf-style typed APIs (as the existing repo does).

### 7.1 Collector Enrollment and Identity

- Each collector has a stable `collector_id` (device identity).
- Enrollment must prevent arbitrary devices from registering (pairing model required; see Security).

### 7.2 Control Plane APIs (minimum set)

Collectors -> Backend:

- `RegisterCollector`: announce identity + capabilities + versions.
- `ReportState`: send health/state snapshot (buffers, capture status, last capture times).
- `SuggestUpload`: collector indicates readiness (charging, idle, on LAN, backlog high).

Backend -> Collectors (via control channel):

- `PushConfig`: deliver configuration updates; collector must acknowledge apply.
- `BeginUploadSession`: instruct collector to upload specific streams and time ranges.
- `PauseCapture` / `ResumeCapture`: minimal safety rail.

### 7.3 Data Plane APIs (minimum set)

Collector -> Backend:

- `UploadChunks(stream Chunk) -> Ack`:
  - chunked streaming RPC (or equivalent) with offsets.
  - supports multiple logical streams (either one stream per session or multiplexed with explicit stream id).

Backend -> Collector:

- `GetUploadOffset(stream_id, session_id) -> Offset`:
  - returns last durably persisted offset for resuming.

#### 7.3.1 Offset Unit (Decision)

For v1, `offset` refers to a **byte offset within a per-stream session file** (classic chunked upload).

Note: collectors must still maintain a per-stream monotonic record sequence number for ordering (Section 4.2). This sequence number is not the resumable upload offset.

### 7.4 Cancellation and Backpressure

- Backend must be able to cancel an in-progress upload session.
- Collectors must handle backpressure: pause sending, keep buffering.

---

## 8. Storage and Indexing (Backend internals; must satisfy queries)

Collectors must not depend on backend storage details. Backend must implement storage that supports:

### 8.0 Hot Store Database (Decision)

v1 uses **SurrealDB** as the hot metadata store, with the following constraints derived from repo analysis:

- Do not store large blobs inline in DB rows (use filesystem CAS).
- Define explicit indexes required for timeline and text search (Section 6.2.1).
- Avoid “table discovery via `INFO FOR DB`” as a runtime catalog mechanism; maintain an explicit stream/origin catalog table for stable query planning.
- Query execution must not devolve into full-table scans for transforms or queries (incremental cursors required; Section 9.2).

### 8.1 Metadata Store

For every record:

- unique record id,
- `collector_id`, `stream_id`, modality,
- device timestamp + ingest timestamp,
- point vs interval semantics,
- searchable fields (e.g., URL/title/command/clipboard text, OCR text pointer),
- blob references (content address) when applicable,
- lineage pointers for derived records.

### 8.2 Blob Store

Blobs must be:

- content-addressed (hash-based) or otherwise integrity-verified,
- referenced from metadata (never duplicated logically),
- segmentable for replay (audio/video chunk boundaries).

#### 8.2.1 Blob Store Implementation (Decision)

v1 blob storage is a **filesystem content-addressed store (CAS)**:

- Objects are addressed by content hash.
- Metadata records store blob references (hash + codec + size).
- Optional future “cold tier” export is supported (Parquet manifests + blobs), but is not required for v1 query execution.

### 8.3 Indexes

v1 must provide:

- time index (range scan by time, per stream),
- text search index for:
  - OCR text,
  - browser URL/title,
  - clipboard text,
  - shell commands,
  - (optional) keystroke content if stored as text.

Vector index can be added later but v1 should reserve schema hooks for embeddings.

---

## 9. Transforms (Derived Streams)

v1 must implement:

### 9.1 OCR Transform

- Input: screen capture blobs.
- Output: text records associated with the source screen record id and time.
- Storage: derived stream with lineage (points to source record).

### 9.2 Transform Execution Model

- Backend schedules transforms via a job runner.
- Transforms are idempotent and resumable.
- Re-transform policy is versioned:
  - changing OCR engine/model version creates a new derived stream version or marks derived records with transform version.

---

## 10. Query Language (Deterministic “nice syntax”)

v1 requires a query language that compiles deterministically to a typed query plan.

### 10.1 Requirements

- Must support:
  - stream selection,
  - boolean conditions,
  - time range restriction,
  - cross-stream correlation operators (WITHIN/OVERLAPS/DURING),
  - returning either:
    - records from a selected stream,
    - or replay sequences over a time window.
- Must be safe:
  - no arbitrary code execution,
  - bounded resource usage (timeouts/limits).

Implementation note (as of 2026-02-10):
- Backend query engine supports `WITHIN(...)`/`DURING(...)`/`OVERLAPS(...)` via a two-stage plan:
  - Phase 1: query source stream(s) for candidate intervals (`t_canonical`, `t_end`) matching the source predicate.
  - Phase 2: intersect all temporal-term interval sets (for conjunctions) and query the target stream for interval overlap (`t_canonical <= end AND t_end >= start`).
- Conjunctions may include multiple temporal terms (including multiple `WITHIN(...)`) and may mix `WITHIN` with `DURING`/`OVERLAPS`.
- Current limitation: temporal joins are only supported under conjunctions (`AND`), not under `OR`/`NOT`, and temporal operators are not supported inside temporal predicates.
- Interim API bridge: the backend accepts **LLQL JSON** embedded in `Query.text` via a `llql:` / `llql-json:` prefix, which is parsed into the typed AST and executed (enables cross-modal queries without a protobuf change).
- Interface support: the Timeline UI has an LLQL mode that submits `llql:` / `llql-json:` queries via `Query.text`.
- Replay now has a dedicated RPC (`Replay`) that returns ordered steps with aligned context keys, and the interface has a Replay view wired to it.

### 10.2 Canonical Example (Must Work)

Retrieve audio during times when:

- browser URL contains “youtube”
- OCR text contains “3Blue1Brown”

This must be expressible and must return:

- audio chunks (interval records) or audio segments clipped to correlated windows (defined explicitly by query).

Implementation note (as of 2026-02-10): verified end-to-end via LLQL JSON integration test (`server/tests/canonical_llql_example.rs`).

### 10.3 Replay Queries

Replay is a query mode that returns ordered steps:

- default step granularity: screen capture interval
- includes aligned context from other streams (keystrokes/clipboard/window/audio markers) within correlation window

Replay must be able to drive a UI that “walks forward” through time.

#### 10.3.1 Replay Semantics for Screen Point Records (Decision)

Screen capture frames are **point records** at times `t_i`. Replay interprets the frame at `t_i` as representing the half-open interval:

- `frame(t_i) := [t_i, t_{i+1})`

For the last frame in a replay window, the interval end is:

- `min(replay_window_end, t_i + capture_interval)`

---

## 11. Interface (Web UI served by backend)

The interface is read-only in v1 (browsing, search, replay). It is served by the backend and must work on:

- desktop browser (primary),
- phone browser (secondary).

### 11.1 Core UI Features

- Timeline navigation (jump by time, filter by modality/stream/device).
- Search box for query language + quick filters.
- Result list with previews:
  - screen thumbnails,
  - OCR snippets,
  - URL/title snippets,
  - clipboard/command text.
- Replay view:
  - step-by-step screen frames,
  - optionally audio playback aligned to frames,
  - visible aligned events (clipboard, key bursts, commands).

### 11.2 Query Cancellation

UI must be able to cancel in-flight queries/streams when:

- user issues a new query,
- user navigates away,
- backend signals overload.

---

## 12. Security and Privacy (v1 minimum viable)

v1 cannot be “perfect privacy”, but it must not be structurally unsafe.

### 12.1 Transport Security

- All collector <-> backend communication must be encrypted in transit (TLS).
- UI clients must also use TLS to connect to the backend.

### 12.2 Enrollment / Pairing

- Collector registration must require user-authorized pairing (exact mechanism TBD):
  - options: pre-shared token, QR code, mTLS cert issuance.

### 12.3 Minimal Safety Rails

Even if the product goal is “record everything”, v1 must support:

- emergency pause/resume capture,
- per-stream disable (config-driven),
- retention controls (at least coarse-grained).

Fine-grained filters (incognito/password field detection) are deferred.

### 12.4 Keystroke Capture Policy (Decision; High Risk)

v1 captures **full text keystrokes with minimal controls**.

This is explicitly high risk. Minimum required mitigations for v1:

- transport encryption (TLS) is mandatory,
- secure collector enrollment/pairing is mandatory,
- at-rest protection requirements must be defined before deployment (OS full-disk encryption at minimum; application-level encryption strongly recommended),
- global pause must be easy and reliable.

### 12.5 Retention Policy (Decision)

Default retention for raw screen/audio and other captured streams is **forever** (unless the user explicitly configures deletion).

---

## 13. Performance Requirements (Backend-focused)

### 13.1 Query Latency

- Return first results fast under recency-biased workloads (target SLA set by performance suite).
- v1 must include a performance test suite that measures:
  - ingestion throughput,
  - time-range scan performance,
  - text search latency,
  - replay assembly latency.

### 13.2 Ingestion Throughput

- Must support multi-gigabyte uploads via chunked streams.
- Must tolerate intermittent connectivity without data loss.

---

## 14. Observability and Operations

Backend must expose:

- collector connectivity status,
- backlog size per collector/stream,
- last successful pull time,
- ingest error rates,
- transform backlog and completion,
- storage usage (metadata vs blobs).

Collectors must expose:

- capture running status,
- buffer fullness,
- last capture timestamps,
- last successful upload ack.

---

## 15. v1 Milestones (Suggested sequencing)

1. Define time model + record schema + blob model.
2. Implement collector disk buffer + control channel + upload resume protocol.
3. Implement backend storage (metadata + blob) + durable ack semantics.
4. Implement OCR transform pipeline + text index.
5. Implement query language compiler (AST + planner) + baseline operators.
6. Implement web UI (timeline + search + replay).
7. Add multi-device robustness: clock skew handling + intermittent connectivity tests.

---

## 16. Artifact Reconciliation (Current Repository Reality)

This section merges in constraints and discrepancies discovered by re-reading the entire repository (docs, proto, modules). It exists to prevent “spec drift” during the rewrite.

### 16.1 What Exists Today (As-Built Snapshot)

- There is an implemented **gRPC backend** (`server/`) that:
  - registers collectors,
  - periodically pulls buffered data from collectors,
  - writes data into SurrealDB,
  - runs an OCR transform into derived tables,
  - serves Query/GetData/GetState style RPCs.
- There is an implemented **gRPC collector** (`collector/`) that:
  - runs local sources (currently screen + browser are wired),
  - streams buffered data to the backend on request,
  - reports its state.
- There is an **interface** (`interface/`) with two integration directions:
  - gRPC calls for system config (per `grpc-frontend.md` and `interface/src-tauri/src/main.rs`).
  - HTTP calls to `/api/loggers/...` endpoints assumed to exist at `http://localhost:8080` (per `server/README.md` and UI code).

### 16.2 Known Divergences vs This Spec

These are “must fix” to align implementation with this document:

1. **Query semantics are stubbed** in the current backend:
   - Query returns “all keys” and ignores the structured query message.
2. **Collector buffering is memory-backed** for implemented sources:
   - conflicts with v1 durable disk buffering requirement.
3. **Backend reachability assumes direct dialing of collector host:port**:
   - breaks for NAT/roaming devices; v1 requires a collector-initiated control channel.
4. **Proto coverage is incomplete for v1 modalities**:
   - payloads and modality enum currently cover only screen/browser/ocr.
5. **Proto correctness issues exist**:
   - some message definitions appear malformed/incomplete (e.g., data key shape); v1 requires a cleaned protocol contract.
6. **API surface is split (gRPC vs planned REST)**:
   - UI expects REST logger endpoints, while core backend is gRPC; v1 must choose and unify the interface-facing API surface (recommended: one backend that serves UI + collector transport).
7. **Blob storage is not separated** in the current backend:
   - screen frames store image bytes inline in DB rows; v1 requires metadata/blob separation.

### 16.3 “What Needs To Be Done” (Derived From Artifacts)

In addition to milestones in Section 15, the repo analysis implies these concrete tasks:

- Decide and implement the **single canonical backend API surface** for the UI:
  - either migrate UI fully to the gRPC backend, or implement an HTTP façade on the same backend process. Do not keep two unrelated backends.
- Redesign the collector-backend relationship for **NAT-safe backend authority**:
  - long-lived control channel initiated by collector; backend issues pull commands over it.
- Replace in-memory buffers with **disk-backed WAL/queue** in collectors for v1-required streams.
- Introduce a real **blob store** (content-addressed) and store only references in metadata.
- Fix and expand **proto/domain schema**:
  - add v1 modalities (audio, keystrokes, clipboard, shell, window/app activity),
  - correct malformed messages,
  - define stable ids (collector_id, stream_id, session_id, record_id/chunk ids).
- Replace transform “set-diff by full UUID scan” with an efficient **incremental transform cursor** model.
- Align configuration semantics:
  - eliminate any “config-as-JSON-string-in-proto” bridging; keep config typed end-to-end.
- **Refactor Type System (Proto-First)**:
  - Current state: Rust structs + macros generate `.proto` files; code manually converts between Rust domain types, Proto types, and Database types.
  - Target state: `.proto` files are the **single source of truth** for schema definitions.
  - Rust types should be generated from Proto (via `tonic`/`prost`), with `serde` derives enabled for config/DB compatibility.
  - Eliminate the "Rust-to-Proto" generation macro.
  - Remove manual type casting layers in Server/Collector where the types are isomorphic.

---

## 17. Open GitHub Issues (From `gh`)

This section is intended to be auto-populated from GitHub issues to ensure the rewrite plan accounts for tracked work.

### 17.1 Snapshot (Open Issues)

Pulled from `gh issue list --state open --limit 200` (user-provided output).

#### UX / Interface

- #79 `[FEATURE]: Have the interface have a "quick access mode"` (label: `feature`)
- #24 `browsing as the frontend`
- #23 `front end search`
- #9 `Improving the interface for search`

#### Reliability / Observability / Crash Handling

- #75 `[BUG]: crash` (label: `bug`)
- #74 `[FEATURE]: Have the collectors/server automatically inform us when there is a crash?` (label: `feature`)
- #72 `[Feature]: Create indicator per device that it is being logger/collector working` (label: `feature`)
- #71 `Create indicator per device that it is being logger/collector working` (duplicate of #72)
- #30 `add notifications for when the loggers fail`
- #7 `Should we ensure data integrity from logger (esp. when sending over network)`

#### Multi-Device / Networking / Distributed

- #70 `Problem with multiple device communication on different ip addresses`
- #36 `Align loggers on a given device and among all devices`
- #55 `instead of sending states, send diffs`
- #44 `Add backpressure like in vector` (labels: `server`, `logger`)
- #43 `Add buffers for sending data to the server` (labels: `server`, `logger`)
- #52 `lets implement intercetpors (future), call cancelation (future), and streaming (now)`
- #20 `Make storing data be a distributed file system`

#### Query / Search / NL-to-Query / Multimodal

- #12 `Taking natural language and converting it into database queries`
- #11 `Be able to query multimodal data (video, audio, and text)`
- #41 `add relevance feedback from user`
- #26 `how to extract information from the data modalities`

#### Data Modalities / Collection Coverage

- #15 `add system audio as a data source`
- #14 `Add clipboard history as a data source`
- #13 `Add CLI history as a data source`
- #37 `image processing from screen`
- #6 `Create a module that can access other software (such as activity watch)`

#### Storage / Compression / Backups / Hot-Cold

- #54 `compression`
- #18 `For each data modalitiy add in lossless compression`
- #16 `automatic backups`
- #40 `make a 'hot' and 'cold' database? for users`
- #10 `Use a different DB`

#### Configuration / Policy / Modularity

- #63 `Hotswap of policy??`
- #42 `allow hotreload for config from anywhere` (labels: `server`, `logger`)
- #38 `how to abstract making modules`
- #49 `Automatically create new data sources that are "from-the-same-family"`
- #50 `Create a rust macro to automatically implement & modify logger functions`
- #45 `Refactor the config to use confy package`
- #47 `use this as a possible transform?`
- #58 `use this as a possible transform?` (ambiguous; likely transform-related)

#### Platform / Install / DevEx / Cleanup

- #8 `Port the software onto Windows and Mac`
- #22 `make a \`lifelog install\` command`
- #68 `Remove not used dependencies, clean up the Cargo.toml's`
- #4 `Consider using mold for compilation`
- #5 `Ensure that this program tells the user they need access to the input group.`
- #21 `change rscam so it works on linux`

#### Data Quality / Bugs

- #28 `processes doesn't align with btop`
- #27 `check this out for tesseract`
- #46 `Use this library for parsing user uploaded docs`

#### Ecosystem / Extensibility (Agents / Apps / Sinks)

- #69 `Be able to view the return of any data modality in a custom application`
- #57 `create an api so other applications can use this log`
- #56 `Sink: summarization of data`
- #39 `add anootation capabilities to data`
- #51 `another feature that we could add is manual event adding, so the user can specify "between START_OF_SCHOOL_YEAR and END_OF_SCHOOL_YEAR"`

### 17.2 Spec Mapping Notes

- Issues #43/#44/#55/#70/#36/#7 map directly to v1 invariants in Sections 5–7 (control/data plane, buffering, backpressure, integrity, multi-device).
- Issues #12/#11/#23/#9/#79 map directly to Sections 10–11 (query language + UI).
- Issues #14/#13/#15 map directly to Section 3 (v1 modalities) and require expanding the typed payload/stream model.

---

## 18. Decision Log (User Selections)

This section records decisions you’ve made so far so they cannot silently drift.

### 18.1 Time and Replay

- Canonical timeline time: **device-reported time corrected by estimated skew** (Section 4.0).
- Screen records: **point records**; replay maps `t_i` to `[t_i, t_{i+1})` (Section 10.3.1).

### 18.2 Data Plane

- Upload resume offset unit: **byte offset within a per-stream session file** (Section 7.3.1).
- Per-stream monotonic ordering: **required sequence numbers** exist for ordering and dedupe, but are not the upload offset (Section 4.2, Section 7.3.1).

### 18.3 Delivery Semantics

- ACK implies **fully queryable** (baseline indexes updated), accepting ingest backpressure (Section 6.2.1).

### 18.4 API Surface

- **gRPC-only** for UI and collectors (Section 5.4).

### 18.5 Storage

- Hot store DB: **SurrealDB** (Section 8.0).
- Blob store: **filesystem content-addressed store (CAS)** (Section 8.2.1).
- Cold tier (future): **hybrid hot/cold** with optional Parquet export later (Section 8.2.1).

### 18.6 Capture and Retention

- Keystrokes: **full text capture with minimal controls** (Section 12.4). (High risk acknowledged.)
- Default retention: **forever** (Section 12.5).

### 18.7 Query Authoring (Ambiguous; Needs Confirmation)

- Templates + builder first, DSL as advanced view

---

## 19. Consolidated Analysis (From `*_ANALYSIS.md`)

This section incorporates the major findings from the generated reviews. It is included so engineering tradeoffs stay explicit; normative requirements remain in Sections 1–15.

### 19.1 Architecture & Semantics (from `SYSTEMS_ARCHITECT_ANALYSIS.md`)

- Replay semantics must be explicitly defined for point-based frames (resolved in Section 10.3.1).
- Cross-device time semantics required canonical “timeline time” and skew handling (resolved in Section 4.0/4.2.1).
- Chunk framing/offset definition was missing (resolved for offset unit in Section 7.3.1). TODO: Identify what is a good practical maximum chunk size, best hashing algorithm for these goals, and canonicalization rules.
- ACK coupled to indexing is a strong coupling; you chose “fully queryable ACK” (Section 6.2.1). This implies ingestion backpressure must be engineered intentionally.
- Backend-pull vs collector-push must be explicit: collectors push bulk data only within backend-authorized sessions (Section 5.2 + 7).
- Keystroke capture policy is a security/product blocker (chosen, high-risk; Section 12.4).
- Security requirements must be concrete for multi-device operation (expanded in Section 12 and reinforced below).
- Performance requirements must be numeric (still **OPEN**; Section 13 needs concrete targets).

### 19.2 Data Layer (from `SENIOR_DATA_ENGINEER_ANALYSIS.md`)

- Current repo query engine is stubbed (returns all keys). v1 requires real query planning and indexes.
- Transform model currently full-scans and does in-memory set diffs; v1 requires incremental transform cursors/checkpoints.
- SurrealDB can work but must be used with:
  - explicit time/text indexes,
  - stable catalog (not `INFO FOR DB`),
  - blob separation.
- Consider hot/cold tiering: Parquet + DuckDB is a strong cold-tier option later (aligned with your hybrid decision).

### 19.3 Rust/Concurrency/Performance (from `ELISE_TANAKA_RUST_REVIEW_ANALYSIS.md`)

Implementation requirements derived from current pitfalls:

- No `static mut` globals in async code; all shared state must be synchronized safely.
- Do not hold global locks across `.await`; adopt an actor model or strict lock discipline.
- Avoid `Any` downcasting for core paths; use an object-safe “erased source” trait.
- Stream ingest end-to-end:
  - collectors drain buffers without cloning,
  - server ingests each streamed item incrementally (do not collect all then insert).
- Background tasks must have real lifecycle handles; joins and failure propagation must be explicit.

### 19.4 Operational Readiness (from `CARLOS_MENDEZ_ANALYSIS.md`)

To satisfy “passive capture, zero maintenance”:

- Provide an installation + service management story (systemd/launchd at minimum).
- Fix config bootstrapping: first run must not panic if config is missing.
- Implement NAT-safe connectivity: collector-initiated persistent control channel (already required by Section 5.3).
- Add monitoring/quiet alerts: durable “last seen”, capture gaps, backlog growth, storage low.
- Define resource budgets (CPU/mem/disk/network) and enforce bounded buffers.
- Decide update mechanism; at minimum, version reporting and drift detection.

### 19.5 ML/Multimodal Platform (from `OMAR_HASSAN_ANALYSIS.md`)

- Spec is conceptually ML-ready if implemented (streams + transforms + lineage).
- Current modality coverage is incomplete; v1 modalities must be represented in the typed model.
- Audio must become first-class in the canonical data plane (proto + storage), not only local wav files.
- OCR should eventually store richer artifacts (confidence/bboxes) for better UI and downstream reasoning (optional v1, valuable v2).
- Embeddings and vector index should be versioned and reproducible when added.

### 19.6 Security & Privacy Launch Audit (from `KWAME_ASANTE_ANALYSIS.md`)

Launch-blocking requirements implied by the audit:

- TLS everywhere (collector/backend and UI/backend).
- Secure pairing/enrollment (mTLS recommended); device identity must be cryptographic, not MAC-based.
- Remove hard-coded DB root credentials; use secrets management.
- Authentication + authorization on all query/config/data APIs.
- Disable gRPC reflection in production (or gate behind auth).
- At-rest protections for highly sensitive capture (keystrokes/audio/screen); OS full-disk encryption minimum, app-level encryption strongly recommended.

### 19.7 UX (from `AVA_CHEN_UX_REVIEW_ANALYSIS.md`)

- Current UI mental model is “logger dashboards” but v1 promise is recall (timeline + search + replay).
- Query authoring should use progressive disclosure (simple search -> templates -> advanced DSL), even if DSL is the canonical representation.
- Multi-device must be first-class in UI: device health, backlogs, and “what’s collecting now” must be visible.
- Privacy must be a first-class surface: pause, per-stream toggles, retention, deletion by time range.
