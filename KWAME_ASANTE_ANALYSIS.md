# Kwame Asante Security & Privacy Launch Audit (Lifelog)

This review treats Lifelog as a pre-launch system that continuously captures and centralizes highly sensitive personal data (microphone audio, screen captures/OCR, browser history/URLs). I read `SPEC.md` and then inspected the current implementation in `server/`, `collector/`, `interface/`, and `proto/` to determine what is actually enforced today.

## Executive Summary (Launch Blockers)

The current repository state is not safe to ship as a product that captures microphone/screen/URLs across devices.

Primary launch blockers:

1. **No authenticated + encrypted transport for device/server/UI traffic** (gRPC uses plain `http://...`).
2. **No secure enrollment/pairing model**: any device can register, spoof identity, and potentially participate in ingestion/control.
3. **No encryption-at-rest for captured buffers/log files** on collectors (e.g., WAV files) and no application-level protection for backend storage.
4. **No real access control** on query/data APIs: `Query`/`GetData`/`GetConfig`/`SetConfig` are unauthenticated.
5. **Hard-coded database root credentials** in server code (`root`/`root`).

`SPEC.md` requires TLS and pairing (Section 12), but these are not implemented in the current code paths.

## Scope & Method

* Spec review: `SPEC.md` (Security & Privacy section and architecture invariants).
* Code review targets:
  * gRPC server and collector transports: `server/src/main.rs`, `server/src/server.rs`, `collector/src/main.rs`, `collector/src/collector.rs`.
  * Data-at-rest handling on collectors: `collector/src/modules/microphone.rs`, `collector/src/modules/screen.rs`, `collector/src/setup.rs`, and other module output patterns.
  * UI network use and auth expectations: `interface/src/lib/api.ts`, `interface/src/components/Login.tsx`, `interface/src-tauri/src/main.rs`, `interface/src-tauri/src/bin/lifelog-server.rs`.
  * Protocol surface: `proto/lifelog.proto`, `proto/lifelog_types.proto`.

I did not run the system end-to-end; findings are based on static inspection of what would execute if built from this repo.

## Threat Model (What You Must Defend Against)

Assume adversaries with one or more of:

* **Local network attacker**: can connect to open ports, spoof traffic, MITM plaintext connections.
* **Malicious device**: a compromised laptop/VM that pretends to be a collector or UI client.
* **Host compromise**:
  * attacker obtains read access to collector disks (stolen device, malware, backup leak),
  * attacker obtains access to backend DB files/process (local user compromise),
  * attacker obtains access to UI runtime (XSS, local malware).
* **Accidental exposure**: binding services on 0.0.0.0, leaving default passwords, logging secrets to stdout.

Given the data types involved (audio, screen, URLs, keystrokes/clipboard in the spec), the risk profile is equivalent to a full surveillance stack. The security bar must be closer to password manager / E2EE messenger than a typical hobby app.

## Findings

### Critical

#### C1. No Encryption In Transit (TLS/mTLS not implemented)

Evidence:

* Collector connects to server with a plaintext URL by default: `collector/src/main.rs` (`--server-address` default is `http://127.0.0.1:7182`).
* Collector handshake uses `tonic::transport::Endpoint::from_shared(self.server_address.clone())` with no TLS config: `collector/src/collector.rs`.
* Server listens with `TonicServer::builder().serve(addr)` and does not configure TLS: `server/src/main.rs`.
* Server connects back to collectors over plaintext: `http://{collector_host}:{collector_port}` in `register_collector`: `server/src/server.rs`.
* Interface Tauri client uses `const GRPC_SERVER_ADDRESS: &str = "http://localhost:7182";`: `interface/src-tauri/src/main.rs`.

Impact:

* On any non-localhost deployment (even “just the LAN”), an attacker can sniff and modify gRPC traffic: screen bytes, OCR text, browser metadata, and control/config operations.
* Token-based auth (even if added later) would be trivially stolen if run over HTTP.

Prescriptive mitigations:

* Implement TLS everywhere as a hard requirement:
  * For gRPC: configure `tonic::transport::ServerTlsConfig` and `ClientTlsConfig` (rustls).
  * Refuse to start in “multi-device” mode without TLS.
* Strong recommendation: **mTLS for collector<->backend** with device certificates issued at enrollment.
* Pin the server CA (or SPKI pin) on collectors to prevent transparent MITM on “trusted” networks.
* Disable gRPC reflection in production (see H3) or ensure reflection is behind auth.

