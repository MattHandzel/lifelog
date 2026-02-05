# Lifelog Specification Recompilation (From Repository Artifacts)

This document re-derives (“recompiles”) the project specification from what exists in this repository today:

- `docs/` design notes
- protobuf definitions in `proto/`
- Rust crates under `server/`, `collector/`, `common/`, and `interface/`
- top-level planning docs (`README.md`, `refactoring-plan.md`, `grpc-frontend.md`, etc.)

It is intentionally split into:

1. **Artifact-Derived Intent**: what the project is trying to be (from docs + scaffolding).
2. **As-Built Behavior**: what the code currently does.
3. **Gaps / Contradictions**: where artifacts disagree or behavior diverges.
4. **Recompiled v1 Requirements**: concrete requirements that follow from the artifacts and your stated goals.

This is a “source of truth” for a rewrite: it explains which parts are real constraints vs aspirational notes.

---

## 1. Artifact-Derived Intent (What This System Is)

### 1.1 System Purpose

From `docs/vision.md`, `docs/concepts*.md`, and `README.md`, the intended system is:

- A **local-first lifelog** that “stores everything” from many modalities.
- A **client-server architecture** where each device captures data and a central backend stores and transforms it.
- A **query + interface layer** that allows the user to retrieve and view their history.
- A platform where “agents/sinks” can eventually consume derived data under strict permissions, but the core is first-party recall and retrieval.

### 1.2 Architectural Roles

Artifacts consistently name three components (with some naming drift):

- **Collectors**: device programs capturing streams (screen, browser, audio, etc.), buffering, and responding to server requests.
- **Server/Backend**: central authority storing data, running transforms (e.g., OCR), processing queries, managing collectors.
- **Interface**: user UI for browsing/querying; in this repo it is a Tauri app plus a web frontend.

### 1.3 Core Principles (Repeated Across Docs)

From `docs/concepts*.md`, `docs/buffering.md`, `docs/acknowledging-data_written.md`, `docs/policy.md`, `docs/server.md`:

- Store everything; don’t lose data.
- Strong typing for data and APIs.
- Fault tolerance (buffering, resumable transfer, idempotency, end-to-end acknowledgement).
- Extensibility: add modalities/transforms without rewriting the whole system.
- Security-first: encrypted transport, explicit permissions, secure enrollment.
- Policy-driven operation: server decides when to sync, transform, compress, backup.

### 1.4 Intended Query Model (From Docs)

From `docs/querying.md`, `docs/search.md`, and examples in docs:

- Multiple query classes: structured predicates, text matching, vector similarity, multimodal.
- Query as boolean composition of subqueries.
- Important implied capability: **cross-modal correlation** in time (using one modality to locate another).

---

## 2. Workspace and Modules (What Exists in Code)

### 2.1 Rust Workspace Layout (Actual)

From top-level `Cargo.toml`, the workspace crates are:

- `server/` (`lifelog-server`) – tonic gRPC backend + SurrealDB storage + transforms.
- `collector/` (`lifelog-collector`) – tonic gRPC collector with local sources (currently screen + browser wired).
- `interface/src-tauri/` (`lifelog-interface`) – Tauri desktop app, includes:
  - a webview UI,
  - an HTTP client to `/api/loggers/...` endpoints,
  - a gRPC client to the backend for system config (per `grpc-frontend.md`).
- `common/` shared crates:
  - `common/lifelog-proto` – generated proto bindings.
  - `common/lifelog-types` – domain types (origins, keys, modalities).
  - `common/data-modalities` – modality structs + transforms (OCR via Tesseract).
  - `common/config` – TOML config models + loader.
  - `common/lifelog-core`, `common/utils`, `common/macros` – shared utilities/traits.

### 2.2 Entry Points (Actual)

- Backend: `server/src/main.rs` starts the gRPC server and also spawns a background “policy loop”.
- Collector: `collector/src/main.rs` starts local capture sources and serves a CollectorService gRPC server.
- Interface:
  - Main Tauri app: `interface/src-tauri/src/main.rs`
  - Additional bins exist under `interface/src-tauri/src/bin/` but at least one is marked deprecated.

---

## 3. Protocol Specification (From `proto/`)

The repository defines two gRPC services:

### 3.1 Backend Service (Server implements)

`service LifelogServerService` in `proto/lifelog.proto`:

- `RegisterCollector`
- `GetConfig`
- `SetConfig`
- `Query`
- `GetData`
- `ReportState`
- `GetState`

