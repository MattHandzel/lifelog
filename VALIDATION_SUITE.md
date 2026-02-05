# Software Validation Suite (BDD) for Lifelog v1

This document is the validation framework to verify the implementation is compliant with `SPEC.md` (ground truth). It includes:

1. Requirements Traceability Matrix (RTM)
2. BDD user stories with Gherkin acceptance criteria (happy/edge/negative paths)
3. Unit test specifications (pseudocode)
4. Integration test specifications (pseudocode)

It is written at a high enough level to be implemented in any language/test runner.

---

## Requirements Traceability Matrix (RTM)

| Req ID | Requirement (Design Doc Source) | Type | User Story ID(s) | Test Case ID(s) |
|---|---|---|---|---|
| REQ-001 | Provide recall via timeline, search, and replay across all stored time (SPEC §1.1, §11.1) | Functional | US-001, US-002, US-003 | IT-001, IT-002, IT-003 |
| REQ-002 | Support cross-modal reconstruction queries: return stream A constrained by predicates over streams B/C (SPEC §1.2, §4.3, §10) | Functional | US-004, US-005 | UT-020, IT-010, IT-011 |
| REQ-003 | Passive capture: no manual save/organize; failures are quiet alerts (SPEC §1.3, §14, §19.4) | Non-functional | US-006, US-007 | IT-020, IT-021 |
| REQ-004 | Local-first: no cloud sync/storage targets in v1 (SPEC §1.4, §2) | Constraint | US-008 | IT-030 |
| REQ-005 | Collect required v1 streams (screen, mic audio, browser, app/window, keystrokes, mouse, clipboard, shell) (SPEC §3.1, §12.4) | Functional | US-009 | UT-001..UT-007, IT-040 |
| REQ-006 | Multi-device readiness: phone can access web UI; backend tolerates multiple collectors (SPEC §3.2, §5.1) | Functional | US-010 | IT-050 |
| REQ-007 | Canonical timeline time is device time corrected by estimated skew; store device + ingest times (SPEC §4.0, §4.2, §4.2.1) | Functional | US-011 | UT-010, IT-060 |
| REQ-008 | Point vs interval records supported; audio is interval; screen is point; replay maps screen frame `t_i` to `[t_i, t_{i+1})` (SPEC §4.1, §10.3.1) | Functional | US-012 | UT-011, IT-061 |
| REQ-009 | Correlation operators exist: WITHIN / OVERLAPS / DURING; default Δt is configurable (SPEC §4.3) | Functional | US-004, US-005 | UT-020..UT-024, IT-010 |
| REQ-010 | Backend-first distributed architecture: control plane + data plane separation (SPEC §5.2) | Architecture | US-013 | IT-070 |
| REQ-011 | NAT-safe connectivity: collectors initiate outbound connection; backend “pull” via control channel (SPEC §5.3) | Architecture | US-013 | IT-071 |
| REQ-012 | gRPC-only API surface for UI and collectors (SPEC §5.4) | Constraint | US-014 | IT-072 |
| REQ-013 | Collector disk-backed WAL/queue buffering; retention controlled by backend ACK (SPEC §6.1) | Functional | US-015 | UT-030, IT-080 |
| REQ-014 | Durable ACK only when fully queryable: metadata+blobs persisted and baseline indexes updated (SPEC §6.2, §6.2.1) | Functional | US-016 | UT-031..UT-033, IT-081 |
| REQ-015 | Resumable upload by chunk offsets; idempotent writes keyed by collector/stream/session/offset/hash (SPEC §6.3, §6.4) | Functional | US-017 | UT-040..UT-043, IT-090 |
| REQ-016 | Offset unit is byte offset within per-stream session file (SPEC §7.3.1) | Functional | US-017 | UT-040, IT-091 |
| REQ-017 | Cancelation/backpressure: backend can cancel; collectors pause and keep buffering (SPEC §7.4) | Functional | US-018 | IT-092 |
| REQ-018 | Storage separates metadata store and blob store; blobs are content-addressed filesystem CAS (SPEC §8.2, §8.2.1) | Architecture | US-019 | UT-050..UT-052, IT-100 |
| REQ-019 | Hot store DB is SurrealDB with explicit indexing and explicit catalog; avoid runtime table discovery for catalog (SPEC §8.0) | Architecture | US-020 | IT-101, IT-102 |
| REQ-020 | Time index and text search index exist for baseline fields (SPEC §6.2.1, §8.3) | Functional | US-021 | IT-103 |
| REQ-021 | OCR transform exists; derived stream with lineage; transform jobs idempotent and resumable; versioned outputs (SPEC §9) | Functional | US-022 | UT-060..UT-062, IT-110 |
| REQ-022 | Query language deterministically compiles to typed plan; bounded resource usage; canonical example works (SPEC §10.1–§10.2) | Functional | US-023, US-024 | UT-070..UT-074, IT-120 |
| REQ-023 | Replay query returns ordered steps; can align context streams within correlation window (SPEC §10.3, §11.1) | Functional | US-003, US-025 | IT-121 |
| REQ-024 | UI is web UI served by backend; works on desktop + phone; can cancel in-flight queries (SPEC §11.2) | Functional | US-026 | IT-130 |
| REQ-025 | TLS for collector/backend and UI/backend (SPEC §12.1) | Security | US-027 | IT-140 |
| REQ-026 | Secure enrollment/pairing prevents arbitrary device registration (SPEC §12.2) | Security | US-028 | IT-141 |
| REQ-027 | Minimal safety rails: pause/resume, per-stream disable, retention controls; default retention forever (SPEC §12.3, §12.5) | Functional | US-029 | IT-142 |
| REQ-028 | Keystroke capture policy: full text with minimal controls; mitigations required (SPEC §12.4) | Security/Functional | US-030 | IT-143 |
| REQ-029 | Observability: backend exposes connectivity/backlog/last pull/errors/transform lag/storage usage; collectors expose capture status/buffer fullness/last ack (SPEC §14) | Non-functional | US-007, US-031 | IT-150 |
| REQ-030 | Performance suite exists; must measure query latency, ingest throughput, replay assembly (SPEC §13.1–§13.2) | Non-functional | US-032 | IT-160 |
| REQ-031 | Eliminate known artifact divergences (single canonical backend; no split REST/gRPC; non-stub query; disk buffering; NAT-safe) (SPEC §16) | Constraint | US-033 | IT-170 |

