# Agent Task: Implement durable-ack

## Objective
Implement the feature "durable-ack" according to the plan.

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

## Handoff Report
- **Durable ACK gating fix**: Modified `server/src/ingest.rs` to ensure that idempotent retries of chunk uploads do not revert the `indexed` flag back to `false` for chunks that have already completed async processing (like OCR). Used `indexed = (indexed = true OR $indexed = true)` in the UPSERT to preserve state.
- **IT-081 validation fix**: Updated `it_081_ack_implies_queryable` in `server/tests/validation_suite.rs` to correctly simulate OCR being enabled via `LIFELOG_TRANSFORMS_JSON`, accurately testing the `Screen` chunk gating behavior. Fixed the SurrealDB update assertion to correctly parse `Vec<RawRec>`.
- **IT-090 fix**: Updated the assertion in `it_090_resume_upload_with_byte_offsets` to anticipate that chunks with unknown stream IDs are immediately marked as `indexed=true` to prevent ACK blocking.
- **Verification**: Verified via `just test-e2e` that all durable ACK integration tests are now passing successfully. No manual user steps are required.

## Completion
- Once the feature is implemented and verified, prepare a commit (do not push).
- Summarize the work done in the Handoff Report above.
