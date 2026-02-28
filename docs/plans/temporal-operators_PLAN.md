# Plan: Temporal Query Engine Completion

## Objective
Enable full cross-modal correlation by finalizing the `WITHIN` and `DURING` operators in the query engine.

## Phase 1: Research & Strategy
1. **Inventory:** Analyze `server/src/query/planner.rs` and `executor.rs`.
2. **Interval Intersection:** The `DURING` operator currently handles conjunctions (`AND`) by intersecting interval sets. We need to ensure it handles the "Point-to-Interval" expansion correctly (e.g., expanding a single Screen Capture timestamp into a small window to catch overlapping Audio).
3. **Multi-Stage Execution:** Ensure the `WITHIN` operator can handle multiple terms (e.g., `A within B and A within C`).

## Phase 2: Execution
1. **Refine Interval Logic:** Update `server/src/query/executor.rs` to use a robust interval intersection algorithm that supports both `Point` and `Interval` record types.
2. **Canonical Test Case:** Ensure the test in `server/tests/canonical_llql_example.rs` (Audio during YouTube + 3Blue1Brown) is fully enabled and passing.
3. **Error Handling:** Add descriptive errors for unsupported nested temporal joins (as per Spec §10.1).

## Phase 3: Verification
1. Run `just test --test canonical_llql_example`.
2. Run `just test --test cross_modal_query`.
3. Verify that result timestamps (`t_canonical`) are correctly returned to the UI.

## Model Recommendation
**Gemini 2.5 Pro** (Required for complex SQL generation and interval math).
