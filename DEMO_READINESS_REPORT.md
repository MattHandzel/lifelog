# Lifelog: Demo Readiness & Quality Report

## Current Status: UNSTABLE (Integrating)

### 1. Detected Friction Points (The "User Experience" Bugs)
*   **Reserved Keyword Conflict**: The initial PostgreSQL migration failed because `offset` is a reserved keyword.
    *   *Fix*: Quoted `"offset"` in the SQL schema.
*   **Infrastructure Overhead**: Server currently requires a running SurrealDB instance even if Postgres is the primary ingest backend.
    *   *Improvement Goal*: Make SurrealDB optional if `LIFELOG_POSTGRES_INGEST_URL` is provided.
*   **Screenshot Dependency**: The default configuration assumed `gnome-screenshot`, which is not standard on NixOS/Wayland.
    *   *Fix*: Updated default to `grim` for the `matts-laptop` profile.
*   **Connectivity Defaults**: The collector was too aggressive in defaulting to `localhost`, ignoring environment overrides in some scenarios.

### 2. Implementation Tracker
- [x] **Non-interactive Pairing**: Added `--yes` flag to `lifelog join`.
- [x] **Postgres Migration Fix**: Quoted reserved keywords in `20260303143000_init_postgres.sql`.
- [ ] **Config Hardening**: Update `matts-laptop` configuration to use `grim`.
- [ ] **Data Flow Verification**: Confirm `matts-laptop` screen data is appearing in the remote Postgres.

---

## 3. Success Metrics for "High Quality"
1.  **Zero Manual SQL**: User should not have to run `psql` to create tables.
2.  **Auto-Discovery**: Collector should find the server via the provided URL without config file surgery.
3.  **Resilient Capture**: If one screenshot tool fails, the software should warn rather than crash.
