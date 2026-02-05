# Large Refactor Preparation (Docs Synthesis)

This document is a plain-English synthesis of everything under `docs/`. It aims to answer:

- What is the purpose of this system?
- What principles does it commit to?
- What architectural boundaries are implied by the docs?
- What abstractions should exist if remaking the project from scratch with a focus on backend + interface, and with collectors decoupled?

It intentionally avoids describing the current implementation details and instead focuses on the intended system.

---

## 1. Purpose: What This System Is For

The system is a **local-first lifelog platform**: it continuously (and optionally) captures a broad range of user-related data from many devices, stores it durably, derives higher-level representations, and provides an interface that lets the user **retrieve and use any part of their history quickly**.

The system’s value proposition, per `docs/vision.md` and `docs/concepts-level-1.md`, is effectively:

- Capture arbitrary personal data over time (many modalities).
- Preserve it without loss.
- Transform it into more meaningful/usable forms (e.g., OCR, embeddings, speech-to-text, object recognition).
- Enable fast, flexible retrieval (SQL-like, keyword-like, vector-like, and multimodal).
- Allow authorized applications to read/append/update in a controlled, least-privilege way.

The system is expected to be useful for:

- Memory augmentation: “find what I did/saw/said/heard at some time”.
- Personal analytics and insights: “what patterns exist in my behavior”.
- Personalization: applications leveraging user-owned data without ceding ownership to third parties.
- Potential opt-in third-party sharing (studies), with explicit controls.

---

## 2. Conceptual Entities (Vocabulary)

Across `docs/vision.md`, `docs/concepts.md`, `docs/concepts-level-1.md`, `docs/collector.md`, and `docs/server.md`, the system is consistently described in terms of three roles:

### 2.1 Device Capture Programs (Collectors)

Device-specific programs that:

- Run on each device (laptop/desktop/phone/etc.).
- Capture data from local sources (screen, audio, processes, browser history, etc.).
- Buffer locally (memory and/or disk).
- Are configurable and ideally hot-reloadable.
- Can send data to the backend when appropriate (policy/conditions).
- Report health/status (sources enabled, buffer sizes, last captured times, failures).

The docs emphasize reliability (fault tolerance, buffering, acknowledgements) and the ability to send large volumes (gigabytes per session).

### 2.2 Central Backend (Server)

A central coordinator that:

- Receives data from many devices.
- Stores raw and derived data durably.
- Runs transforms and maintains derived views.
- Manages synchronization and consistency between device buffers and central storage.
- Exposes APIs for configuration, querying, and retrieval.
- Maintains audit history for important actions, especially anything that is not purely append-only.
- Is policy-driven: it makes decisions about syncing, transforms, compression, backups, resource usage.

### 2.3 User-Facing Interface (Interface)

The primary UI that:

- Authenticates the user to the backend.
- Lets the user browse modalities and derived views.
- Provides querying and fast retrieval.
- Shows system status (collectors alive, buffer sizes, last sync, etc.).
- Allows configuration changes (planned).

### 2.4 External Applications (Agents / Sinks)

The docs use two near-overlapping concepts:

- “Agents” (`docs/vision.md`): applications that can read/append/write with explicit permissions.
- “Sinks” (`docs/sinks.md`): components that extract information from the lifelog (e.g., spellcheck or “server keeps a running model of you”).

Net: the system is intended to be an **open platform** where other applications can consume data and/or produce annotations, but only under a strict permission model (“least privilege”).

---

## 3. Principles (What The Docs Treat As Non-Negotiable)

This section distills the stated principles and implied architectural commitments across `docs/concepts.md`, `docs/concepts-level-1.md`, `docs/vision.md`, `docs/policy.md`, `docs/server.md`, and the buffering/acknowledgement notes.

### 3.1 Local-First, User-Owned Data

- The baseline assumption is that the user owns their data and the system stores it locally (even if the server can be remote).
- Third-party access is opt-in and explicitly scoped.

### 3.2 “Store Everything, Don’t Lose Anything”

Explicitly stated as a principle (`docs/concepts.md`, `docs/concepts-level-1.md`).

Implications:

- Prefer append-only ingestion.
- Prefer durable buffering and resumable upload.
- Do not silently drop data under normal operation.
- When data cannot be stored, fail loudly and surface health/status signals.

### 3.3 Strong Typing + Schema Discipline

Explicitly required (“Strongly typed”).

Implications:

- Data formats must have explicit schemas per modality.
- Schema evolution should be planned (backwards/forwards compatibility).
- Typed APIs are preferred to ad-hoc JSON.

