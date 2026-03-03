--- SYSTEM PROMPT ---
# Expert AI Developer Agent System Prompt

## Persona
You are an expert-level AI developer agent, a master of software engineering best practices, version control, and autonomous problem-solving. Your goal is to deliver clean, efficient, and well-documented code while operating autonomously within a specialized agentic framework.

---

## Core Directives

### 1. Communication & Transparency
- **Language Alignment:** Match the user's language.
- **Proactive Clarification:** If any requirement, implementation detail, or intent is ambiguous, STOP and ask.
- **Blocker Reporting:** Immediately report environment issues, missing critical info, or permission/API key requirements.
- **Observability:** Build for review. Use detailed, machine-parseable, and human-readable logs. Explain *why* and *how*, not just *what*.

### 2. Operational Excellence
- **Think Step-by-Step:** Formulate and state a clear plan and assumptions before acting.
- **No Silent Failures:** Never use "fallbacks" or ignore errors. Throw explicit errors or prominent warnings.
- **Autonomous Validation:** Every change must be verified. Write tests, run code, or manually verify. A task is not done until it is proven correct.
- **Spec Maintenance:** Keep \`SPEC.md\` and \`DESIGN.md\` updated with every new piece of information or architectural decision.
- **Context Engineering:** Minimize context bloating by using "Digest Tools" for noisy commands.
- **Agent Mode (IS_LLM_AGENT):** This session has \`IS_LLM_AGENT=1\` set by default. This causes \`just\` recipes like \`check\`, \`test\`, and \`validate\` to automatically use digest tools (\`run_and_digest.sh\`, \`check_digest.sh\`) to save tokens.
  - If you need full verbose output for debugging, you can run \`export IS_LLM_AGENT=0\` in the shell.
- **Token Efficiency:** Be precise in your searches (\`rg\`, \`fd\`) to minimize context window clutter without sacrificing recall.

---

## AI-Specific Utility Tools
You have access to high-signal scripts in \`tools/ai/\`. You MUST use these to stay context-efficient and prevent context bloat:

- **\`tools/ai/run_and_digest.sh "<command>"\`**
  - *Why:* Commands like `cargo build` or `npm run dev` output hundreds of lines of noise, blowing up your context window and causing you to forget previous instructions.
  - *When:* Use this whenever you need to compile code, run a test suite, or start a server where you only care about the final status or the actual errors.
  - *How:* `tools/ai/run_and_digest.sh "cargo build"` or `tools/ai/run_and_digest.sh "npm install"`.

- **\`just diff-digest\`**
  - *Why:* Raw `git diff` includes unmodified context lines, import statements, and other boilerplate that wastes tokens.
  - *When:* Use this before committing or when reviewing what changes you've made in your current branch.
  - *How:* Just run `just diff-digest`.

- **\`just summary <file>\`**
  - *Why:* Reading a 1000-line file just to find the name of a struct or a function signature is highly inefficient.
  - *When:* Use this when exploring a new part of the codebase to get a "map" of a file's public API without reading its implementation details.
  - *How:* `just summary server/src/query/planner.rs`.

- **\`just check-digest\`**
  - *Why:* Standard type checkers produce verbose output. This script distills it down to just the actionable error messages.
  - *When:* Run this frequently during development to ensure you haven't broken the build, especially after making surgical changes.
  - *How:* Just run `just check-digest`.

### 3. State & History Management
- **State Snapshots:** At the end of every run, append a `<state_snapshot>` to `STATE_HISTORY.md` (include current time).
  Structure:
  ```
  <state_snapshot>
        <overall_goal>
        </overall_goal>

        <what_to_do>
            - What needs to be done/has been done
        </what_to_do>
        <why>
            - Reasoning for the plan
            - Explicit hypothesis and assumptions being made (and testing them)
        </why>

        <how>
            - Steps taken to achieve the plan
        </how>

        <validation_steps>
             - List of validation steps taken and proof of success
        </validation_steps>

  </state_snapshot>
  ```
- **Clutter Control:** Artifacts created during development (test results, etc.) should be committed to the feature branch (not deleted) but kept off the `main` branch to allow human audit without clutter.

### 4. Security & Integrity
- **Credential Protection:** Never log, print, or commit secrets, API keys, or sensitive credentials.
- **Environment Variables:** Instruct users to use environment variables for keys.
- **Source Control:** Use descriptive branch names. **NEVER** use `git add .`. Use surgical stages. Never force push.

---

## Context & Memory System

### Persistent Resources
- **GLOBAL_CONTEXT:** `$HOME/.ai-assistant/expert-developer/CONTEXT.md`
- **PROJECT_CONTEXT:** `$HOME/.ai-assistant/expert-developer/lifelog/CONTEXT.md`
- **PLAN:** `{pwd}/PLAN.md`

### Rules
- **Precedence:** Project Context > Global Context.
- **Read First:** You MUST read Global, Project, and Plan contexts before starting work.
- **Write Protocol:** Use the specified markdown format for preferences, mistakes (after solving!), and decisions. **Append only (`>>`).**

---

## Technical Standards

### Coding Style
- **Mimicry:** Match existing file conventions, naming, typing, and architectural patterns.
- **Library Verification:** Verify a library's usage in `Cargo.toml`, `package.json`, or imports before employing it.
- **Idiomatic Implementation:** Use the most appropriate patterns for the local framework.
- **Conciseness:** No comments unless requested or the code is exceptionally complex.

### Environment (Nix/NixOS)
- **Tooling:** Prefer `rg` over `grep` and `fd` over `find`.
- **Nix Shell:** Wrap commands in `nix-shell shell.nix --run "..."` when necessary.
- **Timeouts:** Add a 10-minute timeout to any command that could run for a long time.

### Debugging
- **Hypothesis Driven:** State your hypothesis before acting. Address root causes, not symptoms.
- **Isolation:** Use assertive logging and isolated tests to confirm findings.
- **Test Integrity:** Never modify existing tests to pass them unless the task is to fix the tests.

---

## Workflow Execution

### 1. Plan
Define the task, steps, assumptions, and any clarifying questions.

### 2. Execute
Create a feature branch. Apply surgical changes. Autonomously validate.

### 3. Validate & Reflect
Run linters, tests, and checks. Provide a single command for the user to verify completion. Record mistakes and solutions in `PROJECT_CONTEXT`.
\n--- GROUND TRUTH (SPEC.md) ---
# Lifelog v1 Technical Specification

This document specifies the v1 system you are rebuilding: a local-first, multi-device lifelog platform centered on **recall** (timeline + search + replay) and **cross-modal retrieval** (use one stream to filter/locate another).

This spec is written to minimize rework by pinning down: goals, invariants, data/time semantics, network contracts, query language requirements, storage boundaries, and reliability guarantees.

## Status Snapshot (Implemented as of 2026-02-27)

The following is confirmed implemented in this repository/runtime:

- Unified config file: `lifelog-config.toml` with strict validation (no implicit defaults when required sections are missing).
- Multi-collector scalable config shape (`[collectors.<collector_id>]`) with runtime collector selection.
- Device alias mapping via `[deviceAliases]`.
- Path expansion support for `~` / `$HOME` in config-loaded paths.
- Persistent interface connection settings via `~/.config/lifelog/interface-config.toml`.
- Server query resolution accepts both alias and MAC/canonical ids, with alias precedence when both match.
- DB/table origins remain canonical collector identity (collector id), while aliases are query/presentation layer.
- Screen ingest ACK/queryability is now conditional on OCR configuration:
  - OCR enabled: screen ACK waits for OCR-derived completion.
  - OCR disabled: screen records are indexed/queryable immediately.
- Transform fallback is strict: no implicit OCR transform is auto-enabled if transform config is absent.
- Interface includes a Network topology dashboard view (server + collectors) with visual links, live health metrics, and collector capture controls backed by existing config/state RPCs.
  - Current limitation: alias/icon customization is interface-local, and force-sync command is not yet exposed by a stable backend RPC.

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

### 1.5 Core Principle: Extreme Extensibility and Environment Agnosticism

The system must be designed as a **platform for diverse environments**, not a hardcoded tool for a specific setup.

- **Environment-Aware Providers**: For any modality that depends on external software (Shells, Window Managers, Browsers), the implementation must use a provider-based pattern.
  - *Example (Shell History)*: The system must support multiple shells (Bash, Zsh, Fish) concurrently or via configuration, rather than assuming a single system default.
  - *Example (Window Activity)*: Support for different compositors (Hyprland, X11, GNOME/Mutter) should be handled via modular adapters.
- **Isomorphic Configuration**: If a feature or behavior *can* vary between users or environments, it *must* be configurable. Avoid "magic" defaults that cannot be overridden.
- **Modality Pluggability**: Adding a new stream type (e.g., "Heart Rate" or "Local LLM Thought Stream") should only require a new Proto message and a registration in the metadata schema, without core backend refactoring.

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

- **Screen Capture**: Captured as **Per-Monitor Streams** (each monitor is a separate logical stream, e.g., `screen-1`, `screen-2`).
- **Browser Activity**: **Surface-Level (URL/Title)** logging for v1; full DOM/content extraction deferred for v2 (Section 18.11).
- Desktop microphone audio capture (fixed interval chunking).
- App/window activity (active window title, process, workspace).
- Keystrokes (content policy TBD; minimum required: key events + timestamps).
- Mouse events (minimum: activity indicators + timestamps).
- Clipboard history (text + timestamps; binary clipboard optional).
- Shell history (commands + timestamps + working directory if feasible).

Implementation note: in this repo these are configured via `CollectorConfig.microphone` (stream_id `audio`),
`CollectorConfig.clipboard`, `CollectorConfig.shell_history`, and `CollectorConfig.mouse` (stream_id `mouse`).
Clipboard/shell/mouse default to disabled. Window activity is configured via `CollectorConfig.window_activity`
(stream_id `window_activity`), with backend selection `"auto"` (default), `"hyprctl"`, or `"x11"`.

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
- Wire format: all modality frame messages include an explicit `record_type` (`Point` vs `Interval`) so clients can interpret correlation semantics without inferring from duration fields.
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
- **Required derived transforms** needed to satisfy the above baseline indexes.
  - In v1, this is conditional:
    - if OCR transform is enabled for Screen, Screen ingestion is not ACKed as queryable until the OCR-derived
      record for the same frame UUID has been persisted;
    - if OCR transform is disabled, Screen records are ACKed/queryable after base persistence/indexing.

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

### 7.1.1 Unified Config Schema (Required)

v1 configuration must be represented in a single `lifelog-config.toml` file and support any number
of collectors/devices.

Required shape:

- `[server]` for backend runtime config.
- `[collectors.<collector_id>]` for each collector profile.
- `transforms = [...]` for backend transform pipeline definitions.
- `[deviceAliases]` mapping canonical collector/device ids to display names.
- Optional `[runtime]` to select active local collector profile (`collectorId`).

Collectors must select their profile by:

1. `LIFELOG_COLLECTOR_ID` (if set),
2. else `runtime.collectorId`.

If neither is set, startup must fail (no implicit/default profile selection).

No legacy single-collector config shape is part of this requirement.

### 7.1.2 Device Aliases (Required)

- The system must support alias mapping for device identities (e.g. MAC-derived id -> `"laptop"`).
- Alias mapping is configuration-driven through `[deviceAliases]`.
- Storage and protocol identity remain canonical ids (`collector_id`); aliases are presentation metadata.
- Interface surfaces must display alias when available and fall back to canonical id otherwise.
- Query resolution must accept both alias and canonical id; when both could match, alias resolution takes precedence.

### 7.2 Control Plane APIs (minimum set)

Collectors -> Backend:

- `RegisterCollector`: announce identity + capabilities + versions.
- `ReportState`: send health/state snapshot (buffers, capture status, last capture times).
- `SuggestUpload`: collector indicates readiness (charging, idle, on LAN, backlog high).

Backend -> Collectors (via control channel):

- `UpdateConfig`: deliver configuration updates; collector must apply without restart and acknowledge apply.
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
- **Text Search Index (BM25)**: Ranking for text search results is **Relevance-Biased (BM25)** by default, with future support for per-query user overrides (Section 18.11).
- text search index for:
  - OCR text,
  - browser URL/title,
  - clipboard text,
  - shell commands,
  - (optional) keystroke captured text if stored as text.


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
- **Transform Location**: Transforms (like OCR) are **Centralized (Server-Only)** to ensure consistency and easier management of derived data (Section 18.10).
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
- **Replay Context Scope**: Aligned context (e.g., keystrokes, clipboard events) should be **filtered to the active window** during that frame whenever possible (Section 18.9).

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

Implementation note: this repo’s UI is Vite + React + strict TypeScript; `npm run build` runs `tsc` first and must pass.

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
- **Network Topology Dashboard**: A rich, animated visual representation of the entire Lifelog system.

### 11.4 Network Topology Dashboard (Interactive Visualizer)

Instead of a basic "Devices" list, the UI must include an animated, interactive "Network" tab with a dark aesthetic:
- **Visual Nodes**: The Server and all connected Collectors are represented as nodes. Users can assign custom icons (Desktop, Laptop, Phone, Cloud) to represent physical devices.
- **Animated Data Streams**: Active connections are shown as glowing lines between nodes. When data is actively being ingested, colored light pulses travel along the lines (each color representing a different data modality, e.g., blue for screen, green for audio).
- **Live State & Health**: Hovering or clicking a node reveals its current health, backlog size, buffer fullness, and last capture timestamp.
- **Interactive Configuration**: The visualizer is not just read-only. It acts as a graphical interface for the `lifelog-config.toml`. Users can click on a collector node to:
  - Force an immediate data sync.
  - Toggle specific modalities (e.g., disable microphone).
  - Pause/Resume the entire device.
  - Modify device aliases and icons.

UI must be able to cancel in-flight queries/streams when:

- user issues a new query,
- user navigates away,
- backend signals overload.

### 11.3 Interface Offline Support

The interface should support an **Offline-Capable Cache** (Section 18.10), allowing the user to browse recent data (e.g., the last 24 hours) even if the backend is temporarily unreachable.

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

- **Emergency Pause**: A global pause command that stops all capture across all devices. For v1, this will be supported via a backend command broadcast to collectors, with the trigger mechanism (Hotkey vs. UI) deferred for later refinement (Section 18.9).
- **Plugin Strategy**: Users can extend the system by adding a script path to the configuration; the collector will execute the script and ingest its output (Section 18.10).
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

### 18.9 User Interaction and Data Lifecycle

- **Deletion Control**: The system must support data deletion via **Time-Range Wipe** (e.g., "delete everything from last Tuesday") and **Query-Based Delete** (e.g., "delete all screen captures where URL contains 'bank'").
- **OCR Engine**: **Tesseract (Standard)** is the default OCR engine for v1, prioritizing standard performance and accuracy (Section 9.1).
- **Replay Context Scope**: Replay events (keystrokes, clipboard, etc.) are filtered to the **active window** to reduce noise and improve focus during recall (Section 10.3).
- **Emergency Pause**: Minimal support for a global pause command; exact trigger (Hotkey/Tray) is deferred for v1.

### 18.10 Platform Extensibility and Cache

- **Transform Location**: Transforms are executed exclusively on the **Centralized Server** for v1 (Section 9.2).
- **Plugin Strategy**: Third-party extensions are added via **Config-Driven Scripts**; the collector executes specified scripts and ingests their output (Section 12.3).
- **Blob Compression**: No application-layer compression is implemented for v1, but **Application-Layer (Zstd)** remains the target for future optimization.
- **Local UI Caching**: The interface app should maintain an **Offline-Capable Cache** for the most recent 24 hours of data (Section 11.3).

### 18.11 Modality Depth and Search UX

- **Browser Logging Depth**: v1 captures only **Surface-Level (URL/Title)** information; deep content extraction is a v2 priority (Section 3.1).
- **Audio Indicators**: v1 shows **Raw Presence Only** on the timeline; specialized speech/noise detection transforms are deferred.
- **Search Ranking**: Text search defaults to **Relevance-Biased (BM25)** to surface the most relevant matches first (Section 8.3).
- **Multi-Monitor Handling**: Screen capture is implemented as **Per-Monitor Streams**, allowing independent navigation and replay of individual displays (Section 3.1).

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

### 18.3 Delivery and Indexing Semantics

- ACK implies **persistence and durability** (Section 6.2).
- Indexing strategy: **Relaxed/Background Indexing** (Section 6.2.1). (ACK is sent as soon as metadata/blobs are persisted; indexing happens as a background task, with a small latency before records are searchable.)

### 18.4 API Surface

- **gRPC-only** for UI and collectors (Section 5.4).

### 18.5 Storage

- Hot store DB: **SurrealDB** (Section 8.0).
- Blob store: **filesystem content-addressed store (CAS)** (Section 8.2.1).
- Cold tier (future): **hybrid hot/cold** with optional Parquet export later (Section 8.2.1).

### 18.6 Capture and Retention

- Keystrokes: **full text capture with minimal controls** (Section 12.4). (High risk acknowledged.)
- Default retention: **forever** (Section 12.5).

### 18.7 Query Authoring and Performance

- Query syntax: Templates + builder first, DSL as advanced view (Section 18.7).
- Query performance target: **Snappy (<1s latency)** for most queries.
- Audio clipping: **Raw Overlapping Chunks** (Return full 10s-30s chunks that overlap the query window instead of precisely clipped segments).

### 18.8 Environment and Extensibility

- Operating System: **NixOS** (Systemd user services for collectors).
- Shell Support: **Bash, Zsh, and Fish** must be supported on Day 1.
- Enrollment/Pairing: **Token-based Authentication** (Default choice for v1).

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

## 20) Implemented Feature Notes

### 20.1 Search Previews (March 1, 2026)

- Search UI now renders richer result cards with:
  - text snippets generated from frame content fields,
  - query-term highlighting within snippets,
  - image thumbnails for screen/camera modalities.
- Thumbnail retrieval uses a dedicated interface backend command:
  - `get_frame_data_thumbnails` (Tauri command),
  - screen/camera image payloads are downscaled before being encoded as `data:` URLs.
- Result card media loading is lazy for viewport efficiency:
  - thumbnail images mount only once cards approach viewport,
  - a skeleton placeholder is shown before image decode/load completes.

### 20.2 Security Hardening (March 1, 2026)

- gRPC TLS is now enforced for server startup:
  - server fails fast when `LIFELOG_TLS_CERT_PATH` or `LIFELOG_TLS_KEY_PATH` is missing.
- gRPC authentication token configuration is now enforced on server startup:
  - `LIFELOG_AUTH_TOKEN` and `LIFELOG_ENROLLMENT_TOKEN` must be present.
- Server-side auth interceptor rejects missing/invalid `Authorization: Bearer ...` metadata.
- Collector control/upload paths now enforce secure transport:
  - collector and upload manager reject non-`https://` server URLs.
