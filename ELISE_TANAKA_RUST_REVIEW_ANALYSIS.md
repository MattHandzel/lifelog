# Elise Tanaka Rust Review: Lifelog

Scope: idiomatic Rust, async Tokio patterns, error handling, performance, concurrency, dependency choices, safety/`unsafe`, organization. This is written as a senior PR review with specific file/line references and concrete rewrites.

Date: 2026-02-05

## Executive Summary

The repository has the rough shape of the v1 system in `SPEC.md` (collector, server, modalities, transforms), but the current implementation is a prototype with several correctness/safety issues and a lot of accidental complexity from `Arc<RwLock<...>>` + `Any` downcasting.

The highest-priority items are:

1. Fix task lifecycle: multiple data sources spawn tasks but the returned `JoinHandle` is a dummy and the real task handle is dropped (`collector/src/modules/screen.rs`, `collector/src/modules/browser_history.rs`).
2. Remove unsound global state (`static mut`) and make system/collector state thread-safe (`server/src/server.rs`, `collector/src/modules/microphone.rs`).
3. Stop holding global locks across `.await` and redesign server/collector concurrency. Current `RwLock` usage is a bottleneck and is deadlock-prone as the code grows (`server/src/server.rs`, `collector/src/collector.rs`).
4. Implement the spec-required durability and resumability: collector buffering is in-memory `Vec`, and upload/ack is not offset-based or durable (`SPEC.md` vs `collector/src/modules/screen.rs`, `collector/src/collector.rs`, `server/src/server.rs`).
5. Eliminate repeated full-buffer clones and “collect all then send” patterns; stream and drain instead (collector get-data path + server ingest path).

## Spec Alignment Notes (SPEC.md vs code)

1. NAT-safe control plane is not implemented.
The spec requires collectors to initiate outbound long-lived control channels, with backend issuing commands over that channel. Current behavior dials collector `host:port` from the server at registration time (`SPEC.md` §5.3, `server/src/server.rs:364-390`, `collector/src/main.rs:111-120` comments).

2. Durable collector buffering is missing.
Spec requires disk-backed append-only WAL/queue and retention by backend acknowledgements (`SPEC.md` §6.1). Current screen buffering is `Arc<Mutex<Vec<ScreenFrame>>>` in memory (`collector/src/modules/screen.rs:31-55`), and server “ack” is implicit by clearing buffers after sending (`collector/src/collector.rs:524-528`), without offsets/checkpoints.

3. Resumable uploads/idempotency keys are missing.
Spec calls out `(collector_id, stream_id, session_id, offset, chunk_hash)` and offset queries (`SPEC.md` §6.3, §6.4, §7.3). Current `GetData` is a “dump everything” stream without offsets and the server inserts rows by UUID with no chunk identity (`collector/src/collector.rs:489-573`, `server/src/server.rs:1096-1167`, `server/src/server.rs:1360-1388`).

4. Blob separation is not implemented.
Spec requires blob store separate from metadata (`SPEC.md` §8.2). Screen frames are stored inline as bytes in SurrealDB rows (`server/src/server.rs:898-905`, `ScreenFrameSurreal.image_bytes`).

5. Query semantics are stubbed.
The server ignores the structured query message and returns essentially “all keys” (`server/src/server.rs:434-466`, `SPEC.md` §10).

## Findings (Ordered By Severity)

### 1) Correctness: DataSource task handle is wrong (tasks become “detached”)

In both screen and browser sources, `start()` spawns a real task (`join_handle`) but returns a brand-new dummy task handle that does nothing.

This breaks:
1. Stop/join lifecycle.
2. Backpressure/cancellation (you cannot cancel/await the real task).
3. Error propagation (panics/errors are lost).

References:
- `collector/src/modules/screen.rs:77-91`
- `collector/src/modules/browser_history.rs:124-138`