#### C2. No Secure Enrollment/Pairing; Device Identity Is Spoofable

Evidence:

* Enrollment is effectively “anyone can register”:
  * `RegisterCollectorRequest` contains only `CollectorConfig` in `proto/lifelog.proto`.
  * `server/src/server.rs` `register_collector` trusts `CollectorConfig.host`, `CollectorConfig.port`, and `CollectorConfig.id`.
* Collector derives its ID from a MAC address and sends it; the server does not verify it:
  * Collector normalizes MAC and sets `config_for_registration.id = mac_addr_string`: `collector/src/collector.rs`.
  * Server uses `collector_config.id` as the collector identity: `server/src/server.rs`.

Impact:

* A malicious device can impersonate a collector ID and race/replace registration.
* A malicious device can point `host:port` to an attacker-controlled gRPC endpoint; the server will dial it and treat it as the collector.
* Any authorization logic based on `collector_id` becomes meaningless.

Prescriptive mitigations:

* Implement a real pairing model (one of):
  * One-time pairing QR code / short-lived enrollment token generated on the backend and entered on collector.
  * mTLS with backend-issued device certificates (recommended).
* Bind identity to cryptographic proof, not MAC address:
  * device keypair generated on device,
  * backend issues cert binding public key to `collector_id`,
  * collector proves possession on every connection.
* Add revocation and rotation:
  * server maintains allowlist of enrolled devices,
  * revoke device certs/tokens from UI.

#### C3. Sensitive Data Is Not Encrypted At Rest (Collectors + Backend)

Evidence (collector):

* Microphone data is recorded to plaintext `.wav` files: `collector/src/modules/microphone.rs` (`WavWriter::create(&output_path, spec)`).
* Screen capture path writes PNG to a real filesystem path (then reads and deletes it): `collector/src/modules/screen.rs` (`out = "{output_dir}/{timestamp}.png"`, `tokio::fs::read`, `tokio::fs::remove_file`).
  * Deleting a file is not secure erasure; the bytes may persist on disk/SSD wear-leveling.
* Multiple SQLite DBs are created as plaintext (`Connection::open(db_path)`): `collector/src/setup.rs`.

Evidence (backend):

* Screen bytes are stored directly in SurrealDB rows (no blob separation, no encryption wrapper): `server/src/server.rs` (`ScreenFrameSurreal.image_bytes: surrealdb::sql::Bytes`).

Impact:

* Stolen device or malware can immediately read raw audio, screenshots, OCR text, URLs, etc.
* Backups/snapshots (even “local”) become high-risk exfil vectors.

Prescriptive mitigations:

* Minimum bar: require OS full-disk encryption and document it, but do not stop there.
* Implement application-level encryption for buffered content:
  * per-record/per-chunk AEAD (XChaCha20-Poly1305 or AES-256-GCM),
  * keys stored in OS keychain/TPM (macOS Keychain, Windows DPAPI, Linux Secret Service/TPM2).
* Separate metadata from blobs as `SPEC.md` requires; encrypt blobs and store only ciphertext + integrity tag.
* Consider searchable encryption carefully:
  * do not “encrypt” OCR text and then add a plaintext full-text index beside it,
  * if you need search, design a threat model for the index (or accept plaintext index with explicit user-facing warnings and strong at-rest protections).

#### C4. Hard-Coded DB Root Credentials in Server

Evidence:

* `server/src/server.rs` signs into SurrealDB with:
  * `username: "root"`,
  * `password: "root"`.

Impact:

* Any process/user that can reach the DB endpoint can attempt default credentials.
* Root credentials maximize blast radius: full read/write of all modalities and tables.

Prescriptive mitigations:

* Remove hard-coded credentials immediately.
* Use config + secret management:
  * generate random DB password on first run,
  * store in OS keychain / secure local secret store,
  * never print it.
* Create least-privilege DB users (separate ingest vs query vs admin).

### High

#### H1. Unauthenticated Query/Data/Config APIs Enable Full Data Exfiltration

Evidence:

* gRPC methods in `proto/lifelog.proto` include:
  * `GetConfig`, `SetConfig`, `GetData`, `Query`, `ReportState`, `GetState`.
* `server/src/server.rs` implements these methods without any auth checks (beyond a weak “is collector registered” check for `ReportState` only).
* `Query` currently ignores structured query constraints and returns keys across all origins: `server/src/server.rs` (`process_query(String::from(""))` then `get_all_uuids_from_origin` per table).

