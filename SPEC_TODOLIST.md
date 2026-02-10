# SPEC_TODOLIST.md — Gap Analysis: Current Repo vs v1 Spec

Generated 2026-02-08; updated 2026-02-10. Cross-referenced against `SPEC.md`, codebase exploration, and memory files.

**Overall completion: ~50%.** Foundations are solid; core differentiators (cross-modal queries, replay, security) are incomplete.

---

## Legend

- `[x]` — Done and working
- `[~]` — Partially done (noted what's missing)
- `[ ]` — Not started

Priority tiers:
- **P0 (Launch blocker)** — Without this, the system cannot fulfill its stated purpose
- **P1 (Must-have for v1)** — Required by spec, but system is partially usable without it
- **P2 (Should-have)** — Improves quality/reliability, spec requires it but not core UX
- **P3 (Nice-to-have)** — Explicitly deferred or low urgency

---

## 1. Time Model & Correlation Semantics (Spec §4) — P0

| # | Task | Status | Notes |
|---|------|--------|-------|
| 1.1 | Add `t_device`, `t_ingest`, `t_canonical` fields to proto records | `[x]` | Frame protos now include `t_device`/`t_ingest`/`t_canonical` (plus `t_end`); server populates them in `GetData` responses. |
| 1.2 | Add `time_quality` enum to proto (Good/Degraded/Unknown) | `[x]` | Added `TimeQuality` enum to proto; server maps stored quality strings (`good`/`degraded`/`unknown`) into the enum for responses. |
| 1.3 | Collector reports `(device_now, backend_now)` samples in `ControlMessage` | `[x]` | Implemented clock sync: server sends `ClockSync` command with `backend_now`, collector responds with `ClockSample { device_now, backend_now }` (`collector/src/collector.rs`, `server/src/grpc_service.rs`, `proto/lifelog.proto`). |
| 1.4 | Server computes & stores `skew_estimate` per collector | `[x]` | `ServerHandle::handle_clock_sample` stores recent samples and updates `skew_estimates` (`server/src/server.rs`, `common/lifelog-core/src/time_skew.rs`). |
| 1.5 | Server computes `t_canonical = t_device + skew` at ingest time | `[x]` | Ingest applies per-collector skew estimate and persists `t_ingest`/`t_canonical`/`t_end`/`time_quality` (`server/src/ingest.rs`). |
| 1.6 | Add point vs interval semantics to proto (explicit `record_type` field) | `[x]` | Added `RecordType` enum + `record_type` field to frame protos; server populates it in `GetData` responses (Audio/WindowActivity = interval; others = point). |
| 1.7 | Define global default correlation window `Δt_default` in config | `[x]` | Added `ServerConfig.default_correlation_window_ms` and wired it so temporal operators use it when their window is omitted/zero (LLQL JSON supports omitting `window`). |
| 1.8 | Support per-predicate `Δt` overrides in query AST | `[x]` | `WITHIN`/`DURING`/`OVERLAPS` carry an explicit per-term `window`; non-zero overrides the global default. |

---

## 2. Collector Modules — v1 Modalities (Spec §3.1)

### Implemented

| # | Module | Status | Files |
|---|--------|--------|-------|
| 2.1 | Screen capture | `[x]` | `collector/src/modules/screen.rs` |
| 2.2 | Browser activity (URL/title) | `[x]` | `collector/src/modules/browser_history.rs` |
| 2.3 | Process list | `[x]` | `collector/src/modules/processes.rs` (bonus, not in spec §3.1) |
| 2.4 | Camera | `[x]` | `collector/src/modules/camera.rs` (bonus) |
| 2.5 | Weather | `[x]` | `collector/src/modules/weather.rs` (bonus) |
| 2.6 | Hyprland window manager | `[x]` | `collector/src/modules/hyprland.rs` (bonus, provides some window activity on Wayland) |

### Missing (required by Spec §3.1) — P1

| # | Module | Status | Proto exists? | Notes |
|---|--------|--------|---------------|-------|
| 2.7 | Desktop microphone audio | `[ ]` | Yes (`AudioFrame`) | Fixed-interval chunking. High storage cost. |
| 2.8 | Keystrokes | `[ ]` | Yes (`KeystrokeFrame`) | **High risk** (Spec §12.4). Needs security controls before deployment. |
| 2.9 | Mouse events | `[x]` | Yes (`MouseFrame`) | Activity indicators + timestamps (cursor-position sampling; stream_id `mouse`) |
| 2.10 | Clipboard history | `[x]` | Yes (`ClipboardFrame`) | Text + timestamps; binary optional |
| 2.11 | Shell history | `[x]` | Yes (`ShellHistoryFrame`) | Commands + timestamps + working dir |
| 2.12 | App/window activity | `[~]` | Yes (`WindowActivityFrame`) | Hyprland covers Wayland; need X11/generic fallback |

---

## 3. Collector Infrastructure (Spec §5–7) — P1

| # | Task | Status | Notes |
|---|------|--------|-------|
| 3.1 | NAT-safe control channel (collector initiates) | `[x]` | `ControlStream` bidirectional gRPC works |
| 3.2 | Disk-backed WAL buffering | `[x]` | `DiskBuffer` with checksums, commit offsets |
| 3.3 | Resumable uploads (offset-based) | `[x]` | `UploadChunks` + `GetUploadOffset` RPCs |
| 3.4 | Per-stream monotonic sequence numbers | `[ ]` | Spec §4.2 requires these separate from upload offset |
| 3.5 | Backpressure handling (pause sending, keep buffering) | `[~]` | Buffer exists but no explicit backpressure signal handling |
| 3.6 | Upload session cancellation | `[ ]` | Server can't cancel in-progress upload (Spec §7.4) |
| 3.7 | Config hot-reload on `UpdateConfig` command | `[ ]` | Collector receives command but doesn't apply live |

---

## 4. Server — Ingest & Storage (Spec §6, §8) — P0/P1

| # | Task | Priority | Status | Notes |
|---|------|----------|--------|-------|
| 4.1 | Separate blobs to CAS (don't store inline in SurrealDB) | P0 | `[x]` | Screen/Camera/Audio blobs stored in CAS with `blob_hash` + `blob_size`. Clipboard binary payloads (when present) are now stored in CAS; SurrealDB stores only the CAS reference. |
| 4.2 | Define & create text search indexes | P0 | `[x]` | `schema.rs` defines `lifelog_text` analyzer + BM25 search indexes for OCR text, browser URL/title, clipboard text, shell commands, and keystrokes. |
| 4.3 | Durable ACK = metadata + blobs + indexes all persisted | P0 | `[~]` | `UploadChunks` ACK now only advances when the backend reports the chunk as indexed/queryable (`upload_chunks.indexed=true`). Remaining gap: full-text index update + derived-index completion (e.g. OCR) are not yet included in the ACK contract (Spec §6.2.1). |
| 4.4 | Idempotent chunk writes with dedup key | P1 | `[x]` | `(collector_id, stream_id, session_id, offset)` used |
| 4.5 | Catalog table for origin registry | P1 | `[x]` | `catalog` table avoids `INFO FOR DB` |
| 4.6 | Per-modality typed schema with time index | P1 | `[x]` | `schema.rs` creates SCHEMAFULL tables + `_ts_idx` |
| 4.7 | Remove hardcoded DB credentials | P1 | `[x]` | `server/src/server.rs` now reads `LIFELOG_DB_USER` / `LIFELOG_DB_PASS` from env (no hardcoded creds). |
| 4.8 | Populate `ServerState.total_frames_stored` and `disk_usage_bytes` | P2 | `[ ]` | Proto fields added but not wired to queries |

---

## 5. Query Engine (Spec §10) — P0

This is the **core differentiator** of the product and the biggest gap.

| # | Task | Status | Notes |
|---|------|--------|-------|
| 5.1 | Query AST with boolean logic + stream selectors | `[x]` | `server/src/query/ast.rs` — complete |
| 5.2 | `Contains`, `Eq`, `TimeRange` operators compile to SQL | `[x]` | `planner.rs` generates SurrealDB SQL |
| 5.3 | Origin resolution from catalog | `[x]` | Planner resolves `StreamSelector` → `DataOrigin` list |
| 5.4 | **Implement `WITHIN(A, B, ±Δt)` operator** | `[~]` | Implemented as a two-stage plan (source timestamps -> target time-window filter). Current limits: one WITHIN term, AND-only, no nested temporal ops inside predicate. |
| 5.5 | **Implement `DURING(A, predicate)` operator** | `[~]` | Implemented as a two-stage plan (source intervals -> target time-window filter) with configurable expansion window for point sources. Supports multiple `DURING(...)` terms under `AND` by intersecting interval sets. Now uses interval-target overlap semantics via `t_end` so `DURING(Audio, pointPredicate)` includes overlapping chunks. |
| 5.6 | **Implement `OVERLAPS(intervalA, intervalB)` operator** | `[x]` | Implemented in AST + LLQL + planner + executor (currently planned/executed like `DURING(...)`). |
| 5.7 | **Implement replay queries** | `[~]` | Backend now exposes a `Replay` RPC returning ordered steps (screen-granularity) with aligned context keys. UI integration pending. |
| 5.8 | Replay semantics: point record `t_i` → interval `[t_i, t_{i+1})` | `[~]` | Implemented via `lifelog-core` replay step assembly; last-step end currently uses replay window end. |
| 5.9 | Query resource limits (timeouts, max results) | `[x]` | Added default server-side limits: `LIMIT 1000` on UUID-returning queries and a `10s` SurrealDB query timeout in the query executor (Spec §10.1). |
| 5.10 | User-facing query syntax (DSL or templates) | `[~]` | **LLQL JSON** supported via `Query.text` prefix (`llql:`/`llql-json:`) → typed AST (WITHIN/DURING). Still missing: “nice” human DSL, templates, and UI builder. |

### The Canonical Example That Must Work (Spec §10.2)

```
Retrieve audio during times when:
  - browser URL contains "youtube"
  - OCR text contains "3Blue1Brown"
```

**Current status: PARTIAL.** The backend supports multi-term `DURING(...)` via interval assembly + intersection, and LLQL JSON can express the canonical example. Remaining gaps: UI/DSL authoring, and end-to-end verification with real Audio capture + OCR-derived stream data. This requires:
1. Query browser stream for records where URL contains "youtube" → get time windows
2. Query OCR stream for records where text contains "3Blue1Brown" → get time windows
3. Intersect the two time window sets
4. Query audio stream for chunks overlapping the intersection
5. Return audio chunks (or clipped segments)

---

## 6. Transforms (Spec §9) — P1

| # | Task | Status | Notes |
|---|------|--------|-------|
| 6.1 | OCR transform (screen → text) | `[x]` | Tesseract-based, watermark-scheduled |
| 6.2 | Transform watermark tracking | `[x]` | `watermarks` table, `last_timestamp` cursor |
| 6.3 | Incremental cursors (no full-table scan) | `[~]` | Watermark query is `WHERE timestamp > $watermark`, but legacy set-diff pattern may remain |
| 6.4 | Transform versioning (version field on derived records) | `[ ]` | Spec §9.2: changing OCR model → new version marker |
| 6.5 | Lineage pointers (`source_record_id` on derived records) | `[ ]` | Spec §8.1: derived records must point to source |
| 6.6 | Idempotent & resumable transforms | `[~]` | Watermark not transactionally updated with derived writes |

---

## 7. Security (Spec §12) — P1

| # | Task | Status | Notes |
|---|------|--------|-------|
| 7.1 | TLS for collector ↔ server | `[~]` | Opt-in TLS via env vars exists (Phase 5 work), but not enforced or tested |
| 7.2 | TLS for UI ↔ server | `[~]` | Same opt-in mechanism |
| 7.3 | Secure enrollment/pairing (not MAC-based) | `[ ]` | Currently MAC address ID. Need pre-shared token, QR, or mTLS cert issuance. |
| 7.4 | Authentication on all RPCs | `[ ]` | All endpoints unauthenticated |
| 7.5 | Authorization (who can query/config/data) | `[ ]` | Single-user system, but API should still verify caller |
| 7.6 | Remove hardcoded DB credentials from source | `[ ]` | `root/root` in `server.rs` |
| 7.7 | Disable gRPC reflection in production | `[ ]` | Or gate behind auth |
| 7.8 | At-rest encryption for sensitive data | `[ ]` | OS full-disk encryption at minimum; app-level recommended |
| 7.9 | Per-stream disable (config-driven) | `[x]` | Works via config |
| 7.10 | Emergency pause/resume | `[x]` | UI global pause → broadcasts to collectors |
| 7.11 | Retention controls (coarse-grained TTL/deletion) | `[ ]` | Spec §12.5: default forever, but user must be able to configure |
| 7.12 | Keystroke security controls (before enabling module) | `[ ]` | Spec §12.4: transport encryption + enrollment + at-rest protection mandatory |

---

## 8. Interface / Web UI (Spec §11) — P1

| # | Task | Status | Notes |
|---|------|--------|-------|
| 8.1 | Tauri + React + gRPC-Web stack | `[x]` | Working, gRPC-Web layer on server |
| 8.2 | Search dashboard | `[x]` | Exists, migrated to gRPC |
| 8.3 | Devices dashboard | `[x]` | Shows connected collectors |
| 8.4 | **Timeline navigation** (jump by time, filter by modality/stream/device) | `[~]` | `TimelineDashboard` component exists (Phase 5) but unclear how complete |
| 8.5 | **Replay view** (step-by-step frames + aligned audio + events) | `[ ]` | Core v1 feature. Needs replay query backend (§5.7) first. |
| 8.6 | Result previews (screen thumbnails, OCR snippets, URL snippets) | `[ ]` | Search returns keys, no rich previews |
| 8.7 | Query language UI (templates + builder → DSL advanced view) | `[ ]` | Spec §18.7: progressive disclosure |
| 8.8 | Query cancellation in UI | `[ ]` | Spec §11.2: cancel on new query, navigate away, or overload |
| 8.9 | Multi-device health/backlog visualization | `[ ]` | Devices dashboard exists but no per-device metrics |
| 8.10 | Privacy controls surface (pause, per-stream toggles, retention, deletion) | `[~]` | Global pause exists; no per-stream toggle UI, no retention UI |
| 8.11 | Mobile-responsive design | `[ ]` | Spec §11: must work on phone browser (secondary) |

### Frontend Infrastructure Gaps (from remaining-work.md)

| # | Task | Status |
|---|------|--------|
| 8.12 | Fix pre-existing TS build errors (34+ TS6133 from `noUnusedLocals`) | `[ ]` |
| 8.13 | Set up Vitest + testing-library/react | `[ ]` |
| 8.14 | Wire stubbed dashboard functions to gRPC (Camera, Mic, Processes, Settings) | `[ ]` |

---

## 9. Observability & Operations (Spec §14) — P2

| # | Task | Status | Notes |
|---|------|--------|-------|
| 9.1 | Collector connectivity status on server | `[x]` | `ControlStream` tracks connected collectors |
| 9.2 | Backlog size per collector/stream | `[ ]` | `upload_lag_bytes` proto field added but not populated |
| 9.3 | Last successful pull time | `[~]` | `last_seen` field added in proto, may not be populated |
| 9.4 | Ingest error rates | `[ ]` | No metrics/counters |
| 9.5 | Transform backlog and completion | `[ ]` | Watermark exists but not exposed as a metric |
| 9.6 | Storage usage (metadata vs blobs) | `[ ]` | `disk_usage_bytes` proto field added but not populated |
| 9.7 | Collector: buffer fullness | `[~]` | `CollectorState` reports buffer info |
| 9.8 | Collector: last capture timestamps per stream | `[ ]` | Not tracked per-stream |
| 9.9 | Systemd service units | `[x]` | `deploy/*.service` + justfile recipes |

---

## 10. Performance (Spec §13) — P2

| # | Task | Status | Notes |
|---|------|--------|-------|
| 10.1 | Performance test suite exists | `[x]` | `server/tests/performance_suite.rs` |
| 10.2 | Define numeric SLA targets | `[ ]` | Spec §13.1: "target SLA set by performance suite" — targets undefined |
| 10.3 | Ingestion throughput test (multi-GB uploads) | `[~]` | Test exists but targets unknown |
| 10.4 | Time-range scan performance test | `[~]` | Same |
| 10.5 | Text search latency test | `[ ]` | Can't test — text search index doesn't exist |
| 10.6 | Replay assembly latency test | `[ ]` | Can't test — replay not implemented |

---

## 11. Protocol & Config Cleanup (Spec §16.3) — P2

| # | Task | Status | Notes |
|---|------|--------|-------|
| 11.1 | Eliminate JSON-string config payload in `ServerCommand` | `[ ]` | Use typed proto field instead of `string payload` |
| 11.2 | Stable IDs: ensure `collector_id`, `stream_id`, `session_id`, `record_id` are well-defined | `[~]` | Most exist but `stream_id` semantics could be tighter |
| 11.3 | Remove `unsafe static mut` in collector microphone module | `[ ]` | Tracked in remaining-work.md — use `OnceLock` pattern |
| 11.4 | Replace `println`/`eprintln` with `tracing` in collector modules | `[~]` | Done for main files, module files remain |

---

## 12. Testing Gaps — P2

| # | Task | Status | Notes |
|---|------|--------|-------|
| 12.1 | Cross-modal query integration test | `[ ]` | `cross_modal_query.rs` exists but can't pass with stubbed operators |
| 12.2 | Clock skew integration test (multi-device time correction) | `[ ]` | Skew algorithm tested in isolation, not end-to-end |
| 12.3 | Replay query test | `[ ]` | No replay implementation to test |
| 12.4 | TLS end-to-end test | `[ ]` | TLS mechanism exists but no test |
| 12.5 | Text search index test | `[ ]` | Index doesn't exist yet |
| 12.6 | Blob CAS separation test | `[ ]` | CAS works in isolation, not tested with ingest pipeline |
| 12.7 | E2E/Playwright UI tests | `[ ]` | Deferred per remaining-work.md |

---

## Priority Summary

### P0 — Launch Blockers (system doesn't fulfill its purpose without these)

1. **Cross-modal correlation operators** (§5.4, §5.5, §5.6) — The canonical query example can't run
2. **Replay queries** (§5.7, §5.8) — Core recall feature
3. **Blob separation to CAS** (§4.1) — DB will bloat and degrade with inline blobs
4. **Text search indexes** (§4.2) — Full-text search across OCR/URL/clipboard is core recall
5. **Clock skew integration** (§1.1–§1.5) — Multi-device time correlation is broken without it

### P1 — Required for v1 (spec mandates, high user impact)

6. Missing collector modules: audio, keystrokes, clipboard, shell, window activity, mouse (§2.7–§2.12)
7. Security: TLS enforcement, enrollment/pairing, auth on RPCs, remove hardcoded creds (§7.1–§7.8)
8. Replay UI (§8.5) — needs replay backend first
9. Timeline navigation UI (§8.4)
10. Result previews in search (§8.6)
11. Transform versioning + lineage (§6.4, §6.5)
12. Query DSL/template UI (§8.7)
13. Retention controls (§7.11)

### P2 — Should-have (quality, reliability, spec compliance)

14. Observability metrics (§9.2–§9.6)
15. Performance SLA targets (§10.2)
16. Config cleanup: eliminate JSON-string payload (§11.1)
17. Sequence numbers separate from upload offset (§3.4)
18. Query resource limits/timeouts (§5.9)
19. Upload cancellation + backpressure signals (§3.5, §3.6)
20. Frontend infra: fix TS errors, add tests, wire stubs (§8.12–§8.14)

### P3 — Deferred (explicitly out of scope or low urgency for v1)

21. Vector/embedding indexes (reserved in schema, not required)
22. Audio transcription transform (only OCR required for v1)
23. Natural language → query (GitHub issue #12, not in spec as v1 requirement)
24. E2E Playwright tests (deferred until UI stabilizes)
25. Cold tier / Parquet export (future per spec §8.2.1)
26. Phone/wearable collector implementations (§3.2: "v1-ready but not required day 1")

---

## Suggested Execution Order

The dependency graph suggests this sequence:

```
Phase A: Data Foundation
  1. Add time fields to proto (t_device, t_ingest, t_canonical, time_quality)
  2. Integrate clock skew estimation into server ingest
  3. Separate blobs to CAS in ingest pipeline
  4. Add text search indexes to SurrealDB schema
  5. Fix durable ACK to wait for index completion

Phase B: Query Engine (the hard part)
  6. Implement WITHIN operator (temporal proximity join)
  7. Implement DURING operator (predicate-gated time windows)
  8. Implement OVERLAPS operator (interval intersection)
  9. Implement replay query mode with frame-stepping semantics
  10. Add query DSL parser (user text → AST)

Phase C: Collector Expansion
  11. Clipboard module
  12. Shell history module
  13. Desktop audio module
  14. Window activity module (generic, not just Hyprland)
  15. Mouse events module
  16. Keystrokes module (AFTER security controls in Phase D)

Phase D: Security Hardening
  17. Enforce TLS (not opt-in)
  18. Implement enrollment/pairing (pre-shared token or mTLS)
  19. Add auth middleware to all RPCs
  20. Remove hardcoded DB credentials
  21. Implement retention controls

Phase E: Interface
  22. Timeline navigation (time scrubber, modality filter, device filter)
  23. Replay view (frame stepper + aligned events + audio)
  24. Rich search results (thumbnails, snippets, previews)
  25. Query builder UI (templates → DSL advanced view)
  26. Privacy controls surface (per-stream toggle, retention config)
  27. Mobile-responsive layout

Phase F: Polish
  28. Transform versioning + lineage pointers
  29. Observability metrics population
  30. Performance SLA definition + enforcement
  31. Config cleanup (typed proto payloads)
  32. Frontend test suite
```

---

## Spec Section Cross-Reference

| Spec Section | This Doc | Completion |
|--------------|----------|------------|
| §3 Modalities | §2 | 50% (6/12 modules) |
| §4 Time Model | §1 | 15% (algorithm exists, not integrated) |
| §5 Architecture | §3 | 80% (NAT-safe, buffering, resume work) |
| §6 Reliability | §4 | 60% (ACK incomplete, text index missing) |
| §7 Protocol | §3, §11 | 70% (RPCs work, config cleanup needed) |
| §8 Storage | §4 | 60% (SurrealDB works, blobs inline, no text search) |
| §9 Transforms | §6 | 55% (OCR works, no versioning/lineage) |
| §10 Query | §5 | 30% (AST done, execution stubbed) |
| §11 Interface | §8 | 35% (search+devices, no timeline/replay) |
| §12 Security | §7 | 20% (opt-in TLS, no auth/enrollment) |
| §13 Performance | §10 | 40% (suite exists, no targets) |
| §14 Observability | §9 | 30% (basic state reporting, no metrics) |
