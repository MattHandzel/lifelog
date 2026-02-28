# Agent Task: Implement temporal-operators

## Objective
Implement the feature "temporal-operators" according to the plan.

## Context
- **Plan Document:** PLAN.md
- **Reference:** @SPEC.md and @SPEC_TODOLIST.md
- **Goal:** Autonomous implementation, testing, and verification.

## Initialization Sequence (MANDATORY)
1. Read `GEMINI.md` and `docs/REPO_MAP.md` to refresh architecture and file context.
2. Read the Plan Document (PLAN.md).
3. Check `git status`, `git worktree list`, and `git log -n 5` to understand the current work state.
4. Run `just check` to verify the baseline is green.

## Instructions
- Work strictly within this worktree.
- Follow the "Research -> Strategy -> Execution" lifecycle.
- Prioritize empirical reproduction of any related issues before fixing.
- Ensure all changes are verified with `just validate` or targeted tests.
- If you encounter significant ambiguity, use `ask_user`. Otherwise, proceed autonomously.

## Handoff Report (AGENT MUST COMPLETE THIS)
**Summary of Changes:**
- Refined the temporal interval intersection algorithm in `server/src/query/executor.rs`. The interval overlap query now correctly handles point records (where `t_end` might be `NONE` in SurrealDB) by gracefully falling back to `t_canonical`. This ensures accurate cross-modal `WITHIN`, `DURING`, and `OVERLAPS` target evaluations for point-to-interval expansion.
- Investigated error handling in `server/src/query/planner.rs` and confirmed that descriptive errors for unsupported nested temporal joins are fully implemented as per Spec §10.1 (nested temporal operators and temporal operators under `OR`/`NOT` yield explicit rejections unless rewritten via bounded DNF for top-level `OR`).
- Verified execution multi-stage behavior for `WITHIN` operators correctly processes disjoint interval generation.
- Formatted source code using `cargo fmt` to adhere to rustfmt guidelines (`ingest.rs` and `server.rs`).

**Verification Results:**
- `server/tests/canonical_llql_example.rs` successfully passed locally.
- `server/tests/cross_modal_query.rs` successfully passed locally.
- All modifications passed standard build and format checks via `just check` and `cargo fmt`.

**Manual Steps for User:**
- The branch `agent/temporal-operators` is ready for review and pushing.

## Completion
- Once the feature is implemented and verified, prepare a commit (do not push).
- Summarize the work done in the Handoff Report above.