The presence of the long Protobuf doc (`docs/protobuf.md`) reinforces this direction: schema-first messages, versioning, and streaming ingestion patterns.

### 3.4 Fault Tolerance, Resumability, and End-to-End Acknowledgement

Explicit requirement: “fault-tolerant … so no data is lost”.

Supporting docs:

- `docs/buffering.md`: compares memory vs disk buffers and emphasizes durability tradeoffs and backpressure.
- `docs/acknowledging-data-written.md`: argues for end-to-end acknowledgement, not “fire and forget”.
- `docs/protobuf.md`: describes resumable, offset-based streaming and idempotency.

Implications:

- Collectors need durable queues/buffers (especially for push-style sources).
- Backend ingestion must be idempotent or deduplicated.
- Acks must reflect persistence, not just receipt.

### 3.5 Configuration As A First-Class Product

Explicit principle: “Everything configurable should be in a single config file” (`docs/concepts-level-1.md`).
Also repeated: system-wide configuration should exist and a UI is planned to edit it (`docs/features-roadmap.md`).

Implications:

- Clear separation between configuration state and runtime state.
- A defined “control plane” for propagating config to collectors and verifying activation.

### 3.6 Policy-Driven Background Operation

`docs/policy.md` and `docs/server.md` describe the backend as an automated worker:

- It decides when to sync, transform, compress, back up, and train models.
- It should consider resource budgets: CPU usage, network usage, etc.

Implications:

- Background jobs are not ad-hoc; they are scheduled/selected by a policy engine.
- Policy needs visibility into system state (collector states, backlog sizes, transform gaps).

### 3.7 Extensibility of Modalities and Transforms

The docs repeatedly frame modalities and transforms as extensible:

- Many modalities listed (`docs/features-roadmap.md`).
- Each modality should define its schema, transforms, sync frequency, compression preferences, and buffering strategy (`docs/data-modality-representation.md`).
- Transforms form a DAG / pipeline per input type (`docs/concepts-level-1.md`).
- External applications should be able to register transforms (implied by “open platform” aspirations).

Implications:

- The core system should not hardcode “which modalities exist”.
- There must be a registry-like concept for modality definitions and transform definitions.

### 3.8 Security, Authentication, and Least Privilege

Security is explicitly called out:

- Secure communication (TLS).
- Explicit permissions for external apps (read/append/write).
- Secure collector registration (“ensure new collectors are from the user and not malicious”).
- Interface authentication.

Implications:

- Separate identity for user, devices, and external apps.
- Authentication at the edge (interface/backend and collector/backend).
- Authorization rules that are easy to reason about and audit.

### 3.9 Auditability, History, and Undo for Non-Append Changes

`docs/server.md` explicitly says that any modification that is not read/append should have history and be undoable.

Implications:

- Mutations must be tracked.
- The system needs a change log / audit log.
- “Rewrite” should likely mean “create a new version” rather than destructive overwrite.

---

## 4. Core Data Model (As Implied By The Docs)

### 4.1 Data Modalities and Data Sources

The docs distinguish:

- A modality: the type of data (screen frames, audio, browser events, etc.).
- A source: a specific origin of that modality (device + modality at minimum; can also include “which screen”, “which browser”, “which sensor”).

Implications:

- Storage should be partitionable by source identity.
- Queries must be able to target modalities and/or sources.

### 4.2 Raw vs Derived Data

Transforms create new derived modalities (e.g., OCR output is its own modality).

Implications:

- Derived data should be stored explicitly, not just computed on the fly.
- There must be lineage: derived record should reference the raw record(s) that produced it, or at least share an identifier plus provenance metadata.

### 4.3 Query Classes

`docs/querying.md` and `docs/search.md` imply multiple query modes:

- Structured (SQL-like).
- Text matching (boolean, fuzzy, wildcards).
- Vector similarity.
- Multimodal / cross-modal retrieval (implied by `docs/research-challenges.md`).

Implication:

- The query layer is not one mechanism; it is an orchestration layer that can route subqueries to different indexes/stores.

---

## 5. Communication Model: Control Plane vs Data Plane

Several docs implicitly call for a split:

- Data plane: bulk transfer of captured records (potentially gigabytes).
- Control plane: status, config, “sync now”, and other commands.

Supporting docs:

- `docs/protobuf.md`: recommends streaming with offsets, resume, idempotency; and a control channel.
- `docs/server-device-communication.md`: explicitly debates who is master (server vs collector) and outlines config update + sync negotiation.
- `docs/server-interface-communication.md`: discusses query streams and explicit cancellation/closing semantics.

Implications (important for your refactor goal):

