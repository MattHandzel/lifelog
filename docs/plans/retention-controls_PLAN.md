# Plan: Retention Controls (Data Lifecycle)

## Objective
Enable users to manage their storage footprint by setting automatic deletion policies.

## Phase 1: Backend (Server)
1.  **Config Schema:** Add `retention_policies` to `ServerConfig` (e.g., `screen: 7d`, `audio: 30d`, `text: forever`).
2.  **TTL Worker:** Implement a background task in `server/src/server.rs` that runs daily.
3.  **Pruning Logic:**
    - Delete SurrealDB records older than TTL.
    - Reference-count CAS blobs and delete orphaned files.

## Phase 2: Interface (UI)
1.  **Settings Update:** Add a "Privacy & Storage" section to `SettingsDashboard.tsx`.
2.  **RPC Wiring:** Ensure `set_component_config` can update these policies live.

## Phase 3: Verification
1.  Seed data with old timestamps and verify the worker prunes it correctly.