Concrete fix (pattern):
```rust
// In start():
let join = tokio::spawn(async move { source_clone.run().await });
Ok(DataSourceHandle { join })
```

Also stop using a global `static RUNNING` (see next item).

### 2) Safety: `static mut` global state is unsound in async/concurrent code

`static mut` is not synchronized; accesses are UB if concurrent, and in this code they are concurrent (Tokio tasks + std threads).

References:
- Server: `static mut SYS: Option<sysinfo::System> = None;` (`server/src/server.rs:244`) used mutably in `get_state` (`server/src/server.rs:637-671`).
- Collector microphone: `static mut CURRENT_CONFIG: Option<Mutex<MicrophoneConfig>> = None;` (`collector/src/modules/microphone.rs:23`) accessed from multiple tasks/threads (`collector/src/modules/microphone.rs:66-96`, `175-180`, `257-262`, `430-438`).

Concrete rewrite options:
1. Put `sysinfo::System` inside `Server` as a normal field (preferred). Then `get_state()` becomes safe and does not require global singleton.
2. Use `once_cell::sync::Lazy<std::sync::Mutex<System>>` if you truly need a global.
3. For microphone config, use `ArcSwap` or `tokio::sync::watch` and pass an `Arc<SharedState>` into the thread. Example shape:
```rust
struct MicShared {
  cfg: arc_swap::ArcSwap<MicrophoneConfig>,
  enabled: AtomicBool,
  paused: AtomicBool,
}
```
Then the blocking loop does `let cfg = shared.cfg.load();`.

### 3) Concurrency: locks held across `.await` in hot paths (will bottleneck and risks deadlocks)

The project uses `Arc<RwLock<...>>` widely, and sometimes holds those locks while awaiting I/O or other async calls.

Collector:
- `CollectorHandle::r#loop()` takes a write lock and awaits `report_state()` and potentially `handshake()` while holding that lock (`collector/src/collector.rs:145-156`, `394-418`, `194-244`). This blocks concurrent RPC handlers (e.g. `get_data`, `get_config`) from accessing collector state and will get worse as you add modalities.

Server:
- `ServerHandle::r#loop()` grabs a write lock on the entire server and awaits `server.step().await` (`server/src/server.rs:165-174` and called from `server/src/main.rs:34-36`). `step()` awaits `get_state()`, policy locks, and triggers async actions; in practice this serializes most server operations behind a single write lock.

Concrete direction:
1. Move to an actor model for `Server` and `Collector` (Tokio `mpsc` command queue + `oneshot` responses).
2. If you want an incremental refactor: keep locks but ensure you never hold a lock guard across `.await`. “Extract data needed, drop lock, then await”.

### 4) Correctness: collector state inspection via `Any` is broken and fragile

In `_get_state`, the downcast path attempts to downcast a `Mutex<Box<dyn DataSource>>` to `ScreenDataSource`, which will never succeed. It also uses `if let is_running = screen_ds.is_running()` which is an always-true pattern binding.

References:
- `collector/src/collector.rs:343-371` (downcast + buffer inspection)

Concrete fix:
1. Don’t use `Any` downcasting for normal operations like `get_state`/`get_data`.
2. Define an object-safe trait for “erased” sources with the operations you actually need:
```rust
#[async_trait::async_trait]
trait ErasedSource: Send + Sync + std::fmt::Debug {
  fn name(&self) -> &'static str;
  fn is_running(&self) -> bool;
  async fn buffer_len(&self) -> usize;
  async fn drain_to_proto(&mut self) -> Vec<lifelog_proto::LifelogData>;
}
```
Then store `HashMap<&'static str, tokio::sync::Mutex<Box<dyn ErasedSource>>>`.

### 5) Backpressure & memory: repeated clones and “collect all then stream” patterns

Collector get_data:
- `ScreenDataSource::get_data()` clones the entire buffer (`collector/src/modules/screen.rs:44-48`).
- gRPC `get_data` collects all images into a `Vec` and then sends them one by one (`collector/src/collector.rs:509-545`, `553-573`).
- buffer clearing occurs after clone; this doubles peak memory and increases latency.