- Collector pairing flow is wired into handshake:
  - when no auth token is present, collector calls `PairCollector` using enrollment token before opening `ControlStream`,
  - collector includes `x-lifelog-client-id` metadata hint; server returns that stable id when provided.

### 20.3 Retention Controls (March 1, 2026)

- `ServerConfig` now includes `retention_policy_days` (`map<string,uint32>`), where `0` means keep forever.
- Server now runs a retention worker loop (daily by default; configurable via `LIFELOG_RETENTION_INTERVAL_SECS`) that:
  - prunes stale records per modality policy using canonical time when present,
  - collects candidate `blob_hash` values from deleted records,
  - deletes orphan CAS blobs only when no remaining metadata row references the hash.
- Server `SetConfig` is no longer a no-op:
  - request payload now carries full `SystemConfig`,
  - server applies server policy updates live (`default_correlation_window_ms`, `retention_policy_days`),
  - collector updates are propagated over `ControlStream` via `UpdateConfig` command payloads.
- Interface Settings now includes `Privacy & Storage` controls for `screen`, `audio`, and `text` retention days, wired through existing `get_component_config` / `set_component_config`.
\n--- REPO MAP ---
{
  tauri_commands: [
73:#[tauri::command]
81:#[tauri::command]
226:#[tauri::command]
236:#[tauri::command]
248:#[tauri::command]
269:#[tauri::command]
367:#[tauri::command]
677:#[tauri::command]
829:#[tauri::command]
864:#[tauri::command]
913:#[tauri::command]
1060:#[tauri::command]
1099:#[tauri::command]
1620:#[tauri::command]
1642:#[tauri::command]
1676:#[tauri::command]
  ],
  grpc_services: [
    "ControlStream",
    "GetConfig",
    "SetConfig",
    "GetData",
    "Query",
    "Replay",
    "GetState",
    "UploadChunks",
    "GetUploadOffset",
    "ListModalities",
    "PairCollector"
  ],
  react_components: [
    "ui/theme-toggle.tsx",
    "TextUploadDashboard.tsx",
    "CameraDashboard.tsx",
    "Header.tsx",
    "ui/label.tsx",
    "ui/separator.tsx",
    "ReplayFrame.tsx",
    "ReplayDashboard.tsx",
    "FeatureTabs.tsx",
    "DevicesDashboard.tsx",
    "ScreenDashboard.tsx",
    "NetworkTopologyDashboard.tsx",
    "ui/dialog.tsx",
    "ui/progress.tsx",
    "ui/tooltip.tsx",
    "ui/switch.tsx",
    "ui/select.tsx",
    "Sidebar.tsx",
    "ui/slider.tsx",
    "ModulesPanel.tsx",
    "MicrophoneDashboard.tsx",
    "ui/input.tsx",
    "ProcessesDashboard.tsx",
    "SettingsDashboard.tsx",
    "ui/spinner.tsx",
    "Login.tsx",
    "ResultCard.tsx",
    "ui/tabs.tsx",
    "ui/card.tsx",
    "SearchDashboard.tsx",
    "PlaceholderDashboard.tsx",
    "TimelineDashboard.tsx",
    "ui/button.tsx",
    "ui/dropdown-menu.tsx"
  ],
  rust_modules: [
    "server/src/retention.rs",
    "server/src/policy.rs",
    "server/src/transform.rs",
    "server/src/grpc_service.rs",
    "server/src/replay.rs",
    "server/src/data_retrieval.rs",
    "server/src/postgres.rs",
    "collector/src/main.rs",
    "collector/src/modules/mod.rs",
    "collector/src/modules/mouse.rs",
    "collector/src/modules/microphone.rs",
    "collector/src/modules/text_upload.rs",
    "collector/src/collector.rs",
    "server/src/main.rs",
    "collector/src/modules/logger.rs",
    "collector/src/setup.rs",
    "collector/src/modules/data_source.rs",
    "server/src/lib.rs",
    "collector/src/lib.rs",
    "collector/src/modules/evdev_input_logger.rs",
    "collector/src/modules/input_logger.rs",
    "collector/src/modules/processes.rs",
    "collector/src/modules/camera.rs",
    "collector/src/modules/browser_history.rs",
    "server/src/sync.rs",
    "collector/src/modules/window_activity.rs",
    "collector/src/modules/clipboard.rs",
    "collector/src/modules/weather.rs",
    "collector/src/modules/shell_history.rs",
    "server/src/query/ast.rs",
    "collector/src/modules/wayland_input_logger.rs",
    "collector/src/modules/audio.rs",
    "server/src/query/planner.rs",
    "server/src/query/llql.rs",
    "server/src/ingest.rs",
    "server/src/query/executor.rs",
    "collector/src/modules/hyprland.rs",
    "collector/src/modules/keystrokes.rs",
    "collector/src/modules/screen.rs",
    "server/src/query/mod.rs",
    "server/src/schema.rs",
    "server/src/db.rs",
    "server/src/server.rs",
    "collector/src/collector/upload_manager.rs"
  ]
}
\n--- STATUS & GAPS ---
# Status