Impact:

* Anyone who can connect to the gRPC port can:
  * enumerate keys and retrieve raw screen bytes (and later audio, URLs, etc.),
  * pull full system config (collector ports, hostnames, output dirs),
  * attempt to mutate config (`SetConfig` path is stubbed but will become high risk once implemented).

Prescriptive mitigations:

* Implement authn/authz at the transport layer:
  * mTLS for collectors,
  * separate client auth for UI (see H4).
* Add authorization policy:
  * per-modality access controls (microphone/screen should be “explicitly granted”),
  * separate “admin” from “read-only” roles,
  * audit log for every query that touches sensitive modalities.
* Implement resource limits required by `SPEC.md` (timeouts, max results, max bytes per response).

#### H2. Query Construction Has Injection Footguns (SurrealDB + SQLite)

Evidence:

* SurrealDB query string is built from interpolated table name:
  * `let sql = format!("SELECT VALUE record::id(id) as uuid FROM `{table}`"); //FIX: Sql injection 🤡`
  * `server/src/server.rs`.
* The legacy CLI “server” in Tauri sources builds raw SQL from a user-supplied WHERE clause:
  * `let sql = format!("SELECT * FROM {} WHERE {}", table_name, query);`
  * `interface/src-tauri/src/bin/lifelog-server.rs`.

Impact:

* If any of these code paths become reachable from an untrusted UI/API input, it becomes trivial to:
  * bypass intended filtering,
  * query arbitrary tables,
  * potentially trigger expensive queries (DoS),
  * depending on DB semantics, mutate data.

Prescriptive mitigations:

* Treat all query strings as untrusted input. Do not accept raw SQL filters from UI.
* Use a typed query AST and compile to safe DB operations (aligned with `SPEC.md` query language goals).
* Validate/allowlist table/origin names:
  * don’t accept arbitrary “origin” strings from clients,
  * use server-side resolved IDs.

#### H3. gRPC Reflection Enabled (Information Disclosure)

Evidence:

* Server registers tonic reflection service: `server/src/main.rs`.
* Collector registers tonic reflection service: `collector/src/main.rs`.

Impact:

* Reflection significantly lowers the cost of probing and building exploit tooling against your API surface.

Prescriptive mitigations:

* Disable reflection in production builds by default (feature-gate it).
* If kept for dev, bind to localhost only and/or gate it behind mTLS/auth.

#### H4. UI Auth Model Is Confused; Demo Credentials and Insecure Token Storage Patterns

Evidence:

* UI includes hard-coded demo creds (`admin/admin`) in the login form state: `interface/src/components/Login.tsx`.
* UI stores JWT in `localStorage`: `interface/src/lib/api.ts`.
* `server/README.md` documents a REST API with JWT/CORS, but the Rust `lifelog-server` binary in `server/src/main.rs` is a gRPC server and does not implement those REST endpoints.

Impact:

* Hard-coded credentials invite “it shipped like this” failures.
* JWT in `localStorage` is vulnerable to token theft if any XSS exists in the UI stack (including future additions).
* Architectural drift (REST vs gRPC) often results in accidental exposure: a debug REST server gets revived without the intended auth.

Prescriptive mitigations:

* Decide a single interface API surface (gRPC vs HTTP) and implement auth end-to-end.
* Remove demo credentials from production builds; enforce first-run password setup.
* For desktop (Tauri): store tokens in OS keychain (or Tauri secure storage) and prefer short-lived access tokens + refresh.
* If using cookies on web: use `HttpOnly`, `Secure`, `SameSite=Strict`, CSRF protections.

### Medium

#### M1. Data Lifecycle Controls (Retention, Deletion, “Right to Be Forgotten”) Are Not Implemented

Evidence:

* Spec requires retention controls at least coarse-grained (Section 12.3), but no retention enforcement was found in the reviewed paths.
* Collector modules continuously append to output dirs (`dev-config.toml` points to `/home/matth/lifelog/...`).

Impact:

* Infinite retention dramatically increases breach impact and user harm.

Prescriptive mitigations:

* Implement retention policies per modality + per device:
  * e.g., keep audio 7 days, screen 30 days, URLs 90 days, etc.
* Implement deletion:
  * delete by time range, by modality, by collector, by “selectors” (e.g., “delete all microphone between X and Y”),
  * delete derived data consistently (OCR derived from deleted screenshots must also be deleted).
