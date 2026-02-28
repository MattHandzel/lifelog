# Plan: Durable ACK Gating

## Objective
Tighten the data consistency loop: `ACK` only when data is fully indexed and searchable.

## Phase 1: Research & Strategy
1. **Consistency Gaps:** Analyze `server/src/ingest.rs`. Currently, `indexed=true` is set on chunk metadata after persistence, but it may not wait for the SurrealDB text indexer to finalize its write.
2. **Indexing Strategy (Decision):** We decided on "Relaxed Indexing" (background indexing). We need to confirm that `indexed=true` is set *after* the text search index update has been triggered.

## Phase 2: Execution
1. **Wait for Searchability:** Update the `ingest` logic to query the SurrealDB index status or explicitly include index updates in the ingestion transaction.
2. **Modality Checks:** Ensure that for each modality, its baseline text fields (URL/title, Clipboard text, Shell command) are part of the `indexed` flag's scope.
3. **Screen-OCR Coupling:** Keep the existing logic that Screen Capture `indexed=true` waits for the OCR-derived record to be persisted (if OCR is enabled).

## Phase 3: Verification
1. Create a "Durable Search" integration test:
    - Ingest a record.
    - Immediately (within the same test step) attempt a `Search` for a unique word in that record.
    - Assert that if `indexed=true` was returned by the ingest RPC, the search *must* succeed.

## Model Recommendation
**Gemini 2.5 Flash** (Standard transactional/ingest work).