- The backend should define a stable protocol for control and data transfer.
- Collectors should be “clients” of that protocol; they should not be coupled to the backend’s internal storage choices.

---

## 6. What This Means For A Rewrite: The Key Abstractions

You said: focus the remake on **backend + interface**, and reduce tight coupling between collectors and server/database.

The docs already support that direction: collectors are described as device programs with their own buffering and logging responsibilities, while the server is described as storage/processing/querying. The coupling you want to remove should be removed by designing the boundaries intentionally.

Below are the abstractions that follow from the docs.

### 6.1 A Stable “Ingestion Contract” (Collector-Independent Backend)

Define a backend API that treats collectors as generic producers:

- Register device identity + capabilities.
- Report state/health.
- Push or be-pulled for data uploads.
- Receive config updates and apply them.

The backend should not assume anything about:

- the collector’s internal buffering implementation,
- how the collector captures data,
- what local database (if any) the collector uses.

Instead, the contract should revolve around:

- **typed records** and **explicit schemas**,
- **durable, idempotent ingestion** (dedupe keys),
- **acknowledgements** that prove persistence,
- **incremental resume markers** (offsets or per-stream checkpoints),
- **explicit cancellation** and **backpressure** signals.

### 6.2 A Storage Abstraction (Database Choice Hidden Behind Interfaces)

The docs mention SurrealDB today, but also explicitly entertain separating “blob-like” data into an object store (`docs/database.md`).

A rewrite should treat storage as a set of roles:

- Metadata + indexing store (timestamps, modality fields, search indexes).
- Blob store for large binary payloads (images, audio, video).
- Optional vector index store (for embeddings).

This is a natural seam for decoupling:

- Collectors upload records; backend decides where/how to store them.
- Interface queries the backend; backend routes to appropriate store(s).

### 6.3 A Modality Definition Registry

`docs/data-modality-representation.md` is essentially a checklist for what each modality must define:

- availability constraints (device/OS/dependencies/permissions),
- schema,
- preferred transforms and parameters/priorities,
- sync schedule hints,
- compression preferences,
- buffering preferences.

In a rewrite, represent this as a “modality definition” abstraction that the backend and interface can introspect:

- Backend uses it to validate ingestion, store correctly, and schedule transforms.
- Interface uses it to render modality-specific UI and configuration.

Critically: collectors should only need the subset relevant to capture; they should not need database schemas or transform graphs unless you decide they do local pre-processing.

### 6.4 A Transformation Engine + DAG Model

The docs explicitly want transform pipelines as DAGs. That implies:

- A registry of transforms (name, input modality, output modality, parameters, priority).
- A planner/executor that schedules transforms, tracks progress, and persists results.
- A lineage model so derived outputs can be traced and re-derived when transforms change.

This also ties directly into policy.

### 6.5 A Policy Engine That Selects Jobs Under Resource Budgets

The policy abstraction is central in the docs:

- Choose when to sync, transform, compress, back up, train.
- Respect CPU/network budgets.
- React to system state: collector backlog sizes, query demand, transform gaps.

In a rewrite, keep policy as:

- a pure decision function over a read-only “system state snapshot”,
- producing “job intents” (work items) to be executed by a job runner,
- with audit logging around the chosen actions.

### 6.6 A Query Layer That Orchestrates Multiple Retrieval Methods

The docs acknowledge retrieval is a major research challenge and that queries come in classes:

- structured filters,
- keyword text matching,
- vector similarity,
- cross-modal retrieval,
- possibly conversational refinement (`docs/collaboration-between-system-and-user.md`).

A rewrite should represent queries as:

- a typed, composable query AST (boolean composition),
- with backend routing to the appropriate index/store,
- with streaming results + explicit cancellation semantics.

---

## 7. Architectural Boundaries To Enforce (To Avoid Tight Coupling)

This is the “principles turned into module boundaries” part.

### 7.1 Backend Should Not Import Collector Implementations

The backend should depend on:

- protocol definitions (schemas/messages),
- storage interfaces,
- transform interfaces,
- policy interfaces.

It should not depend on device capture code, OS-specific modules, or concrete collector “drivers”.

### 7.2 Collectors Should Not Know Backend Storage Internals

Collectors should never need to know:

- what database the backend uses,
- how tables are named,
- how derived data is stored.

Collectors should only know:

- how to serialize/stream records for a modality,
- how to persist local buffers durably,
- how to apply config and report state,
- how to resume uploads and handle acknowledgements.

### 7.3 Protocol First: Backend + Interface Can Be One Repo; Collectors Can Be Separate

Given your stated direction (backend + interface focus), the cleanest decomposition is:

