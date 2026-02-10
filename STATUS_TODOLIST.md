# STATUS_TODOLIST.md

Last updated: 2026-02-10

## P0 (Launch Blockers)

- `[x]` Time model wire fields on frames (Spec §4.2.1)
  - Frame protos include `t_device`/`t_ingest`/`t_canonical`/`t_end`/`time_quality`
  - Server populates these fields in `GetData` responses for ingested modalities
- `[x]` Query resource limits (Spec §10.1)
  - Default `LIMIT 1000` on UUID-returning queries
  - Default `10s` SurrealDB query timeout in the query executor
- `[~]` Query correlation operators
  - `[~]` `WITHIN(...)` implemented (two-stage plan; AND-only; single term)
  - `[~]` `DURING(...)` implemented (two-stage plan; AND-only; supports multiple terms via interval intersection; window expansion for point sources)
  - `[x]` `OVERLAPS(...)` implemented (AST + planner + executor; currently planned/executed like `DURING(...)`)
  - `[x]` Interval-target overlap semantics (`t_canonical`/`t_end`) wired so `DURING(Audio, pointPredicate)` includes overlapping chunks
- `[~]` Canonical cross-modal example (Spec §10.2)
  - `DURING(audio, browser URL contains "youtube" AND OCR contains "3Blue1Brown")`
  - Needs multi-stream predicate evaluation + interval intersection + audio overlap semantics
  - Implementation note: backend now supports executing this via LLQL JSON embedded in `Query.text` (`llql:`/`llql-json:`), assuming Audio/Browser/OCR streams exist.
  - Remaining: UI query authoring (templates/builder), and end-to-end validation with real ingest + OCR derived stream.
- `[x]` Replay queries (Spec §10.3)
  - Backend `Replay` RPC returns ordered steps with aligned context keys (UI integration pending)

## P1 (v1 Requirements)

- `[x]` Remove hardcoded DB credentials (`server/src/server.rs`) (now requires `LIFELOG_DB_USER`/`LIFELOG_DB_PASS`)
- `[ ]` Security: TLS enforcement + pairing + auth on RPCs
- `[ ]` Collector: audio, clipboard, shell, mouse, window activity modules (plus safe defaults)
- `[ ]` UI: replay view + query builder/templates + previews
