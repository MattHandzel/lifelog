# Omar Hassan (ML/Multimodal) Review: Lifelog As A Downstream Intelligence Platform

This review treats Lifelog as an eventual *personal intelligence substrate* (multimodal capture -> indexing -> retrieval -> ML/LLM reasoning), and evaluates the current repo + `SPEC.md` against that goal with emphasis on:

- ML pipeline integration (how cleanly raw/derived data flows into models)
- audio + transcription (Whisper)
- screen OCR quality + storage
- semantic search / embeddings
- activity recognition (cross-modal classification)
- LLM-powered querying (“What was I researching last Tuesday?”)
- analytics / dashboards
- ML-friendly storage and dataset export

I’m anchoring claims in: `SPEC.md`, the current Protobuf surface (`proto/`), collector modules (`collector/src/modules/*`), the gRPC backend (`server/src/server.rs`), and the current OCR transform (`common/data-modalities/src/ocr/mod.rs`).

---

## 1. What The Spec Enables (If Implemented)

`SPEC.md` is conceptually aligned with downstream ML:

- A **stream/record model** with point vs interval semantics and explicit correlation operators (`WITHIN/OVERLAPS/DURING`). This is exactly what you need for cross-modal supervision and retrieval augmentation.
- A first-class **transform pipeline** concept (“derived streams”), with idempotency and versioning requirements (e.g., OCR engine changes create a new derived stream/version).
- A deliberate **metadata vs blob separation** (images/audio as content-addressed blobs, referenced by metadata records), which is the right base for model training and scalable embedding indexes.
- Explicit mention that a **vector index** is a later addition but should have schema hooks reserved.

The main issue: most of what makes this ML-ready is currently *not enforced in code*, and some implemented paths move in the opposite direction (inline bytes in DB rows; in-memory buffers; incomplete modality enum).

---

## 2. Current As-Built Reality (What’s Wired Today)

### 2.1 Data Modalities Actually Flowing End-to-End

Protobuf `DataModality` and `LifelogData` only cover:

- `BrowserFrame`, `ScreenFrame`, `OcrFrame` (`proto/lifelog_types.proto`)
- `ScreenFrame` includes `bytes image_bytes` inline (same file)

Collector sources currently implemented for backend sync:

- Screen capture buffers frames **in memory** (`collector/src/modules/screen.rs`).
- Browser history reads Chrome sqlite directly and returns `BrowserFrame` records (also not durably buffered; it updates a last-query timestamp file) (`collector/src/modules/browser_history.rs`).

Backend ingestion + storage:

- gRPC server pulls from collector via streaming `CollectorService.GetData` and writes into SurrealDB, table-per-origin (`server/src/server.rs`, `sync_data_with_collectors`).
- Storage schema is simple (timestamp + payload fields); **screen image bytes stored inline in SurrealDB rows** (`server/src/server.rs`, `ScreenFrameSurreal.image_bytes: surrealdb::sql::Bytes`).
- OCR transform exists and writes derived `OcrFrame` into a destination origin/table. Implementation currently forces derived record UUID to equal the source UUID (explicit comment “THIS NEEDS TO BE FIXED”) (`server/src/server.rs`, `transform_data`).

### 2.2 API Surface Is Split / Confusing

- The backend in `server/` is a **gRPC-only** tonic server (`server/src/main.rs`).
- Separately, there is a Rocket-based “lifelog-server” binary under the Tauri interface (`interface/src-tauri/src/bin/lifelog-server.rs`) that queries local sqlite DBs directly.
- The web UI (`interface/src/lib/api.ts`) expects REST endpoints like `/api/loggers/...` documented in `server/README.md`, but those are not implemented by the tonic gRPC server in `server/`.

For ML integration and downstream tools, the “canonical” API needs to be unambiguous; right now it isn’t.

---

## 3. ML Pipeline Integration (How Easy Is It To Feed Models?)

### What Exists

- A reasonably clean *domain modularization attempt*:
  - modality structs + transform trait live under `common/data-modalities/`
  - transforms are intended to be a backend concern (`SPEC.md` and `server/src/server.rs`)
- A transform execution loop exists conceptually: backend periodically runs OCR for all “source keys not in destination” (`server/src/server.rs`, `get_keys_in_source_not_in_destination`, `transform_data`).

### What’s Missing / Pain Points

