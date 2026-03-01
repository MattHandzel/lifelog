# Plan: Replay UI (Frame Stepper + Context Overlays)

## Objective
Build the frontend Replay interface that allows users to step through time with frame-synchronized event overlays.

## Context
From `SPEC.md` Section 11.1:
- Replay view: step-by-step screen frames, optionally audio playback, visible aligned events (clipboard, key bursts, commands).
- Interface: Vite + React + TypeScript.

## Phase 1: Research & Strategy
1. **API Integration:** Review the `Replay` RPC implementation in the backend (verified in Phase 1).
2. **Buffering Model:** Design the "YouTube-style" buffering system where metadata steps are fetched first, and image blobs are batch-fetched in the background.
3. **Overlay Design:** Design the React components for displaying keystrokes and clipboard events as overlays on top of the screen frame.

## Phase 2: Execution
1. **State Management:** Implement a `useReplay` hook to manage the playback state (current index, buffered frames, playing status).
2. **Frame Component:** Create a `ReplayFrame` component that displays the image blob and the active window metadata.
3. **Context Overlays:** Implement components to render keystroke "bursts" and clipboard snippets, filtered by the active window (as per Section 18.9).
4. **Playback Controls:** Build the scrubber, play/pause, and step-forward/backward buttons.
5. **Batch Prefetching:** Implement the logic to batch-fetch image blobs based on the current playback position.

## Phase 3: Verification
1. Run `just test-ui`.
2. Manually verify playback sync: ensure keystrokes appear exactly when the corresponding frame is visible.
3. Verify performance: ensure smooth transitions between frames when buffered.

## AI Token-Efficient Guidelines
- Use `just diff-digest` to summarize UI changes.
- Use `tools/ai/run_and_digest.sh "npm run build"` for type-checking.

## Model Recommendation
**Gemini 3.1 Pro Preview** (Required for complex synchronization logic).