Key message families:

- System config and state are typed (`SystemConfig`, `SystemState`) via `lifelog_types.proto`.
- Query is a structured message (`Query`) with:
  - `search_origins`, `return_origins`
  - `time_ranges`
  - optional embeddings
  - repeated text strings

**Intended**: query returns keys; client can then call GetData with keys (two-step retrieval).

### 3.2 Collector Service (Collector implements)

`service CollectorService` in `proto/lifelog.proto`:

- `GetState`
- `GetConfig`
- `SetConfig`
- `GetData` (server-streaming, returns `stream LifelogData`)

**Intended**: backend “pulls” data by calling `GetData` on collectors.

### 3.3 Modality Payloads (Current proto coverage)

`proto/lifelog_types.proto` defines:

- `ScreenFrame` includes `bytes image_bytes` inline.
- `BrowserFrame`, `OcrFrame`.
- `LifelogData` is a `oneof` of these.
- `DataModality` enum currently includes Browser/Ocr/Screen only.

**Implication**: the current proto and domain model do not yet cover microphone/audio, keystrokes, clipboard, shell history, etc., although configs exist for several modalities.

---

## 4. As-Built Behavior (What the code does today)

This is the most important “recompilation” for preventing rework: v1 rewrite should not accidentally preserve wrong behavior.

### 4.1 Backend (gRPC + SurrealDB)

From `server/src/server.rs` and `server/src/main.rs`:

- The backend connects to SurrealDB over websocket, signs in as root, and uses a namespace/db.
- It maintains:
  - a system state snapshot (CPU/memory usage, last sync time, pending actions),
  - a list of registered collectors (with gRPC clients back to them),
  - a list of transforms (currently OCR from screen).
- It runs a background loop that:
  - periodically chooses an action (sync vs transform vs sleep),
  - executes sync by calling `CollectorService.GetData` on each registered collector and inserting results into SurrealDB,
  - executes transform by scanning for source ids missing in destination tables and running OCR.

**Important behaviors**:

- **Query RPC currently does not implement query semantics**. It returns “all keys from all origins/tables” and ignores the query object.
- **GetData does per-key DB select** and reconstructs the UUID into returned records (the DB row format doesn’t round-trip it cleanly; code patches it in).
- **Transforms** are stored as new rows in new tables per origin/modality, and use a set-diff of UUIDs to find work.
- **Schema** is enforced with “ensure table exists” using DDL templates per modality (screen/browser/ocr).
- Screen image bytes are stored directly in SurrealDB today (no blob store separation).

### 4.2 Collector (device-side capture + gRPC server)

From `collector/src/collector.rs`:

- Collector manages enabled local sources (currently wired: screen, browser history).
- It contains an outbound gRPC client to the backend (handshake registration) and also exposes gRPC endpoints as a server.
- On `CollectorService.GetData`, it:
  - reads buffered screen frames, clears the buffer, streams them to caller,
  - reads browser history frames and streams them (buffer clearing differs by source).

**Important behaviors**:

- Collector identity uses MAC address normalization.
- Collector buffering in this code path appears **memory-backed**, not disk-backed.
- The GetData request’s keys are effectively ignored; collector sends “whatever is currently buffered”.

### 4.3 Interface (Tauri + Web, plus two different backend notions)

Artifacts reveal two parallel interface-to-backend mechanisms:

1. **gRPC to lifelog backend**:
   - `grpc-frontend.md` describes and `interface/src-tauri/src/main.rs` implements gRPC calls to `GetConfig` and `SetConfig` to retrieve/modify collector config values.

2. **HTTP REST calls to `/api/loggers/...`**:
   - `interface/src-tauri/src/main.rs` heavily uses `reqwest` to call endpoints like `/api/loggers/screen/data`, `/api/loggers/microphone/config`, `/api/loggers/text/upload`, etc.
   - There is a `server/README.md` describing a REST server with JWT auth for these endpoints.
   - However, the Rust backend under `server/src/` is gRPC+SurrealDB and does not implement this REST API.

**Recompiled conclusion**:

- The repo contains two architectural directions:
  - a gRPC “lifelog backend” (collector sync + SurrealDB + transforms),
  - an intended REST “logger management server” that fronts local loggers for the UI.
- The REST server appears to exist only as a plan/README; the Tauri app assumes it exists at `http://localhost:8080` by default.

---

## 5. Artifact Gaps and Contradictions (Where a rewrite must choose)