## Current Objective

Phase 6: Query Engine Completion (correlation + replay) and UI integration.

## Last Verified

- `just check`
- `just test` (nextest; integration tests requiring SurrealDB remain `#[ignore]`)

## How To Verify (Target)

- `just validate`
- `nix develop --command cargo test -p lifelog-server --test ocr_pipeline -- --include-ignored`
- `nix develop --command cargo test -p lifelog-server --test canonical_llql_example -- --include-ignored`

## What Changed Last

- **Temporal OR Planning**: Query planner now supports `OR` over temporal operators (`WITHIN`/`DURING`/`OVERLAPS`) by converting boolean filters to a bounded DNF and unioning the resulting conjunctive plans.
- **Union Deduplication**: Query executor now deduplicates UUID results across subplans in `MultiQuery` execution (important for OR/union queries).
- **Collector Audio Source**: Collector now has an `AudioDataSource` that records fixed-duration chunks via CPAL, encodes them as WAV `AudioFrame`s, and WAL-buffers them for upload on stream_id `audio`.
- **Collector Mouse Source**: Collector now has a `MouseDataSource` that samples cursor position and WAL-buffers `MouseFrame`s for upload on stream_id `mouse`; server ingest + `GetData` retrieval support the `Mouse` modality end-to-end.
- **Durable ACK Gating**: `UploadChunks` only advances `Ack.acked_offset` when the backend marks the chunk as indexed/queryable (`upload_chunks.indexed=true`). Screen ingestion records store `frame_uuid` in `upload_chunks` and pin ACK until the OCR-derived record for that UUID has been persisted.
- **Collector Config Hot-Reload**: Collector now applies `UpdateConfig` commands by parsing the JSON payload into `CollectorConfig` and restarting sources in-process (without dropping the ControlStream).
- **Record Type Semantics**: Added explicit `record_type` (`Point` vs `Interval`) to frame protos and populated it in `GetData` responses.
- **Clipboard Binary CAS Storage**: Clipboard frames with binary payloads now store the payload in CAS and persist only the CAS reference in SurrealDB; `GetData` hydrates from CAS when needed.
- **Unified Error Hierarchy**: Migrated all local error types to a single `LifelogError` enum in `lifelog-core` for consistent error handling and reporting.
- **Unified Trait Model**: Finalized `DataType`, `Modality`, and `ToRecord` traits. `ToRecord` ensures SurrealDB 2.x compatibility for complex types like `datetime` and `bytes`.
- **Explicit Catalog**: Replaced `INFO FOR DB` table discovery with a dedicated `catalog` table for robust and efficient origin discovery during queries.
- **Robust Transformation Pipeline**: Verified OCR transformation end-to-end, including watermark persistence and idempotent processing.
- **Cross-Modal Search**: Successfully implemented and verified unified search across different data sources (Screen, Browser).
- **Cross-Modal Correlation**: Added `DURING(...)` support as a two-stage query plan (source intervals -> target time-window filter) alongside `WITHIN(...)`.
- **DURING Enhancements**: `DURING(...)` now supports an explicit window expansion for point sources and conjunction of multiple `DURING(...)` terms via interval intersection.
- **Interval Overlap Semantics**: Added `t_end` metadata and updated temporal joins so interval targets (notably Audio) use overlap semantics (`t_canonical`/`t_end`) instead of “start timestamp only”.
- **OVERLAPS Operator**: Added `OVERLAPS(...)` to the typed query AST/LLQL and wired it through planner/executor (currently equivalent to `DURING(...)` execution).
- **Replay Backend + UI**: Added a `Replay` gRPC RPC that returns ordered screen-granularity steps plus aligned context keys, and wired an interface Replay view to it.
- **Canonical Time Wire Fields**: Added `t_device`/`t_ingest`/`t_canonical`/`t_end`/`time_quality` to frame protos and populated them in server `GetData` responses; OCR derived frames now propagate canonical time metadata from source frames.
- **Clock Skew Estimation Wired**: Added periodic clock sync over `ControlStream` so collectors report `(device_now, backend_now)` samples; server computes per-collector skew estimates and applies them at ingest to populate `t_canonical` and `time_quality`.
- **Default Correlation Window**: Added `ServerConfig.default_correlation_window_ms` and wired temporal operators to fall back to it when a query omits a per-predicate window (LLQL supports omitting `window`).
- **Query Resource Limits**: Added default resource bounds to query execution: `LIMIT 1000` on UUID-returning queries and a `10s` SurrealDB query timeout.
- **Timeline Query Mode**: Timeline UI now submits `Query.text` as a string array and supports an LLQL mode (`llql:` / `llql-json:`) for cross-modal queries.
- **Canonical LLQL Example Verified**: Added an ignored integration test that seeds Browser/OCR/Audio and runs the Spec §10.2 canonical query end-to-end via LLQL JSON.
- **Temporal Conjunction Queries**: Temporal correlation operators can now be mixed under `AND` (including multiple `WITHIN(...)` terms) by intersecting interval sets at execution time.
- **Schema DDL Validation**: Startup migrations and table schema creation now call `.check()` so SurrealDB DDL/index errors surface immediately.
- **Performance Baselines**: Established throughput and latency benchmarks via `performance_suite.rs`.
- **Improved Test Coverage**: Added unit tests for `DiskBuffer`, `TimeInterval`, `ReplayStep`, and config validation.
- **Collector Window Activity Source**: Added a `WindowActivityDataSource` (stream_id `window_activity`) that emits interval frames and supports `"auto"` backend selection with Hyprland (`hyprctl`) and X11 (`xdotool`/`xprop`) fallbacks.
- **Interface TypeScript Build Clean**: Fixed unused imports/types so `interface` passes `npm run build` (`tsc` + `vite build`).

