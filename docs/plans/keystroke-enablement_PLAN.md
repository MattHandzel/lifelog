# Plan: Keystroke Enablement

## Objective
Enable the Keystroke capture module by default and ensure it is functional across environments.

## Phase 1: Collector Enablement
1.  **Config Default:** Update `config/src/lib.rs` to enable the keystroke module by default in the base template.
2.  **Module Check:** Verify `collector/src/modules/keystrokes.rs` properly captures events via `rdev`.
3.  **No Gating:** Ensure there are no code-level checks that prevent the module from running if TLS is disabled (per user instruction).

## Phase 2: Server Ingest
1.  **Index Verification:** Ensure the `keystrokes` table in SurrealDB has the correct BM25 indexes for searching.

## Phase 3: Verification
1.  Run collector, type some text, and verify it appears in the Search/Timeline UI.