### 5.1 Buffering and Acknowledgement

Docs emphasize durable buffering and end-to-end acknowledgements.

As-built:
- collectors buffer in memory (for implemented sources),
- collector streams “buffer contents then clears”.

This is incompatible with “don’t lose anything” under collector crash/reboot.

### 5.2 “Backend Pull” vs NAT/Reachability

Proto and server code assume backend can connect to collector gRPC endpoint.

As-built:
- collector registers by telling backend its host/port; backend dials it.

In real multi-device environments (phone, roaming laptop) this fails without:
- collectors dialing outbound to backend and maintaining a control channel, or
- some relay/tunnel.

### 5.3 Query Semantics

Proto defines a structured query. Docs discuss classes of query and cross-modal retrieval.

As-built:
- Query returns all keys and ignores query.

### 5.4 Storage Model

Docs question whether blobs should live in object storage vs DB.

As-built:
- inline bytes stored in SurrealDB rows (screen images).

### 5.5 Interface Contract

Docs and Tauri code assume:
- UI can read data and also configure components.

As-built:
- there is no single backend that cleanly serves both:
  - “lifelog retrieval (gRPC/SurrealDB)” and
  - “logger operations (REST/JWT)”.

This is a major design choice for the rewrite: unify control + data + retrieval behind one backend surface.

---

## 6. Recompiled Specifications (Concrete requirements implied by artifacts)

This section is the “compile output”: requirements that follow from the artifact set + the direction you stated earlier.

### 6.1 System Boundaries

- There is exactly one “central backend authority” per lifelog system.
- Collectors are replaceable and must not depend on backend internal storage layout.
- UI clients (desktop + phone) are stateless consumers of backend APIs and can be served as web UI by the backend.

### 6.2 Control Plane Requirements

- Secure enrollment/pairing for collectors (do not accept arbitrary registrations).
- Config distribution to collectors (Get/Set must be coherent and not rely on ad-hoc JSON encoding).
- Collector health reporting (buffer sizes, capture status, last sync).
- Backend scheduler/policy decides when to ingest and when to transform.

### 6.3 Data Plane Requirements

- Chunked, offset-based, resumable transfer for large blobs.
- Durable ack semantics (ack means persisted).
- Idempotency keys for retry safety.
- Collectors must implement disk-backed buffers for v1 critical modalities.

### 6.4 Storage Requirements

- Separate metadata from blobs (object store / file store) to avoid storing multi-GB inline in a transactional DB.
- Maintain lineage between raw and derived streams (OCR etc).
- Efficient time-range scans as primary access path.
- Text index for OCR/browser/clipboard/shell history (v1).

### 6.5 Query Requirements

- Deterministic query language that compiles to a typed query plan.
- Cross-stream correlation semantics with explicit time windows and interval overlap operators.
- Two-phase retrieval (keys then fetch) is acceptable, but must support streaming and cancellation for UI.

### 6.6 Transform Requirements

- OCR as first-class transform pipeline, scheduled and idempotent.
- Transform “missing work” detection must be efficient (avoid full table scans and in-memory set diffs at scale).

---

## 7. What to Treat as Authoritative vs Aspirational

### 7.1 Authoritative (Most binding)

- `docs/vision.md`, `docs/concepts*.md`, `docs/server.md`, `docs/collector.md`, `docs/policy.md`
- `proto/*.proto` (as the intended typed contract, though it needs fixes/expansion)
- The working gRPC backend + collector behavior in code (as “what exists today”)

### 7.2 Aspirational / Research Notes (Less binding)

- `server-distributed-file-system.md`, `server-encryption-resource.md`, `docs/tcp-optimizations.md`, large parts of `docs/protobuf.md`
  - These are useful reference material but not an implemented plan.

---

## 8. Immediate Spec Fixes Suggested by This Recompilation

If you keep the current proto-based direction, the minimum spec corrections are:

1. Expand `DataModality` and `LifelogData` payloads to include the v1-required modalities (audio, keystrokes, clipboard, shell history, window/app activity).
2. Fix the query/key message definitions (some proto fields appear malformed/incomplete) and define key identity cleanly.
3. Decide whether the “REST logger server” is:
   - deprecated (fold into gRPC backend), or
   - the canonical “UI backend”, with gRPC only for collector transport.
4. Replace “backend dials collectors by host/port” with a NAT-safe control channel strategy for multi-device.
5. Define durable buffering and durable ack semantics explicitly and implement them end-to-end.

