# Plan: Replay Backend Implementation

## Objective
Implement the logic to assemble Replay steps with perfectly aligned context (keystrokes, clipboard, shell commands).

## Phase 1: Research & Strategy
1. **API Mapping:** Review the `Replay` gRPC request/response in `proto/lifelog.proto`.
2. **Context Scope (Decision):** As decided, context keys (keys/clicks) should be **filtered to the active window** occurring *during* that frame's time window.
3. **Step Assembly:** The backend should take a `(start, end, interval)` and identify all "Keyframes" (Screen Captures).

## Phase 2: Execution
1. **Replay Step Generator:** In `server/src/query/replay.rs` (create if needed), implement a function that:
    - Queries all `ScreenCapture` records in the window.
    - For each frame `t_i` (representing `[t_i, t_{i+1})`), perform a sub-query for all `KeystrokeFrame`, `ClipboardFrame`, and `ShellHistoryFrame` that overlap that window.
    - Filter those context events to match the `active_window` property of the `ScreenCapture`.
2. **Lazy Hash Response:** Ensure the `ReplayResponse` only contains the `blob_hash` for the screen, not the bytes.
3. **Audio Alignment:** Add a context key for `AudioOverlap` indicating if an audio chunk covers this frame.

## Phase 3: Verification
1. Create a new integration test `server/tests/replay_assembly.rs`.
2. Verify that a keystroke happening during Frame A is correctly attached to Frame A's metadata.
3. Ensure large replay windows don't cause OOM by streaming the response.

## Model Recommendation
**Gemini 2.5 Pro** (Required for interval alignment and window filtering logic).