* Use crypto-erasure if using per-epoch keys:
  * rotate keys per day/week and delete keys to render old ciphertext unrecoverable.

#### M2. Privacy By Design Gaps: Data Minimization, Purpose Limitation, Consent UX

Evidence:

* `SPEC.md` includes extremely sensitive modalities (keystrokes, clipboard, shell history).
* Current dev config has microphone enabled (`dev-config.toml`), and modules write raw outputs without any “consent gate” in code paths.

Impact:

* You risk capturing third-party conversations, credentials, private content with no user awareness or control.

Prescriptive mitigations:

* Explicit consent and clear indicators:
  * first-run enrollment flow that enumerates each stream and requires opt-in,
  * always-visible “recording” indicator for microphone/screen,
  * emergency stop hotkey and “panic pause” (spec mentions pause/resume).
* Default to least collection:
  * ship with microphone disabled unless explicitly enabled,
  * require per-stream enablement and explain consequences.
* Add local redaction rails:
  * “incognito mode” heuristics,
  * ignore password fields and secure input surfaces where feasible,
  * allow user-defined exclude lists (apps/domains/windows).

#### M3. Excessive Debug Logging Risks Leaking Sensitive Data

Evidence:

* Multiple `println!` statements log configs, origins, and operational details (e.g., `server/src/server.rs`, `collector/src/collector.rs`, `interface/src-tauri/src/main.rs`).

Impact:

* Logs become another sensitive dataset, often copied into bug reports or centralized logging.

Prescriptive mitigations:

* Use structured logging with levels; default to `info`/`warn` with redaction.
* Never log raw payloads, tokens, secrets, file contents, or full URLs by default.

### Low

#### L1. MAC Address Used as Stable Device Identifier

Evidence:

* Collector uses MAC address as ID: `collector/src/collector.rs`.

Impact:

* MAC addresses are PII and correlate devices across networks/time.
* MACs are not reliable identity anchors (spoofable; may change with adapters/OS privacy features).

Prescriptive mitigations:

* Use a random device ID generated at install time and bound to device keys/certs.
* If you need hardware binding, do it cryptographically (TPM/secure enclave attestation), not with MAC.

#### L2. Temporary Screen Capture Written to Disk

Evidence:

* Screenshot tool writes to `{output_dir}/{timestamp}.png` before read/delete: `collector/src/modules/screen.rs`.

Impact:

* Deleted files can often be recovered; this increases exposure in disk forensic scenarios.

Prescriptive mitigations:

* Capture directly to memory where possible (platform APIs) or write to tmpfs.
* If disk write is unavoidable, store encrypted immediately and avoid leaving plaintext intermediates.

## Spec vs Reality Gaps (Security-Relevant)

* `SPEC.md` requires TLS and a user-authorized pairing model (Section 12.1–12.2). Current gRPC paths are plaintext and unauthenticated.
* `SPEC.md` requires durable disk buffering on collectors; current `ScreenDataSource` and `BrowserHistorySource` buffer in memory and/or write raw outputs without a secure WAL design (`collector/src/modules/screen.rs`).
* `SPEC.md` calls for “must be safe” query language with bounded resource usage; current `Query` returns “all keys” and has no auth/limits (`server/src/server.rs`).

## Recommended “Secure v1” Baseline (Concrete Checklist)

If you want a realistic path to launch, treat these as required:

1. **mTLS everywhere** for collector<->backend; separate auth for UI clients.
2. **Enrollment/pairing flow** with revocation (QR/token, certificate issuance).
3. **Encryption at rest** for all captured content:
   * encrypt blob payloads (audio/images) and any durable buffer/WAL,
   * keys protected by OS keychain/TPM,
   * adopt crypto-erasure via key rotation.
4. **Authorization**:
   * role-based access for UI,
   * per-modality gating (microphone/screen are “high sensitivity”),
   * comprehensive audit logs (who queried what, when).
5. **Retention + deletion**:
   * enforce retention by default,
   * propagate deletions to derived data and indexes.
6. **Reduce attack surface**:
   * disable reflection in production,
   * bind services to localhost unless explicitly configured with TLS,
   * remove default creds and hard-coded secrets.

## Notes

* `proto/lifelog.proto` appears malformed in `LifelogDataKey` (there is a stray `timestamp` line). Independently of correctness, this is a “paper cut” that tends to push teams toward ad hoc parsing or multiple protocol variants, which increases security risk. Fix protocol definitions early.

