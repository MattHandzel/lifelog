# Agent Task: Implement search-previews

## Objective
Implement the feature "search-previews" according to the plan.

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
- Implemented rich search previews in the Interface:
  - `SearchDashboard` now performs two-step enrichment:
    1. `query_timeline` for candidate keys.
    2. `get_frame_data_thumbnails` for enriched frame data including text fields and image previews.
  - Added query-term snippet extraction and term highlighting in result cards.
  - Added file-type and source filtering plus deterministic timestamp sorting on enriched results.
- Added lazy thumbnail rendering:
  - `ResultCard` now uses a `Thumbnail` renderer with intersection-based lazy image mounting and loading skeleton.
- Added backend (Tauri command) thumbnail support:
  - New command `get_frame_data_thumbnails` in `interface/src-tauri/src/main.rs`.
  - Thumbnail mode generates downscaled images for screen/camera payloads to reduce preview payload size.
- Added tests:
  - New `interface/src/test/SearchDashboard.test.tsx` covering snippet highlighting and thumbnail rendering.
  - Updated test invoke setup to provide empty defaults for unmocked frame-data commands, preserving Timeline tests.

### Verification
- Baseline check: `just check` (pass).
- UI tests: `just test-ui` (pass after dependency install and compatibility fixes).
- Full validation: `just validate` (pass).

### Manual Steps
- None required for local development beyond existing `just` workflows.

## Completion
- Once the feature is implemented and verified, prepare a commit (do not push).
- Summarize the work done in the Handoff Report above.
