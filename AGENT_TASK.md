# Agent Task: Implement network-topology-ui

## Objective
Implement the feature "network-topology-ui" according to the plan.

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
- Implemented a new `NetworkTopologyDashboard` in the interface with:
  - Server + collector node visualization in a topology graph.
  - Glowing connection lines and animated per-modality pulse markers inferred from live source state.
  - Node selection and detailed health panel (status, last seen, backlog bytes, buffer count, last upload time).
  - Per-modality enable/disable controls (`screen`, `camera`, `microphone`, `processes`, `hyprland`) via existing `get_component_config`/`set_component_config` RPC-backed commands.
  - Collector pause/resume action by toggling all available modalities.
  - Alias/icon editing as explicit local UI overrides (persisted in localStorage).
  - Force sync action wired to an RPC call with explicit user-facing warning when unavailable in current backend API.
- Replaced the top-level sidebar view from `Devices` to `Network` and wired it to the new dashboard.
- Added a focused frontend test: `interface/src/test/NetworkTopologyDashboard.test.tsx`.

### Verification Results
- `tools/ai/run_and_digest.sh "just check"`: pass (baseline).
- `tools/ai/run_and_digest.sh "just test-ui"`: pass.
- `tools/ai/run_and_digest.sh "just validate-all"`: pass.

### Manual Steps
- Open the Interface and navigate to the `Network` view.
- If a backend force-sync RPC is introduced later, wire it to the existing `force_collector_sync` invoke call in the dashboard.

## Completion
- Once the feature is implemented and verified, prepare a commit (do not push).
- Summarize the work done in the Handoff Report above.
