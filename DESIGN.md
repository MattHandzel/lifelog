# Design Notes

## Network Topology Dashboard (2026-03-01)

### Scope

- Replaced the legacy Devices page with an interactive Network dashboard.
- Added a visual topology model showing one server node and all connected collector nodes.
- Added collector health/detail controls directly from node selection.

### Architecture Decisions

- Implemented topology rendering with native React + SVG instead of adding a new graph library.
  - Rationale: no additional dependency churn and enough flexibility for glowing edges + pulse animation.
- Used existing Tauri commands as the backend integration surface:
  - `get_system_state`, `get_collector_ids`,
  - `get_component_config`, `set_component_config`.
- Kept alias/icon customization local to the interface (localStorage override) because current backend config RPCs do not expose alias/icon fields.
- Implemented force-sync action as an explicit attempted RPC call (`force_collector_sync`) with surfaced warning when unavailable.

### Data Flow

1. UI loads collector state via `get_system_state` (fallback `get_collector_ids`).
2. UI loads per-collector modality configs for managed modalities.
3. Topology graph renders:
   - edge glow based on online/offline status,
   - pulse animation based on active source state inference.
4. Selecting a node opens controls for:
   - per-modality enable/disable,
   - pause/resume all known modalities,
   - local alias/icon override save.

### Validation

- Added `NetworkTopologyDashboard` unit test for node render + modality update command dispatch.
- Verified via `just test-ui` and `just validate-all`.

## Search Previews (2026-03-01)

### Scope

- Enhance search results with:
  - text snippets around query terms,
  - highlighted term matches,
  - lightweight thumbnails for image modalities.

### Architecture Decisions

- Keep `query_timeline` as the primary key retrieval path.
- Add an interface backend enrichment command:
  - `get_frame_data_thumbnails(keys)` returns frame metadata plus downscaled image previews.
- Perform snippet construction in the frontend from enriched frame fields.
  - Rationale: avoids proto churn and allows UI-level tuning of snippet length and highlight behavior.

### Data Flow

1. UI calls `query_timeline` with text query.
2. UI calls `get_frame_data_thumbnails` for returned keys.
3. UI builds `SearchResult` models with:
   - `snippet`,
   - `highlightTerms`,
   - `preview` (thumbnail data URL for image frames).
4. `ResultCard` renders lazy thumbnail + highlighted snippet.

### Validation

- Added `SearchDashboard` UI tests for:
  - snippet highlighting,
  - thumbnail rendering.
- Verified with `just test-ui` and `just validate`.
