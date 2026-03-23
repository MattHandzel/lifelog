# Lifelog: High-Fidelity Autonomous Integration Plan (Level 3)

**Objective**: Establish a secure, PostgreSQL-backed gRPC stream from the laptop (`matth@nixos`) to the remote server (`192.168.0.177`) and verify end-to-end data integrity autonomously.

---

## 1. System Architecture & Context
*   **Remote Server (Sink)**: Runs `lifelog-server` on `192.168.0.177:7182`.
*   **Laptop Client (Source)**: Runs `lifelog-collector` and `interface`.
*   **Database**: PostgreSQL 16+ (Local to Server).
*   **Security**: Mandatory TLS (Self-signed) + Bearer Token Auth + Pairing Handshake.

---

## 2. Autonomous Validation Suite (Success Criteria)

### V1: Environment Parity (Sync Check)
*   **Command**: `[[ $(git rev-parse HEAD) == $(ssh matth@nixos "cd ~/Projects/lifelog && git rev-parse HEAD") ]]`
*   **Metric**: Git hashes must be identical.

### V2: The "Join" Handshake (Pairing)
*   **Action**: Laptop Agent executes `lifelog join`.
*   **Metric**: Server's `lifelog-config.toml` contains the laptop's `collector_id`.

### V3: Ingestion Throughput (The 60s Pulse)
*   **Action**: Query Postgres `screen_records` count at $T=0$ and $T=60s$.
*   **Metric**: `Count(T=60) - Count(T=0) >= 10` (Assuming 5s capture interval).

### V4: BLOB/CAS Verification
*   **Action**: Extract `blob_hash` from the latest Postgres record; verify file existence on server disk.
*   **Metric**: `test -f /home/matth/lifelog/cas/<hash>` returns 0.

### V5: UI/RPC Loopback (The Interface Test)
*   **Action**: Laptop Agent runs a headless Playwright script to verify the React app can fetch and render a screenshot from the Remote Server.
*   **Metric**: `<img>` tag detected with valid `src` blob URL.

---

## 3. Execution Phases

### Phase 1: Server Hardening (Orchestrator)
1.  Initialize PostgreSQL `lifelog` database and user.
2.  Generate/Verify `cert.pem` and `key.pem`.
3.  Launch Server with `LIFELOG_POSTGRES_INGEST_URL` and `LIFELOG_ENROLLMENT_TOKEN`.

### Phase 2: Client Trust & Hand-off (Laptop Agent)
1.  Spawn **Codex Sub-agent** on Laptop via SSH.
2.  Inject `server_cert.pem` into Laptop environment.
3.  Execute `Join` protocol to establish secure identity.

### Phase 3: Active Stream & Ingest (Autonomous Monitoring)
1.  Start Laptop Collector.
2.  Orchestrator monitors Postgres `screen_records` and `upload_chunks` tables.
3.  Verify "ACK Gate" logic (chunks are marked `indexed=true` after persistence).

### Phase 4: Interface Integration (UI Agent)
1.  Start Interface Vite dev server on Laptop.
2.  Execute UI Smoke Test via Playwright.

---

## 4. Failure Recovery & Rollbacks
- **Port Conflict**: `lsof -ti:7182 | xargs kill -9`
- **DB Conflict**: `DROP DATABASE lifelog; CREATE DATABASE lifelog;`
- **Auth Failure**: Regenerate `LIFELOG_AUTH_TOKEN` and re-pair.