Server ingest:
- `sync_data_with_collectors()` reads the entire gRPC stream into a `Vec` (`server/src/server.rs:1113-1124`) before ingesting. That’s the opposite of streaming and kills memory usage under backlog.

Concrete rewrite:
1. Change screen buffer access to a drain:
```rust
pub async fn drain(&self) -> Vec<ScreenFrame> {
  let mut g = self.buffer.lock().await;
  std::mem::take(&mut *g)
}
```
2. Stream ingest on the server: process each chunk as it arrives and write to DB incrementally.

### 6) Async/task structure issues: `tokio::spawn` results ignored, join semantics incomplete

Collector main:
- You spawn `collector_handle` but only `try_join!(server_handle)?;` waits on server, not collector (`collector/src/main.rs:122-130`). If the collector loop errors or exits, main won’t notice.

Server actions:
- `do_action` spawns tasks and ignores join results (`server/src/server.rs:767-788`, `808-835`). Errors will be lost and pending actions may get stuck.

Concrete fix:
1. Track join handles (or use `JoinSet`) for background tasks and surface failures.
2. Adopt `tracing` spans and structured error logs rather than `println!`.

### 7) Unsafe/fragile code in build script: leaking strings to `'static`

`common/data-modalities/build.rs` uses `Box::leak` to produce a `'static` protobuf type string (`common/data-modalities/build.rs:193-195`). This is “only build-time”, but it’s still a smell and makes the code harder to reason about.

Concrete fix:
- Return `Cow<'a, str>` or `String` instead of `&str` from `map_protobuf_type`, and format into the output string directly. Build scripts do not need `'static` lifetimes here.

### 8) Performance: screen capture is “write to disk -> read back -> decode image” per frame

Screen logging runs an external command that writes a PNG file, then reads it, then deletes it (`collector/src/modules/screen.rs:172-214`). Then it decodes the image to compute dimensions (`collector/src/modules/screen.rs:105-116`).

Issues:
1. Extra disk I/O per capture.
2. Decode cost just to get width/height.
3. No bounded buffer; `Vec` grows unbounded.

Concrete improvements:
1. Use a platform capture library that returns bytes in memory (macOS: CGDisplayStream / ScreenCaptureKit wrappers, Linux: PipeWire/xdg-desktop-portal, Windows: DXGI/Windows.Graphics.Capture). If external tools remain: pipe stdout instead of writing temp file.
2. Parse PNG header (IHDR) for dimensions without full decode (or store unknown and compute later server-side if needed).
3. Add a bounded queue (e.g., `VecDeque` with max entries or max bytes) and drop/rotate according to policy until disk-WAL exists.

### 9) Query/transform pipeline: O(N) full scans and set-diff in memory

`get_keys_in_source_not_in_destination()` fetches all UUIDs from both tables and computes a set difference in Rust (`server/src/server.rs:1191-1220`). That is O(N) network/DB reads and O(N^2) contains checks in worst case (`Vec::contains`).

Concrete rewrite:
1. Use `HashSet` at minimum for contains.
2. Better: push this logic into SurrealDB query, or maintain a transform cursor/checkpoint per transform (spec already implies this in “resumable/idempotent transforms”).

### 10) Error handling: widespread `unwrap`/`expect` in server data plane and parsing

Examples:
- Database connect uses `expect` with a very long string (`server/src/server.rs:250`).
- Ingest pipeline unwraps stream chunks and payloads (`server/src/server.rs:1106-1125`).
- Parsing UUIDs uses `expect` (`server/src/server.rs:1184-1186`, `common/lifelog-types/src/lib.rs:333-335`).