---

## BDD User Stories & Acceptance Criteria

### Feature: Recall (Timeline, Search, Replay)

**US-001 Timeline Browse**  
As a knowledge worker, I want to browse a unified timeline across devices and modalities, so that I can reconstruct what happened at any time.

```gherkin
Scenario: Timeline returns records in canonical time order
  Given a backend with records from multiple streams and devices
  And each record has device time, ingest time, and canonical time
  When I request a timeline slice for a time range
  Then results are ordered by canonical time
  And each item includes enough metadata to open a replay view

Scenario: Timeline handles clock skew
  Given two devices with a 5-minute clock skew
  When both send records for the same real-world interval
  Then the backend’s canonical timeline places them correctly relative to each other
  And the UI marks time quality if skew confidence is low
```

**US-002 Search**  
As a power user, I want to search across OCR, URLs, clipboard, and shell history, so that I can find the exact moment I need.

```gherkin
Scenario: Text search returns matched moments
  Given OCR and browser text indexes are built
  When I run a search for "3Blue1Brown"
  Then I receive matching moments with previews (snippet + timestamp)
  And each result links to replay at that time

Scenario: Search respects time filters
  Given matches exist across many days
  When I add a time range restriction
  Then only matches within that range are returned
```

**US-003 Replay**  
As a power user, I want to replay a time window step-by-step, so that I can recover lost context (e.g., reconstruct deleted work).

```gherkin
Scenario: Replay steps map screen point frames to intervals
  Given screen frames at times t0, t1, t2 within a replay window
  When I request replay for [t0, t2)
  Then replay steps are [t0, t1) and [t1, t2)
  And each step includes the frame at the step start

Scenario: Replay aligns context events
  Given clipboard and shell events near a replay step time
  When I request replay
  Then each step includes events within the correlation window
```

### Feature: Cross-Modal Queries (Core Capability)

**US-004 Cross-Modal Filter**  
As a power user, I want to retrieve audio during times when browser and OCR conditions were true, so that I can find what I was listening to while watching/reading something.

