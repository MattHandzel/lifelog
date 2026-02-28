# Plan: Network Topology Dashboard (Interactive Visualizer)

## Objective
Implement a rich, animated visual representation of the entire Lifelog system (Server and Collectors) that acts as an interactive dashboard for system configuration and health monitoring, replacing the basic Devices list.

## Context
From `SPEC.md` Section 11.4:
- The dashboard is located in a "Network" tab.
- It displays the Server and all connected Collectors as visual nodes with customizable icons (Desktop, Laptop, Phone, Cloud).
- Active connections are shown as glowing lines. Data ingests trigger colored light pulses (e.g., blue for screen, green for audio) moving along the lines.
- Hovering/clicking a node reveals health, backlog size, buffer fullness, and last capture timestamp.
- It acts as a graphical interface for `lifelog-config.toml`: users can force data syncs, toggle modalities, pause/resume devices, and modify aliases/icons.

## Phase 1: Research & Strategy
1. **Frontend Architecture:** Review the existing `interface` React application (built with Vite + Tailwind + Tauri). Locate the current Devices dashboard.
2. **Visualization Library:** Choose a suitable library for the node-edge topology visualization. Potential options include `react-flow`, `vis-network`, or D3.js (via a React wrapper). Evaluate which one best supports custom glowing animated edges and custom React nodes.
3. **Backend API Mapping:** Ensure the backend gRPC endpoints exist to fetch:
   - Node status (health, backlog, last capture).
   - Currently active ingest streams (for the colored pulse animations).
   - Configuration updates (pausing, toggling modalities, updating aliases).
4. **Design Aesthetic:** Plan the "dark aesthetic" with CSS variables/Tailwind classes for the glowing effects and node designs.

## Phase 2: Execution
1. **Setup & Scaffolding:** Create the new `NetworkTopology` component and wire it to a new "Network" tab in the main navigation.
2. **Node Rendering:** Implement custom node components. A "Server" node and "Collector" nodes. Add the ability to display the custom icon (Desktop, Laptop, etc.) based on the configuration.
3. **Edge & Animation:** Implement the connections (edges). Create a system to trigger animations (colored pulses) along the edges when the backend reports active data ingestion.
4. **Interactivity (Read):** Add hover/click states to nodes to display the detailed popover/modal with health metrics (buffer fullness, backlog).
5. **Interactivity (Write):** Build the control panel within the node modal to:
   - Trigger `ForceSync` RPC.
   - Toggle specific data modalities (calling a config update RPC).
   - Send `PauseCapture`/`ResumeCapture` commands.
   - Edit the device alias/icon in the config.
6. **Backend Integration:** Connect the UI components to the real gRPC streams. Use WebSockets or server-sent events (via gRPC streaming) if real-time animation of data pulses is required.

## Phase 3: Verification
1. Run `just test-ui` to verify no regressions in the frontend build.
2. Run the interface locally with `just run-server` and `npm run tauri dev`.
3. Manually verify:
   - Nodes appear correctly.
   - Forcing a sync triggers the edge animation.
   - Disabling a modality persists to the backend config.

## AI Token-Efficient Guidelines
- Use `just diff-digest` to summarize your UI changes.
- Use `tools/ai/run_and_digest.sh "npm run build"` to verify the TypeScript compilation without flooding context.

## Model Recommendation
**Gemini 3.1 Pro Preview** (Required for complex React visualization logic and state synchronization).