Concrete direction:
1. Pick a policy: `thiserror` for library crates + typed errors; `anyhow` only at binaries or “top-level boundary”.
2. Replace unwraps in streaming paths with error propagation and backoff/retry.
3. Convert external failures to `tonic::Status` in gRPC layer with consistent mapping.

### 11) API/data model issues: UUID handling is inconsistent and currently “patched over”

You frequently set UUID to `0` in conversions and later overwrite it from the key (`server/src/server.rs:921-932`, `953-963`, `980-988`, and then `get_data_by_key` overwrites UUID at `server/src/server.rs:1319-1320`, `1336-1337`, `1353-1354`).

This indicates the ID model is confused between:
1. DB record ID.
2. In-record UUID field.
3. Proto UUID field.

Concrete direction:
1. Make DB record ID the canonical identifier.
2. Do not store `uuid` inside the row if the row ID already encodes it, or enforce they match in one place.
3. Introduce a typed `RecordId(Uuid)` and a `Record<T>` wrapper for payload + metadata.

### 12) Crate choices and maintainability risks

1. `surrealdb` over remote WebSocket engine for “local-first” introduces a lot of moving parts (WS server process, auth, schema DDL races). If the target is embedded local-first, SQLite (with FTS5) + an object store for blobs is a simpler and battle-tested baseline. `rusqlite` is already in workspace deps.
2. `rust-bert` embedding model download at runtime (`server/src/text-embedding.rs:13-16`) is heavyweight and likely unsuitable for “always-on local” unless you explicitly manage caching, CPU usage, and model files. Also: those tests will hit network/model downloads and will be flaky in CI.
3. `rusty-tesseract` is fine for OCR prototyping but the transform pipeline needs idempotent checkpoints and robust error handling.

### 13) Organization/API surface

1. `lifelog_core` re-exports a large set of crates (`common/lifelog-core/src/lib.rs:1-17`). This tends to hide dependencies and makes versioning harder. Prefer explicit dependencies per crate; if you want a prelude, keep it small and local.
2. Mixed responsibilities in `server/src/server.rs`: gRPC service implementation, DB schema management, transform runner, policy engine, and state management all live in one file. Breaking this into modules will pay off quickly (db, ingest, transforms, policy, api/grpc).
3. Stray/odd file in repo root: a file literally named `command list-panes: too many arguments (need at most 0)` exists. This looks accidental and should probably be removed from the repo history.

## Concrete Refactor Recommendations (Minimal-to-Strong)

### A) Minimal fixes (1-2 days)
1. Fix `DataSourceHandle` to return the real join handle (screen/browser).
2. Replace `static mut` singletons:
`SYS` becomes `Server.sys: sysinfo::System`.
`CURRENT_CONFIG` becomes `OnceCell<ArcSwap<MicrophoneConfig>>` or `watch::Receiver`.
3. Implement buffer drain patterns (`mem::take`) and stream ingest without collecting.
4. Fix collector main to await both spawned tasks (`try_join!(server_handle, collector_handle)`).

### B) Medium refactor (3-7 days)
1. Define `ErasedSource` trait and remove `Any` downcasting from collector.
2. Introduce bounded buffering (max items/max bytes) per modality.
3. Replace `println!` with `tracing` across async tasks, add spans per collector/session.

### C) Strong refactor (1-3 weeks)
1. Actor model for server and collector (single-threaded state machine per component):
- gRPC handlers send commands over `mpsc`.
- long operations run without holding locks.
2. Implement spec’s upload protocol:
- control channel collector-initiated.
- offset-based chunk upload.
- durable ack and resumable uploads.
3. Split metadata store and blob store; store content hashes and references.

## Notes On Tests

1. Embedding tests likely download model weights and will be slow/flaky in CI (`server/src/text-embedding.rs:87-136`). Mark those as ignored or gate behind a feature flag with cached artifacts.
2. Add a property/integration test for “drain semantics”: data is never duplicated and buffers only clear after durable ack (this aligns directly to spec invariants).