```gherkin
Scenario: Canonical cross-modal query works
  Given audio chunks exist as interval records
  And browser events include URLs
  And OCR text exists for screen frames
  When I execute a query selecting audio during times where URL contains "youtube" and OCR contains "3Blue1Brown"
  Then returned audio segments overlap the matched time windows
  And results include evidence links (matching URL and OCR snippets)

Scenario: No matches returns empty without errors
  Given no OCR text contains "3Blue1Brown"
  When I execute the cross-modal query
  Then I receive an empty result set
  And the backend returns success with zero results
```

**US-005 Correlation Operators**  
As a developer, I want correlation operators to have deterministic semantics, so that query results are stable and debuggable.

```gherkin
Scenario: WITHIN uses configured default window
  Given Δt_default is configured to 30 seconds
  When I execute a query using WITHIN without specifying Δt
  Then the backend uses 30 seconds

Scenario: OVERLAPS on intervals matches expected overlaps
  Given interval A [10:00,10:05) and interval B [10:04,10:10)
  When I execute OVERLAPS(A,B)
  Then the predicate evaluates true
```

### Feature: Collector Enrollment, Control Plane, and NAT-Safe Operation

**US-013 Backend-Controlled Pull Over Collector-Initiated Connection**  
As a user, I want collectors to work on roaming/NAT networks, so that multi-device capture works without manual networking setup.

```gherkin
Scenario: Collector initiates control channel and backend schedules upload
  Given a collector can reach the backend outbound
  When the collector starts
  Then it establishes a long-lived control channel to the backend
  And the backend can authorize an upload session over that channel

Scenario: Backend cancels an upload session
  Given an upload session is in progress
  When the backend cancels the session
  Then the collector stops sending
  And the collector retains unacked data in its disk buffer
```

### Feature: Durable Buffering, Upload, Idempotency, ACK

**US-015 Disk WAL buffering**  
As a user, I want collectors to buffer on disk, so that data is not lost on crash/reboot.

```gherkin
Scenario: Collector crash does not lose buffered records
  Given a collector has captured records and persisted them to its disk buffer
  When the collector process crashes and restarts
  Then the buffered records remain available to upload
  And no records are silently dropped
```

**US-016 ACK implies fully queryable**  
As a user, I want acknowledgements to mean “I can query it now,” so that I never lose data or see confusing partial states.

```gherkin
Scenario: ACK withheld until indexes are updated
  Given the backend has persisted blobs and metadata but indexes are not updated
  When the backend receives chunks for a session
  Then the backend does not ACK those offsets
  And once baseline indexes complete, ACK is issued for the durable offset

Scenario: Backpressure applied under indexing lag
  Given indexing falls behind ingestion
  When the collector attempts to upload at full speed
  Then the backend applies backpressure or limits sessions
  And the collector continues buffering on disk
```

**US-017 Resumable chunked upload**  
As a developer, I want upload to resume by byte offset safely, so that intermittent networks don’t cause duplication or loss.

```gherkin
Scenario: Resume after disconnect
  Given an upload session with chunks at offsets 0..N
  And the collector disconnects after sending chunk at offset K
  When the collector reconnects and asks for the persisted offset
  Then the backend returns offset K'
  And the collector resumes from K'
  And the backend does not duplicate records
```

### Feature: Storage, Indexing, Transforms

**US-019 Blob CAS**  
As a developer, I want blobs stored in a filesystem CAS, so that DB stays small and blobs are deduplicated.

```gherkin
Scenario: Identical blobs deduplicate
  Given two records reference identical blob content
  When both blobs are ingested
  Then the CAS stores one physical object
  And both records reference the same hash
```

**US-022 OCR transform**  
As a user, I want OCR derived text for screens, so that screen content becomes searchable.

```gherkin
Scenario: OCR produces derived records with lineage
  Given a screen frame exists in storage
  When the OCR transform runs
  Then a derived OCR record is created
  And it references the source screen record
  And it is indexed for text search
```

### Feature: UI (Web UI Served by Backend, gRPC-only, Cancellation)

**US-026 Web UI recall**  
As a user, I want a backend-served UI that works on desktop and phone, so that I can recall anywhere.

```gherkin
Scenario: UI can cancel a query
  Given a query is running
  When I issue a cancel action
  Then the backend stops work for that query
  And resources are released
  And the UI receives a cancellation acknowledgement
```

### Feature: Security & Privacy Baseline

