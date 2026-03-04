# Refactor Todo List (from `changes-to-be-made.txt`)

## 1) Enrollment and CLI ownership

- [ ] Update `lifelog-server generate-token` UX:
  - offer `--set-active`/equivalent non-interactive flag to immediately activate generated token, and
  - offer a path to retrieve/show current active enrollment token for existing enrollments.
- [ ] Define enrollment token rotation policy:
  - decide whether each new collector enrollment invalidates prior tokens,
  - document tradeoff between one-time token security and multi-device onboarding ergonomics.
- [ ] Move `join` command from server binary to collector binary.
- [ ] Move `join` helpers to collector crate (e.g., `parse_server_url`, fingerprint helpers, cert verifier).
- [ ] Restrict server CLI surface to `serve`, `generate-token`, and `init`.

## 2) Configuration model and runtime behavior

- [ ] Enforce project rule: all non-secret runtime configuration must come from `lifelog-config.toml`.
- [ ] Reduce env var usage to secrets/bootstrap only (DB passwords, API keys, token secrets, cert/key paths if needed).
- [ ] Migrate server runtime knobs currently in env (e.g., retention interval) into server config schema.
- [ ] Replace second-based retention interval settings with user-facing duration units (days/weeks/months/years).
- [ ] Ensure every collector device has its own `lifelog-config.toml` (shared schema across server + collectors).
- [ ] Revisit env-file design:
  - support explicit `--config <path>`,
  - optionally persist “last-used config path” behavior similar to tmux semantics.

## 3) Collector/server init UX

- [ ] Remove microphone-specific prompt from server init flow.
- [ ] Introduce collector init/config flow that lets users select:
  - enabled modalities,
  - enabled transforms,
  - transform-specific settings.
- [ ] Ensure all init/config flows support non-interactive mode (`--yes`, flags, or config-driven execution).

## 4) Architecture and code organization

- [ ] Remove modality-specific storage-root path rewrites (`apply_storage_root_paths`) if DB/CAS storage makes them obsolete.
- [ ] Refactor utility functions currently in `main.rs` (e.g., `sanitize_collector_id`) into shared utils modules.
- [ ] Apply DRY pass to CLI and config-handling code paths.
- [ ] Add lint/review rule: avoid modality-specific one-off functions in server core (generalize abstractions).

## 5) Replay and interface boundaries

- [ ] Decouple replay from server-owned modality-specific assembly.
- [ ] Keep replay orchestration in interface/client layer:
  - server provides time-window/modality query primitives,
  - client composes playback sequence.
- [ ] Remove/replace modality-specific replay helpers (example: `build_replay_steps_for_screen`) with generalized query/sequence APIs.
- [ ] Ensure frontend exposes server state clearly in UI.

## 6) Database and migrations

- [ ] Complete migration to PostgreSQL as the only primary database backend.
- [ ] Remove SurrealDB/hybrid ingest paths after PostgreSQL parity is confirmed.
- [ ] Refactor commands, ingest paths, and runtime checks to PostgreSQL-only assumptions.
- [ ] Move migration location from `server/migrations/` to `database/migrations/`.

## 7) Transform system

- [ ] Add `SSTTransform` (speech-to-text) with configurable provider target:
  - self-hosted speech server endpoint and/or managed service.
- [ ] Verify OCR and speech-to-text transforms are both functional end-to-end.
- [ ] Add transform capability model declaring which modalities each transform accepts.
- [ ] Add per-transform modality configuration (e.g., OCR allowed on screen and/or camera).
- [ ] Support transform chaining (`modality -> transform -> transform ...`) with explicit lineage.

## 8) Storage naming and data model proposals

- [ ] Evaluate proposed per-collector/modality table naming model:
  - `MAC_ADDRESS_OF_COLLECTOR{SEP}DATA_MODALITY{SEP}`
  - derived table variant with chained transform suffixes.
- [ ] Decide canonical identity source for table naming (MAC vs stable cryptographic collector id).
- [ ] Document naming constraints (length, character set, migration safety, cross-platform consistency).

## 9) Testing and automation principles

- [ ] Make all major CLI workflows scriptable and non-interactive.
- [ ] Ensure interface behavior is verifiable headlessly (integration/e2e automation without manual browser steps).
- [ ] Keep browser-only failures scoped to rendering/runtime, not business logic correctness.