- No stable **“ML extraction/export API”**:
  - There’s no way to ask the system for “all (image, OCR, URL, timestamp, device) tuples between T1 and T2 as a dataset”.
  - There’s no manifest format, no partitioning scheme, no columnar exports (Parquet/Arrow), and no WebDataset-style blob shards.
- No **blob store**; images are inlined into SurrealDB rows today, which is awkward for:
  - GPU dataloaders (zero-copy streaming is hard)
  - storage tiering (hot metadata vs cold blobs)
  - dedup and recompute (content-addressing)
- No strong **lineage tracking** beyond the “destination origin derived from source origin” convention; derived records should point to:
  - source record id(s)
  - transform id/version
  - parameters (e.g., OCR language)
  - timestamps of derivation
- No **time alignment primitives** are implemented (interval vs point; ingest time vs device time; clock skew handling), even though the spec calls this out as foundational.
- The current query path is essentially “return everything”:
  - `Query` RPC ignores the query and returns all keys (`server/src/server.rs`, `query` and `process_query`).
  - This makes ML “retrieval augmentation” hard to evaluate inside the system; you can’t reliably build training data via queries.

### ML-First Recommendations

1. Establish a canonical “data lake” export contract:
   - `ExportDataset(time_range, modalities, origins, derived_versions, filters) -> manifest`
   - manifest lists metadata rows + blob references; blobs are fetchable by content hash/path.
2. Treat transforms as first-class “jobs” with:
   - transform id, version, config hash
   - incremental cursors (do not compute set diffs by scanning full UUID sets)
3. Store data in ML-friendly partitions:
   - by day/week, by modality, by origin
   - metadata in Parquet; blobs in content-addressed files

---

## 4. Audio Processing (Raw vs Transcribed, Whisper?)

### What Exists

- There is a microphone logger that records **plaintext `.wav` files** to an output directory (`collector/src/modules/microphone.rs` using `hound::WavWriter`).
- `MicrophoneConfig` exists in config + proto (`common/config/src/lib.rs`, `proto/lifelog_types.proto`).
- UI expects microphone REST endpoints and can list/play local wav files (`interface/src/components/MicrophoneDashboard.tsx`, `interface/src-tauri/src/storage.rs`, `server/README.md`).

### What’s Missing

- Microphone/audio is **not** part of the backend’s canonical data plane:
  - `DataModality` doesn’t include audio/microphone; `LifelogData` has no audio payload (`proto/lifelog_types.proto`).
  - Backend ingestion (`server/src/server.rs`) only matches `Screenframe` and `Browserframe` in `sync_data_with_collectors`.
- No Whisper integration:
  - Whisper is mentioned as a desirable processing step in docs (`docs/data-modality-representation.md`), but there’s no implementation or pipeline.
- No interval semantics:
  - Spec requires audio chunks as interval records (`SPEC.md`), but current audio storage is filename timestamp + wav file; no “t_start/t_end” record and no alignment to screen frames/events.

### Recommendations

1. Add `AudioChunk` (interval record) to proto + domain model:
   - metadata: `{uuid, t_start, t_end, sample_rate, channels, codec, blob_ref}`
   - blob: content-addressed audio chunk (WAV/FLAC/Opus)
2. Add `TranscriptSegment` as derived stream:
   - `{t_start, t_end, text, confidence, language, model_version, source_audio_uuid}`
3. Implement Whisper as a scheduled transform job:
   - GPU worker optional; on-CPU fallback
   - store word-level timestamps when available (critical for correlation + activity recognition)

---

## 5. Screen OCR Quality (Engine, Accuracy, Raw + Processed Storage)

### What Exists

- OCR engine: Tesseract via `rusty-tesseract` (`common/data-modalities/src/ocr/mod.rs`).
- Transform behavior:
  - converts image -> string using `rusty_tesseract::image_to_string`
  - on conversion failure it returns empty text (transform “succeeds” with empty output)
  - on OCR error, also returns empty string (logs the error)
- Data stored:
  - raw screen image bytes are stored (currently inline in DB) (`server/src/server.rs`, `ScreenFrameSurreal.image_bytes`)
  - derived text stored as `OcrFrame {timestamp, text}` (`server/src/server.rs`, `OcrFrameSurreal`)

### Quality Limitations (From Implementation)