**US-027 TLS**  
As a user, I want encrypted transport, so that my data cannot be sniffed on the network.

```gherkin
Scenario: Backend refuses non-TLS connections (multi-device mode)
  Given the backend is configured for multi-device operation
  When a collector attempts plaintext connection
  Then the backend rejects the connection
```

**US-028 Pairing**  
As a user, I want secure pairing, so that only my devices can join.

```gherkin
Scenario: Unpaired collector cannot register
  Given a collector without pairing credentials
  When it calls RegisterCollector
  Then registration is denied
```

**US-029 Pause/Retention**  
As a user, I want pause and retention controls, so that I can stop capture and manage storage.

```gherkin
Scenario: Global pause stops capture
  Given capture is enabled
  When I issue a global pause
  Then collectors stop capturing new data
  And collectors continue buffering and uploading existing data unless configured otherwise
```

**US-030 Keystroke capture**  
As a power user, I want full keystroke text capture, so that I can reconstruct work precisely.

```gherkin
Scenario: Keystroke text is captured and queryable after ACK
  Given keystroke capture is enabled
  When I type text during normal usage
  Then records are captured
  And after ACK, they are searchable via text index
```

### Feature: Observability and Performance Suite

**US-031 Health/backlog visibility**  
As a user, I want to see device health and backlog, so that failures are quiet but obvious.

```gherkin
Scenario: Device backlog and last-seen are visible
  Given a collector is capturing and buffering
  When I request system state
  Then I can see last-seen times and backlog size per stream
```

**US-032 Performance suite**  
As a developer, I want a performance suite, so that I can verify latency/throughput/backlog recovery.

```gherkin
Scenario: Performance suite executes defined workloads
  Given a seeded corpus and indexes
  When I run the performance suite
  Then it reports query latency, ingest throughput, and backlog recovery time
  And it fails if any metric exceeds configured thresholds
```

---

## Unit Test Specifications

### Time Skew Estimator
- Objective: compute and apply skew estimate; produce `time_quality`.
- Pseudocode:
  - Arrange: mock collector samples `(device_now, backend_now)` over time.
  - Act: compute skew estimate and confidence.
  - Assert: `t_canonical = t_device + skew`; confidence decreases with jitter/outliers.

### Replay Step Builder
- Objective: map screen point frames into replay intervals as specified.
- Pseudocode:
  - Input: sorted frame timestamps `[t0,t1,t2]`, `replay_window_end`.
  - Output: steps `[(t0,t1),(t1,t2)]`, last-step-end rule for final frame.
  - Assert: boundaries match spec; handles single-frame window.

### Correlation Operator Evaluator
- Objective: deterministic semantics for WITHIN/OVERLAPS/DURING (once truth tables are defined).
- Pseudocode:
  - For each operator, feed point/interval combinations.
  - Assert: expected boolean results for worked examples.
  - Include: default Δt usage and override behavior.

### Chunk Offset Validator
- Objective: enforce offsets are byte offsets; chunk sizes consistent; hash computed correctly.
- Pseudocode:
  - Input: chunk bytes, declared offset.
  - Assert: offset increments by chunk length; reject overlaps/gaps unless explicitly allowed for resume; compute hash and match declared hash.

### Idempotent Chunk Apply
- Objective: repeated chunk writes with same identity key do not duplicate records.
- Pseudocode:
  - Arrange: mock metadata store + CAS.
  - Act: apply same chunk twice.
  - Assert: exactly one set of records persisted; CAS stores blob once; ack cursor unchanged after second apply.

### Durable ACK Gate
- Objective: ACK only after baseline indexes updated.
- Pseudocode:
  - Arrange: mock “persist metadata OK”, “persist blobs OK”, “index update pending”.
  - Act: attempt to finalize ack at offset K.
  - Assert: ack withheld; after index completion event, ack advances.

### Filesystem CAS
- Objective: content hash addressing and dedupe.
- Pseudocode:
  - Write blob A; write identical blob A again.
  - Assert: same hash; physical object count remains 1; metadata refs point to same hash.

### OCR Transform Job
- Objective: idempotent derived output + lineage.
- Pseudocode:
  - Arrange: a source screen record exists.
  - Act: run OCR transform twice.
  - Assert: derived record exists once per transform version; lineage references correct source; text indexed.

