# STATUS_TODOLIST.md

Last updated: 2026-02-10

## P0 (Launch Blockers)

- `[x]` Time model wire fields on frames (Spec §4.2.1)
  - Frame protos include `t_device`/`t_ingest`/`t_canonical`/`t_end`/`time_quality`
  - Server populates these fields in `GetData` responses for ingested modalities
- `[x]` Clock skew estimation wired (Spec §4.2.1)
  - Server periodically sends `ClockSync` over `ControlStream`
  - Collector replies with `ClockSample { device_now, backend_now }`
  - Server maintains per-collector skew estimates and applies them at ingest (`t_canonical`, `time_quality`)
- `[x]` Default correlation window config (Spec §4.3)
  - Added `ServerConfig.default_correlation_window_ms`
  - LLQL JSON may omit `window` and rely on the server default (per-term non-zero window overrides)
- `[x]` Point vs interval semantics on frames (Spec §4.1)
  - Frame protos include `record_type` (`Point` vs `Interval`)
  - Server sets `record_type` in `GetData` responses (Audio/WindowActivity = interval; others = point)
- `[x]` Blobs stored in CAS, not inline (Spec §6 / §8)
  - Screen/Camera/Audio blobs stored in CAS (`blob_hash` + `blob_size`)
  - Clipboard binary payloads (when present) are stored in CAS; SurrealDB stores only the reference
- `[x]` Durable ACK implies queryable (Spec §6.2.1)
  - `UploadChunks` ACK only advances when the backend marks the chunk queryable (`upload_chunks.indexed=true`)
  - Screen ingestion pins ACK until the OCR-derived record for the same frame UUID has been persisted, then marks the chunk indexed/queryable.
- `[x]` Query resource limits (Spec §10.1)
  - Default `LIMIT 1000` on UUID-returning queries
  - Default `10s` SurrealDB query timeout in the query executor
- `[~]` Query correlation operators
  - `[~]` `WITHIN(...)` implemented (two-stage plan; supports multiple terms; can mix with `DURING`/`OVERLAPS` via interval intersection; OR supported via bounded DNF; NOT over temporal ops still unsupported)
  - `[~]` `DURING(...)` implemented (two-stage plan; supports multiple terms via interval intersection; window expansion for point sources; OR supported via bounded DNF; NOT over temporal ops still unsupported)
  - `[x]` `OVERLAPS(...)` implemented (AST + planner + executor; currently planned/executed like `DURING(...)`)
  - `[x]` Interval-target overlap semantics (`t_canonical`/`t_end`) wired so `DURING(Audio, pointPredicate)` includes overlapping chunks
- `[x]` Canonical cross-modal example (Spec §10.2)
  - `DURING(audio, browser URL contains "youtube" AND OCR contains "3Blue1Brown")`
  - Implementation note: backend now supports executing this via LLQL JSON embedded in `Query.text` (`llql:`/`llql-json:`), assuming Audio/Browser/OCR streams exist.
  - Verified end-to-end via ignored integration test (`server/tests/canonical_llql_example.rs`) that seeds Browser/OCR/Audio and executes LLQL JSON through gRPC.
  - Remaining (v1 UX): templates/builder UI, and validation with real collector ingest + OCR transform output.
- `[x]` Replay queries (Spec §10.3)
  - Backend `Replay` RPC returns ordered steps with aligned context keys
  - Interface Replay view is wired to `Replay` (via Tauri gRPC client)

## P1 (v1 Requirements)

- `[x]` Remove hardcoded DB credentials (`server/src/server.rs`) (now requires `LIFELOG_DB_USER`/`LIFELOG_DB_PASS`)
- `[ ]` Security: TLS enforcement + pairing + auth on RPCs
- `[x]` Collector: config hot-reload on `UpdateConfig` command (Spec §7.2)
  - Applies JSON-encoded `CollectorConfig` payload and restarts sources without tearing down the ControlStream
- `[~]` Collector: audio, clipboard, shell, mouse, window activity modules (plus safe defaults)
  - `[x]` Clipboard capture module (polls `wl-paste`/`xclip`/`xsel`; WAL-buffered; server ingest + retrieval wired)
  - `[x]` Shell history capture module (tails history file; zsh extended + bash `HISTTIMEFORMAT` parsing; WAL-buffered; server ingest + retrieval wired)
  - `[x]` Audio capture module (CPAL -> WAV bytes; WAL-buffered as `AudioFrame`; stream_id `audio`)
  - `[ ]` Mouse events module
  - `[~]` Window activity module (Hyprland-only; generic fallback missing)
- `[~]` UI: replay view + query builder/templates + previews
  - `[x]` Replay view
  - `[~]` Query authoring (LLQL mode in Timeline; templates/builder still missing)
