# AVA CHEN UX REVIEW: Round 2 (Deep Dive)

**Date:** February 7, 2026
**Focus:** Interface Implementation & "Recall" Reality Check
**Reference:** `AVA_CHEN_UX_REVIEW_ANALYSIS.md` (Round 1)

## Executive Summary

The "Dual Interface" confusion is resolved: **Lifelog is a single Tauri desktop application** (`interface/`) that talks to a central gRPC server (`server/`). The server does *not* serve a competing web UI. This is a solid architectural foundation.

However, the current UI (`v0.1.0`) is structurally a **System Administrator's Dashboard**, not a **Personal Memory Tool**. It exposes implementation details (Collectors, Processes, Config JSON) rather than user goals (Recall, Context, Privacy).

## 1. The "Recall" Gap (Critical)

**Expectation:** "Help me remember what I was doing."
**Reality:** "Here is a list of files and database rows."

### Current State
- **Timeline (`TimelineDashboard.tsx`):** A raw list of events (`uuid`, `modality`, `timestamp`). No visual timeline, no clustering of "moments", no visual context.
- **Search (`SearchDashboard.tsx`):** A file-explorer grid. Good for "find that screenshot," bad for "reconstruct that context."
- **Replay:** Non-existent. You can view a single screenshot, but you cannot "play back" your day.

### Prescription
Refactor `TimelineDashboard` from a list to a **Visual Stream**:
1.  **Swimlanes:** Replace the list with horizontal tracks:
    -   *Context Track:* Active Window Title / App Icon.
    -   *Visual Track:* Screenshot thumbnails (clustered, not every frame).
    -   *Audio Track:* Waveform or "Speech detected" blocks.
2.  **The "Moment" Card:** Group events by time (e.g., 5-minute chunks). Don't show raw UUIDs. Show "Coding in VS Code" or "Browsing YouTube".

## 2. Privacy & Control (Safety Critical)

**Expectation:** "I am in control of what is recorded."
**Reality:** Controls are buried in `Settings -> Collector -> Component`. "Account" tab implies non-existent cloud features.

### Current State
- **No Global Pause:** There is no "Emergency Stop" button in the header or sidebar.
- **Misleading "Account" Page:** `AccountDashboard.tsx` shows "Email Updates" and "Security Alerts". This is a local-first app; these features are deceptive/irrelevant and erode trust in the "local-only" promise.
- **Buried Config:** To disable the camera, a user must navigate: `Settings` -> `Select Collector` -> `Camera` -> `Enabled: false`.

### Prescription
1.  **Global Kill Switch:** Add a persistent toggle in the `Sidebar` or `Header`: *"Recording [ON/OFF]"*.
    -   *Implementation:* This must broadcast a "pause" command to all collectors via the existing gRPC `SetSystemConfig`.
2.  **Delete "Account" View:** Remove `AccountDashboard.tsx`. Replace it with a **"Data & Privacy"** view.
    -   *Content:* Retention settings ("Keep data for 30 days"), Storage usage ("Using 50GB"), and a "Purge Data" danger zone.

## 3. Query Experience

**Expectation:** "Find when I was working on the budget."
**Reality:** Basic text search.

### Current State
- `SearchDashboard.tsx` calls `query_timeline` with a simple string.
- No syntax highlighting, no filters for "App" or "Window Title" (only "Source" and "File Type").
- The underlying `query_timeline` Rust command (`interface/src-tauri/src/main.rs`) supports time ranges, but the UI only exposes them in the `Timeline` view, not the `Search` view.

### Prescription
**Unified "Recall Bar":**
- Merge Search and Timeline search into one global omnibox.
- **Smart Chips:** When typing, suggest filters like `app:Chrome`, `has:Audio`.
- **Natural Language Stub:** Even if the backend isn't ready, the UI should frame the input as "Ask your history" to set the mental model.

## 4. Device Management

**Expectation:** "Manage my devices."
**Reality:** Buried in Settings.

### Current State
- **Functional Backend:** `SettingsDashboard.tsx` successfully fetches `get_collector_ids` and can push config. This is a huge win.
- **Poor UI:** It uses a dropdown selector inside a generic Settings page. It feels like debug UI.

### Prescription
**Promote to Top-Level "Devices" View:**
- Move "Collectors" out of Settings.
- Show a card for each Device/Collector:
    -   *Status:* Online/Offline (Heartbeat).
    -   *Active Modalities:* Icons for Mic/Screen/etc. (Green = recording, Gray = off).
    -   *Quick Actions:* "Pause this device".

## 5. Specific Code Actions

### `interface/src/components/FeatureTabs.tsx`
- **Rename:** Change "Modules" to "Explore" or "Activity".
- **Reorganize:** Hide "Processes", "Mouse", "Keyboard" into a "Debug/System" submenu. Normal users don't want to see a "Process Dashboard" as a primary activity.

### `interface/src/App.tsx`
- **Navigation:**
    -   Remove `Account` (User icon).
    -   Add `Devices` (Monitor/Laptop icon).
    -   Add `Privacy` (Shield icon).

### `interface/src/components/SettingsDashboard.tsx`
- **Refactor:** Extract the "Collector Configuration" logic into the new `Devices` view. Keep `Settings` for app-local preferences (Theme, etc.).

## Summary of Work Required
1.  **Delete** `AccountDashboard` (Code rot/Misleading).
2.  **Create** `DevicesDashboard` (utilizing `get_collector_ids` and `set_component_config` properly).
3.  **Refactor** `TimelineDashboard` to use visual cards instead of a table.
4.  **Add** Global "Pause" Toggle in `Sidebar.tsx`.