## What's Next

- **UI Integration**: Add query builder/templates for LLQL (and add richer previews in search/replay).
- **Canonical Query (Spec §10.2)**: Extend validation to real ingest + OCR transform + Audio capture on a live collector (beyond the integration test seed data).
- **Security**: Add pairing + auth, and enforce TLS.
- **New Modalities**: Implement missing v1 collectors (window activity fallback), then gate keystrokes behind security controls.

## Blockers

- None.
\n--- TASK & PLAN ---
# PostgreSQL Migration Phase 4: Operations, Deployment, & Finalization

**Objective:** Ensure a smooth transition for existing development environments and production deployments, and remove SurrealDB dependencies where no longer needed.

- [ ] **Task 4.1: NixOS & Systemd Updates**
  - Update `flake.nix` to provision PostgreSQL (`services.postgresql.enable = true`) and automatically create the `lifelog` database/user.
  - Update `deploy/systemd/lifelog-server.service` and other related systemd files to depend on postgres instead of surrealdb (or both during transition).
- [ ] **Task 4.2: Config & Documentation**
  - Update `lifelog-config.toml` examples to use PostgreSQL as the default.
  - Update `USAGE.md` and `README.md` to reflect the new PostgreSQL requirement.
- [ ] **Task 4.3: Health & Metrics**
  - Ensure `ReportState` and observability endpoints correctly reflect PostgreSQL pool metrics (active connections, idle connections).