- Repository A: protocol + backend + interface (your focus).
- Repository B: collectors (optional, device-specific, can be multiple independent implementations).

Even if you keep them in one monorepo, treat collectors as if they were external by enforcing dependency direction (collectors depend on protocol crate; backend depends on protocol crate; backend does not depend on collectors).

---

## 8. Derived “North Star” Requirements (From The Docs)

This is a checklist of what the rewrite should still satisfy if it is faithful to the docs.

### 8.1 Reliability

- Durable buffering (prefer disk-backed queues for important modalities).
- End-to-end acknowledgement semantics: only drop local data after backend has persisted it.
- Resumable uploads (offset/checkpoint model).
- Idempotent ingestion (dedupe keying).

### 8.2 Scale / Performance

- Transfers measured in gigabytes per session are first-class.
- Streaming APIs (HTTP/2 streaming, chunking).
- Backpressure and cancellation semantics.
- Monitoring and a performance test suite (`docs/tests.md`).

### 8.3 Extensibility

- Easy to add a new modality definition.
- Easy to add transforms and schedule them.
- Ability to support many devices and OS combinations, even if not all are implemented immediately.

### 8.4 Security

- TLS/mTLS for device-backend.
- AuthN/AuthZ for interface-backend.
- Least privilege for external apps.
- Secure collector enrollment / pairing.

### 8.5 Usability

- “Easy setup per device”.
- Real-time feel: captured data can show up quickly.
- A single configuration surface (file now, UI later).

---

## 9. Risks and Ambiguities In The Current Docs (Good To Decide Up Front)

These are recurring questions or inconsistencies that matter when re-architecting.

### 9.1 Who Initiates Sync? (Push vs Pull)

`docs/server-device-communication.md` explicitly debates whether the device program is “master” or “slave”.

A rewrite should make an explicit choice, or support both:

- Push: device uploads when it decides conditions are right.
- Pull: backend requests uploads based on policy and demand.
- Hybrid: device can request permission to upload; backend schedules actual retrieval.

### 9.2 What Does “Don’t Delete Anything” Mean In Practice?

Storing everything forever conflicts with:

- disk constraints,
- privacy needs (user wants some things not tracked),
- legal compliance.

The rewrite should define:

- retention policies (even if default is “forever”),
- redaction and “private mode” handling,
- whether raw data can be pruned after derived data is created (maybe only under explicit user policy).

### 9.3 What Is The Canonical Representation?

Docs show “everything as tables”, but also question if a better representation exists (knowledge graph idea).

A rewrite can still start with a table/record model, but should:

- preserve explicit schemas and lineage,
- avoid coupling to one DB’s DDL features,
- keep the door open for alternate indexes (vector/text/graph).

### 9.4 “Agents” vs “Sinks” vs “Interface”

The docs mention three consumers:

- the main UI,
- external apps that consume lifelog data,
- “sinks” that compute on lifelog data.

In a rewrite, define these explicitly:

- UI: privileged user-facing app.
- External app: third-party or local app with a permission grant.
- Compute job: transform pipeline execution (internal) vs external compute plugin (registered transform).

---

## 10. Suggested Rewrite Framing (If You Want A Clean Scope)

Given your stated goal (backend + interface focus), a reasonable “from-scratch” scope that stays true to the principles is:

1. Backend implements:
   - device/app identity and auth,
   - control plane (register/config/status),
   - data plane (streaming upload + acks + resume),
   - storage abstraction (metadata + blobs),
   - transform engine skeleton (one transform end-to-end),
   - query API (keys + fetch + at least one retrieval method),
   - audit log.
2. Interface implements:
   - auth and connection lifecycle,
   - query UI with cancellation,
   - modality browsing,
   - system status panel.
3. Collectors become “external implementations” that speak the protocol:
   - provide a reference collector later, but do not let backend architecture depend on it.

---

## Appendix: Where Each Principle Came From

- Vision and roles: `docs/vision.md`
- Principles/requirements: `docs/concepts.md`, `docs/concepts-level-1.md`
- Collector responsibilities and sync ideas: `docs/collector.md`, `docs/server-device-communication.md`
- Server responsibilities and audit/undo: `docs/server.md`, `docs/policy.md`
- Modality definition checklist: `docs/data-modality-representation.md`
- Reliability patterns (buffering/acks): `docs/buffering.md`, `docs/acknowledging-data-written.md`
- Query/search directions: `docs/querying.md`, `docs/search.md`, `docs/research-challenges.md`
- Streaming + resumability patterns: `docs/protobuf.md`
- Interface query cancellation: `docs/server-interface-communication.md`