- No preprocessing (grayscale/threshold/resize/denoise). This can be a 2-10x quality delta on screen text OCR.
- No structured OCR output:
  - No bounding boxes, confidence scores, per-line/per-word segmentation.
  - That removes many downstream options: “highlight matching region”, “layout-aware embeddings”, “UI element extraction”, “event segmentation by screen region”.
- No transform versioning stored with derived records, despite `SPEC.md` requiring versioned derivations.
- `engine_path` is part of `OcrConfig` but not used to initialize/locate the engine (`common/data-modalities/src/ocr/mod.rs`).

### Recommendations

1. Store raw + derived with richer OCR artifacts:
   - `OcrFrameV2`: `full_text`, `blocks/lines/words`, `bbox`, `confidence`
   - optionally a compact “rendered text layer” for fast preview
2. Add image preprocessing pipeline knobs (per origin/monitor):
   - downscale cap, sharpening, adaptive thresholding, language packs
3. Persist transform metadata:
   - `{transform_name, version, config_hash, engine_version}` per derived record or per derived stream partition

---

## 6. Semantic Search (Vector Embeddings, Cross-Modal Similarity)

### What Exists

- Proto has an `Embedding { repeated float }` message and `Query` includes `image_embedding` and `text_embedding` fields (`proto/lifelog.proto`).
- There is an experimental sentence embedding module using `rust-bert` (`server/src/text-embedding.rs`), but `rust-bert` is commented out in `server/Cargo.toml`, so this is not currently a working integrated path.
- Screen transforms list `image_embedding` as an available transform, but the implementation is a stub (`common/data-modalities/src/screen/transforms.rs`).

### What’s Missing

- No embedding storage schema (vector column, model version, modality tag, source ref).
- No vector index (FAISS/HNSWlib/pgvector/etc).
- No query planner that can combine:
  - time constraints + text match + vector similarity + cross-stream correlation
- No “embedding jobs” scheduler; `server/src/jobs.rs` and `server/src/process_data.rs` are placeholders.

### Recommendations

1. Define an `Embeddings` derived stream family:
   - `TextEmbedding` for OCR/browser/title/clipboard/shell
   - `ImageEmbedding` for screenshots (and later camera frames)
   - optionally unified multimodal (CLIP/ImageBind) for cross-modal retrieval
2. Pick a pragmatic vector storage/index approach for local-first:
   - SQLite + `sqlite-vss` (fast to ship) or
   - Postgres + pgvector (heavier) or
   - SurrealDB + external FAISS index (two-store approach)
3. Make embeddings versioned and reproducible:
   - store `{model_id, model_sha, dim, pooling, normalize}` and keep old embeddings side-by-side

---

## 7. Activity Recognition (Screen + Audio + Browser -> Auto-Classes)

### What Exists

- Screen frames + OCR + browser URL/title are enough for a baseline “activity classifier” (research vs meeting vs coding vs entertainment), but the system doesn’t yet expose them in an aligned, ML-friendly way.
- Config hints exist for additional context streams (processes, hyprland/window activity, input loggers), but they are not represented in `DataModality` / `LifelogData` for backend ingestion (`proto/lifelog_types.proto`).

### What’s Missing

- A first-class **event segmentation layer**:
  - Lifelogs become tractable when you convert point streams into “events” (chunks) with stable IDs: `{t_start, t_end, summary, modality evidence}`.
- Feature assembly + labeling:
  - You need an annotation workflow (even lightweight) to train and validate activity models.
  - Without labels, you’re limited to unsupervised clustering (still useful, but needs embeddings + event segmentation).
- Cross-modal alignment utilities:
  - map screen frames to the nearest audio chunk(s)
  - map browser visits to screen intervals
  - incorporate active window/process context

### Recommendations

1. Add an `Event` derived stream:
   - event = an interval with pointers to contributing records (screen IDs, transcript segments, browser visits)
2. Implement baseline activity recognition as a transform:
   - rule-based first (cheap, debuggable)
   - then ML classifier (light model, interpretable)
3. Build a small labeling UI:
   - show replay window + suggested label
   - store labels as another derived stream versioned by label schema

---

## 8. LLM Integration (Natural Language Queries Over Personal Data)

### What Exists