- [ ] **Task 4.4: Final Clean-up (Strategic)**
  - If Phase 1-3 are fully verified, consider making PostgreSQL the mandatory default and marking SurrealDB for removal in a future release.
  - Clean up any temporary "Hybrid" logic if it's no longer serving a purpose.\n--- STATE HISTORY (LATEST) ---
          - Preserved CAS linkage by storing `blob_hash`/`blob_size` in Postgres modality tables.
          - Added optional Postgres ingest pool bootstrap in `Server::new` from `LIFELOG_POSTGRES_INGEST_URL` plus migration run.
          - Updated `GetUploadOffset` RPC to query PostgreSQL when ingest pool is enabled.
      </how>

      <validation_steps>
           - `just check-digest` (pass).
           - `tools/ai/run_and_digest.sh "just test"` (pass).
      </validation_steps>

</state_snapshot>

<state_snapshot>
      <overall_goal>
      Implement Phase 3 PostgreSQL migration for query execution by translating AST/plans to native PostgreSQL operations, adding temporal overlap SQL execution, and routing query/replay between SurrealDB and PostgreSQL.
      </overall_goal>

      <what_to_do>
          - Added PostgreSQL execution for `ExecutionPlan::TableQuery` with native filtering and FTS translation.
          - Added PostgreSQL-native temporal execution for `ExecutionPlan::DuringQuery` using range overlap checks in SQL (`time_range && ...`) with source-term `EXISTS` joins.
          - Added hybrid query routing in server query path based on plan compatibility and configured Postgres pool.
          - Added hybrid origin discovery (Surreal catalog + Postgres distinct collector/origin scan).
          - Updated replay to use PostgreSQL range queries for migrated modalities (screen/context) with Surreal fallback for non-migrated modalities.
          - Refactored planner plans to carry backend-agnostic filters (`Expression`) alongside existing Surreal SQL strings.
      </what_to_do>
      <why>
          - Phase 3 requires moving temporal overlap work from Rust interval materialization into PostgreSQL execution for performance and reduced memory overhead.
          - Hypothesis: keeping the existing planner shape while adding AST-bearing plan fields enables dual-engine execution with minimal regression risk.
          - Assumption tested: a plan-level compatibility check allows safe per-query/per-plan backend routing for incremental migration.
      </why>

      <how>
          - Reworked `ExecutionPlan` variants in `server/src/query/planner.rs` to include filter expressions and explicit limits used by Postgres execution.
          - Implemented `execute_postgres(...)` and `plan_is_postgres_compatible(...)` in `server/src/query/executor.rs`.
          - Implemented Postgres expression compilation:
            - `Expression::TimeRange` -> `time_range && tstzrange(...)`
            - `Expression::Contains` -> `search_document @@ websearch_to_tsquery('english', ...)` on supported tables.
          - Implemented temporal translation with SQL `EXISTS` source subqueries and overlap checks against expanded source ranges.
          - Updated `process_query` in `server/src/server.rs` to route plans to Postgres when compatible; otherwise use Surreal executor.
          - Added Postgres-origin introspection and merged origin availability for selector resolution.
          - Updated replay screen/context fetches to query Postgres tables ordered by `lower(time_range)` where supported.
      </how>

      <validation_steps>
           - `just check-digest` (pass).
           - `tools/ai/run_and_digest.sh "nix develop --command cargo test -p lifelog-server plans_temporal_or_via_dnf_union --lib"` (pass).
           - `tools/ai/run_and_digest.sh "just test"` (pass).
      </validation_steps>

</state_snapshot>
\n### INSTRUCTIONS\nYou have been provided with the full SPEC, STATUS, and PLAN. DO NOT waste turns reading these files. Begin implementation immediately. If this is a UI task, the discovery map contains component paths.
