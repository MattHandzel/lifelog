# STATUS_TODOLIST.md

Last updated: 2026-02-10

## P0 (Launch Blockers)

- `[~]` Query correlation operators
  - `[~]` `WITHIN(...)` implemented (two-stage plan; AND-only; single term)
  - `[~]` `DURING(...)` implemented (two-stage plan; AND-only; supports multiple terms via interval intersection; window expansion for point sources)
  - `[ ]` `OVERLAPS(...)` (AST + planner + executor)
- `[ ]` Canonical cross-modal example (Spec §10.2)
  - `DURING(audio, browser URL contains "youtube" AND OCR contains "3Blue1Brown")`
  - Needs multi-stream predicate evaluation + interval intersection + audio overlap semantics
  - Implementation note: backend now supports executing this via LLQL JSON embedded in `Query.text` (`llql:`/`llql-json:`), assuming Audio/Browser/OCR streams exist.
- `[ ]` Replay queries (Spec §10.3)
  - Backend query mode returning ordered replay steps with aligned context

## P1 (v1 Requirements)

- `[ ]` Remove hardcoded DB credentials (`server/src/server.rs`)
- `[ ]` Security: TLS enforcement + pairing + auth on RPCs
- `[ ]` Collector: audio, clipboard, shell, mouse, window activity modules (plus safe defaults)
- `[ ]` UI: replay view + query builder/templates + previews
