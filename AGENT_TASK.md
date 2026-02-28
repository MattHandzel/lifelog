# Agent Task: Implement replay-backend

## Objective
Implement the feature "replay-backend" according to the plan.

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

### Summary of Changes
- Verified that the `replay-backend` implementation correctly exists and works, properly utilizing `lifelog-core` replay step assembly logic.
- Created an integration test `server/tests/replay_query.rs` to empirically verify that the `Replay` RPC properly translates screen frames into sequential time steps and correctly assigns temporally overlapping context records (e.g. `BrowserFrame`) to those steps.
- Updated `SPEC_TODOLIST.md` to reflect that the replay queries (§5.7, §5.8) and the replay query test (§12.3) have been successfully implemented and verified.

### Verification Results
- Executed `cargo test -p lifelog-server --test replay_query -- --ignored`, which passed successfully, confirming the correctness of interval matching and step assembly.
- All code formatted and linted properly.

### Manual Steps
- Review the `replay_query.rs` test coverage. Ensure it aligns with your expectations for testing edge cases if any additional complex overlapping semantics emerge.
- UI Integration is still pending per `SPEC_TODOLIST.md`. You will need to wire the frontend replay view to this verified backend RPC.

## Completion
- Once the feature is implemented and verified, prepare a commit (do not push).
- Summarize the work done in the Handoff Report above.