### SurrealDB Catalog Manager
- Objective: stream/origin catalog is explicit; no `INFO FOR DB` discovery required.
- Pseudocode:
  - Arrange: create streams via registration.
  - Act: query catalog.
  - Assert: stable stream list; queries use catalog IDs not table enumeration.

### Security Policy Checker
- Objective: enforce TLS required, pairing required, reflection disabled in prod mode.
- Pseudocode:
  - Attempt plaintext connection; assert reject.
  - Attempt register without pairing; assert deny.
  - Attempt reflection call in prod; assert deny.

---

## Integration Test Specifications

### IT-010 Cross-Modal Query End-to-End
- Objective: ingest browser + screen + OCR + audio; execute canonical query; verify results.
- Pseudocode:
  - Setup:
    - Start backend with SurrealDB + CAS + baseline indexes enabled.
    - Start one collector with deterministic test streams.
    - Ingest: browser events containing “youtube”, screen frames whose OCR contains “3Blue1Brown”, audio chunks overlapping same windows.
  - Execute:
    - Run query selecting audio DURING predicates on browser URL and OCR.
  - Assert:
    - Returned audio segments overlap correct windows.
    - Evidence includes matching URL + OCR snippets.
    - Query respects Δt_default and explicit overrides.

### IT-060 Canonical Time Across Devices
- Objective: multi-device skew correction yields stable ordering.
- Pseudocode:
  - Setup two collectors with synthetic device clocks skewed by +300s.
  - Ingest synchronized “real-world” events.
  - Assert timeline ordering uses corrected canonical time.
  - Assert time_quality degrades if skew samples are inconsistent.

### IT-080 Crash/Restart Durability
- Objective: disk buffer prevents loss; no duplicates under resume.
- Pseudocode:
  - Setup collector with disk WAL; capture N records; do not upload yet.
  - Kill collector process.
  - Restart collector; confirm WAL still has N records.
  - Begin upload session; complete; verify backend has N records and indexes; collector WAL prunes only after ACK.

### IT-090 Resume Upload with Byte Offsets
- Objective: resume from persisted offset; idempotent.
- Pseudocode:
  - Begin upload session; send chunks 0..K; drop connection mid-stream.
  - Reconnect; request offset; resume from returned offset.
  - Assert no gaps/duplication; final ACK corresponds to fully indexed data.

### IT-081 ACK implies queryable
- Objective: validate that ACK does not advance until text+time indexes updated.
- Pseudocode:
  - Configure indexer to delay.
  - Upload chunk that introduces OCR text and browser URL.
  - Assert ACK withheld until indexer reports completion.
  - Then query immediately and assert results include the new records.

### IT-100 Blob Separation
- Objective: DB rows do not contain large blob payloads; only CAS references.
- Pseudocode:
  - Ingest a screen frame and audio chunk.
  - Assert SurrealDB record stores hash refs, not raw bytes.
  - Assert CAS contains objects at expected hash paths.

### IT-110 OCR Transform Pipeline
- Objective: derived records appear, indexed, and are lineage-linked.
- Pseudocode:
  - Ingest screen frames.
  - Trigger transform job runner.
  - Assert OCR records exist; search finds them; lineage points back to screen.

### IT-130 UI Query Cancellation
- Objective: cancel in-flight query releases backend work.
- Pseudocode:
  - Start a long-running query (large time range).
  - Issue cancel from client.
  - Assert backend terminates query execution and client receives cancel status.
  - Assert subsequent query executes normally.

### IT-140 TLS and Pairing Enforcement
- Objective: reject insecure/unpaired collectors and UI.
- Pseudocode:
  - Attempt collector register without pairing; assert deny.
  - Attempt plaintext UI connection; assert deny when multi-device mode enabled.
  - Attempt reflection call; assert deny/gated.

### IT-150 Observability Surface
- Objective: system state exposes last-seen, backlog, last ack, transform lag.
- Pseudocode:
  - Ingest partial data; hold indexing back.
  - Query system state.
  - Assert backlog shows pending bytes/records; last-seen updates; transform lag visible; errors surfaced.

### IT-160 Performance Suite Smoke
- Objective: performance suite runs and gates thresholds.
- Pseudocode:
  - Seed corpus.
  - Run suite.
  - Assert it outputs required metrics and fails when thresholds exceeded.