- Spec defines the required capability and a deterministic query language model (`SPEC.md` Sections 10–11).
- There is an open issue placeholder in the spec’s embedded issue list (#12 NL-to-query), but the implemented query handler is currently a stub (`server/src/server.rs`, `query`).

### What’s Missing

- A typed query AST + planner + safe execution:
  - LLM integration is an *adapter* on top of this, not a replacement.
  - The current system cannot reliably answer “What was I researching last Tuesday?” because it can’t express “Tuesday” -> time window -> correlated OCR + browser + transcript constraints.
- Indexes for fast retrieval:
  - OCR/browser full-text index; embedding index; time index.
- A privacy-aware prompt/data policy layer:
  - LLMs should be constrained to only pull the minimum required snippets, and never exfiltrate raw audio/screen unless explicitly requested.

### Recommendations

1. Implement the deterministic query language first (spec-aligned).
2. Add an NL-to-AST layer:
   - LLM produces AST + provenance (“why this filter”), then you validate/execute.
3. Add a “retrieval summarizer” step:
   - Return small evidence packets (snippets + thumbnails + transcript segments) plus a synthesized answer.

---

## 9. Analytics / Dashboards (Screen Time, App Usage, Productivity)

`SPEC.md` explicitly lists “quantified-self analytics UI” as a v1 non-goal. So dashboards are missing by design, not by accident.

That said, downstream intelligence benefits from computed metrics and materialized views.

### What’s Missing To Support Analytics Well

- A metric store (daily aggregates, app usage durations, meeting time, focus blocks).
- App/window activity streams (process/window title intervals) represented end-to-end.
- Habit/pattern mining over an event layer (not raw frames).

### Recommendation

Treat analytics as derived streams (materialized views) with the same transform/version model:

- `DailyUsageSummary {date, app, minutes, context}`
- `FocusBlocks {t_start, t_end, confidence}`
- `TopicTrends {date, embedding_cluster_id, top_terms}`

---

## 10. Data Formats And Exportability (Training Dataset Friendly?)

### What Exists

- Screen frames are currently “already in bytes”; easy to feed to models *if you can query them efficiently*.
- Browser frames are structured and small.
- OCR text exists as a derived modality.

### What Blocks ML Dataset Creation Today

- No canonical store layout for blobs (spec wants blob store; current code inlines bytes in SurrealDB).
- No dataset export or manifest format.
- No consistent schema/versioning for transforms and derived data.
- Many important modalities exist only as configs or local sqlite (hyprland/processes/microphone) but not in the unified backend schema.

### Recommendations (Concrete)

1. Blob store + metadata DB separation (spec requirement):
   - blobs in a content-addressed filesystem layout (or local object store)
   - DB stores blob hash + size + codec + timestamps
2. Dataset export:
   - metadata to Parquet (Arrow schema) with stable columns and transform version fields
   - blobs referenced by hash, exportable as WebDataset tar shards for training
3. Add “ML schema stability” rules:
   - never mutate records in-place; only new versions
   - every derived record includes `{source_ids, transform_id, transform_version}`

---

## 11. The Missing Infrastructure To Turn Capture Into Intelligence

If the goal is “actionable personal intelligence” (not just capture + recall), the highest-leverage missing building blocks are:

1. **Unified canonical backend API**
   - pick gRPC or REST for the UI and tooling, and provide one stable surface
2. **Blob store + content addressing**
   - required for scalable embeddings, audio, and training exports
3. **Job system for transforms**
   - durable queue, incremental cursors, versioned outputs
4. **Indexing layer**
   - full-text index (OCR/browser/etc), vector index, time index
5. **Event segmentation + lineage**
   - events become the unit of retrieval, labeling, and summaries
6. **Embeddings + similarity search**
   - model registry, embedding storage schema, query integration
7. **Transcription (Whisper) pipeline**
   - interval semantics + word timestamps + correlation hooks
8. **Dataset export and evaluation harness**
   - manifests, partitions, reproducible slices, and a test suite for retrieval tasks

---

## 12. “If I Had To Prioritize” (ML Readiness Path)

1. Implement spec-required storage boundaries (metadata vs blobs) and durability semantics.
2. Expand proto/domain coverage for v1 modalities, starting with audio chunks + window/app activity.
3. Build a transform/job runner with versioned outputs (OCR, Whisper, embeddings).
4. Add text index + vector index.
5. Add event segmentation and activity labeling/classification as derived streams.
6. Only then: NL-to-query via LLM on top of deterministic query planning.

