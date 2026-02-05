# AVA CHEN UX REVIEW: Lifelog User-Facing Product Analysis

This review focuses on the user-facing implications of the v1 spec in `SPEC.md`, and sanity-checks it against the current UI/code reality in `interface/` and the current server integration approach described in `server/README.md` and `grpc-frontend.md`.

## Executive Takeaways

1. The v1 spec’s core promise is “recall” via timeline + search + replay, plus cross-modal correlation. The current UI is organized as “logger dashboards” and a basic file search, which does not match the recall mental model (`SPEC.md:1.1`, `SPEC.md:11.1`).
2. Raw SQL-like query framing (see `docs/search.md`) is not accessible for most users and is also risky if it leaks into the UI. v1 needs a progressive disclosure query experience: simple search first, templates next, advanced deterministic query last (`SPEC.md:10`).
3. Privacy controls are acknowledged in the spec but not productized: users need an always-visible “what is being collected, right now” surface plus time-range deletion and pause controls (`SPEC.md:12.3`).
4. Multi-device is central to the architecture, but the UI currently mixes “server REST endpoints” and “Tauri gRPC calls,” and even hardcodes a collector id in at least one place (`interface/src/components/ScreenDashboard.tsx`). This will feel brittle and confusing.

## Product Mental Model (What Users Think They’re Using)

User goal: “Help me find moments in my past.”

The spec’s architecture terms (collector, stream, buffer, transform) are implementation concepts. The UI should translate them into:

- Devices (collector) with last-seen + health.
- Data sources (streams) as “Screen,” “Microphone,” “Browser,” “Apps,” “Clipboard,” etc.
- Sync/backlog (buffering) as “You have X hours pending from Device Y.”
- Derived data (transforms) as “Searchable text from screens (OCR)” with status.

Recommendation: keep engineering nouns available, but behind an “Advanced details” expander. The default UI should stay in the user’s recall language.

## Query Interface

### Current Spec Direction

`SPEC.md:10` defines a deterministic query language with correlation operators, compiled to a typed plan. That is good for correctness and reproducibility, but the spec does not define how normal people will author these queries.

`docs/search.md` explicitly mentions raw SQL-like queries. That is a strong signal the current concept is too low-level for the target “zero maintenance” goal (`SPEC.md:1.3`).

### Recommendation: 3-Layer Query UX (Progressive Disclosure)

1. **Simple search (default):** one search bar that “just works” across text-bearing streams.
2. **Templates (bridge):** a “recipe” picker for common cross-modal intents.
3. **Advanced query (power):** a structured builder and a text DSL view, with deterministic compilation.

Concrete UI pattern:

- Single omnibox at the top of the app: `Search anything...`
- Inline “chips” for time range, device, modality.
- A “Build query” button that opens a side panel.
- A “Show compiled query plan” toggle for trust/debugging.

### Visual Builder: Make Correlation Comprehensible

Cross-modal correlation is the signature capability (`SPEC.md:1.2`, `SPEC.md:4.3`). Users need to “feel” what correlation means.

Builder proposal:

- Step 1: “Return” stream (what you want back): Screen frames, Audio chunks, URLs, Commands, Clipboard.
- Step 2: “During times when” conditions are true.
- Step 3: For each condition: pick a stream + operator + value.
- Step 4: Correlation window control (defaults): `Within ±30s` with per-condition override (`SPEC.md:4.3`).

Make time explicit in the UI:

- Show a small timeline preview of matched windows.
- Let users drag to expand/shrink the window for “within” matching.

### Natural Language (Optional) Without Breaking Determinism

Issue #12 in `SPEC.md:17.1` hints at NL-to-query. That’s useful, but only if the UI makes the mapping explicit:

- NL input produces a structured query draft.
- The app displays “Interpreted as:” with editable structured predicates.
- The final execution uses the deterministic plan (`SPEC.md:10.1`).

This is how you keep power and approachability without making results feel “random.”

### Templated Queries (Start Here)

To satisfy “what was I listening to yesterday while coding?” you need first-class templates:

- “What was I listening to while…” (Return: audio, During: app/window predicates, Time: yesterday)
- “Show me what I saw when…” (Return: screen, During: browser URL/title, OCR contains)
- “Find the command I ran when…” (Return: shell history, During: active app + time)

Each template should be editable and should teach the user the query model over time.

## Data Exploration (Timeline + Search + Replay)

### Spec Requirement

The v1 UI requirements in `SPEC.md:11.1` are:

- timeline navigation (jump by time, filter by modality/stream/device)
- search box for query language + quick filters
- result list with previews
- replay view with aligned context

### Current UI Reality

In `interface/src/components/FeatureTabs.tsx`, exploration is framed as “modules” (screenshots, camera, microphone, processes). That supports “is the logger working?” but not “help me recall.”

`interface/src/components/SearchDashboard.tsx` is a paginated file-like search with filters by type/source, but not a time-first exploration model.

### Recommendation: Two Primary Exploration Modes

1. **Timeline (primary):** time-first browsing with modality overlays.
2. **Search (secondary):** query-first, returns “moments” that can be opened in replay.

#### Timeline View

Core components:

- A time scrubber with zoom (day, hour, minute).
- A multi-lane view per modality (Screen, Audio, Browser, Apps, Commands, Clipboard).
- “Moments” are clusters: e.g., 2 minutes of activity becomes a selectable segment.
- Filters: device, modality, app, location (later), “only when I was active” (based on input bursts).

#### Replay View

Replay is where cross-modal context pays off (`SPEC.md:10.3`, `SPEC.md:11.1`).

Minimum viable replay:

- Screen frames stepper (with play controls).
- Audio aligned playback (even if coarse).
- Event rail: key bursts, clipboard events, shell commands, URL changes within the current window.
- “Jump to previous/next matched moment” navigation.

Important: replay should feel like “time travel,” not like opening files.

## Device Management

### What Users Need (Not “Collectors”)

Users need a “Devices” page that answers:

- Which devices are connected?
- What’s being collected on each device?
- Is it healthy? last seen? backlog?
- What changed recently? (config updates, failures, pauses)

### Spec Support

`SPEC.md:7.2` includes `RegisterCollector`, `ReportState`, `SuggestUpload`, config push, pause/resume. That is exactly the data needed to power a device UI.

### Recommendation: Devices Page Information Architecture

Top-level list:

- Device name (user-editable) + platform icon
- Online/offline state
- Last capture time per stream (simple badge list)
- Backlog estimate (time and/or storage)
- Errors/warnings (clickable)

Device detail:

- Data sources toggles (per stream enable/disable)
- Capture rates (intervals)
- Storage budget (buffer size, free disk)
- “Pause all capture” emergency control
- Audit log (last 50 actions)

Note: the current UI has a collector selector in `interface/src/components/SettingsDashboard.tsx`, but it’s not integrated into the main exploration model, and `interface/src/components/ScreenDashboard.tsx` hardcodes a collector id, undermining multi-device UX.

## Privacy Dashboard (Local-First Trust Must Be Visible)

### Spec Minimum

`SPEC.md:12.3` requires:

- emergency pause/resume capture
- per-stream disable
- retention controls (coarse-grained)

### Recommendation: Privacy As a First-Class Surface

Add a dedicated “Privacy” page (or a persistent header control) that answers:

- What is being collected right now?
- What data exists on this machine?
- How long do we keep it?
- How do I delete it, precisely?

#### Required Controls

- Global “Pause capture” toggle (all devices) with a clear status indicator.
- Per-stream pause toggles (Screen, Audio, Browser, etc.).
- Retention presets:
  - Keep forever
  - Keep 30 days
  - Keep until disk low
- “Delete data” workflow:
  - Select device(s) + streams + time range
  - Preview counts and estimated storage reclaimed
  - Type-to-confirm (high stakes)
  - Optional “cryptographic wipe” semantics later, but start with reliable deletion.

#### Trust UX

Local-first privacy needs explicit copy:

- “Runs locally. No cloud sync in v1.” (`SPEC.md:1.4`)
- “Data never leaves your machine unless you explicitly export.” (even if export is v2, set expectation)

## Onboarding (First-Time Setup)

The spec says “passive capture, zero maintenance” after setup (`SPEC.md:1.3`). That makes onboarding disproportionately important.

### Recommended Onboarding Steps (v1)

1. **Choose where data lives** (backend machine), show storage expectations.
2. **Add first device**:
  - Pairing via QR or one-time token (`SPEC.md:12.2`)
  - Name the device
3. **Pick what to collect**:
  - A simple checklist with plain-language consequences
  - “Recommended defaults” for common personas
4. **Privacy quickstart**:
  - Teach pause button
  - Teach time-range deletion
5. **First query moment**:
  - Prompt: “Try: what were you doing yesterday at 3pm?”
  - Show results in timeline/replay immediately

This is where you win the “effortless recall” promise.

## Notifications (Failures Must Be Quiet, But Obvious)

`SPEC.md:1.3` calls for “quiet alerts (health/status), not interactive debugging tasks.”

Current UI has “Notification Preferences” in `interface/src/components/AccountDashboard.tsx`, but it’s not backed by system events, and it’s “account/email” oriented, which conflicts with “local-first, no cloud.”

### Recommendation: In-App Notification Center + Status Bar

1. Persistent status indicator:
  - “All good” or “Needs attention” with a count of issues.
2. Notification center categories:
  - Sync/backlog warnings
  - Collector offline
  - Capture failures (permissions revoked, device unavailable)
  - Storage low
  - Transform backlog large (OCR lag)
3. Each notification should have:
  - what happened
  - impact (data gap?) and exact time window affected
  - one-click fix when possible

Avoid email-first defaults; local desktop notifications can be optional.

## “What Was I Listening To Yesterday While Coding?” (Make This 10 Seconds)

Target interaction (ideal):

1. Open app, omnibox focused.
2. Type: `yesterday coding audio`
3. App suggests template: “Audio during coding (apps: VS Code, Terminal)” with an editable “coding = …” definition.
4. Results appear as a few “moments” with:
  - time range
  - top app/window
  - optional OCR highlights (“TODO”, “build”, repo name)
5. Click a result:
  - replay opens at that time
  - audio plays aligned to screen frames

What must exist under the hood:

- An “active app/window” stream (spec requires it, `SPEC.md:3.1`).
- A way to define “coding” as an app/window predicate (no ML required).
- Audio chunks as interval records (`SPEC.md:4.1`) so correlation feels correct.

## Architecture vs UX: Where It Currently Leaks

The current interface mixes three integration models:

- REST logger endpoints (`server/README.md`, `interface/src/lib/api.ts`, many dashboards use `/api/logger/...`)
- Tauri commands that talk to gRPC (`grpc-frontend.md`, `interface/src/components/ScreenDashboard.tsx`, `interface/src/components/SettingsDashboard.tsx`)
- A separate `/api/search` endpoint used by `interface/src/components/SearchDashboard.tsx` that is not reflected in `server/README.md` (at minimum, it increases product uncertainty).

UX consequence: users will experience inconsistent behavior depending on which module they’re in. Even if this is “developer scaffolding,” it sets the wrong product shape.

Recommendation: choose one canonical UI-facing API surface as the spec already recommends in `SPEC.md:16.3`:

- Either the backend serves an HTTP API tailored to the UI (recommended for web UI and phone support), with gRPC reserved for collector transport.
- Or the UI speaks gRPC directly, but then “phone browser” is substantially harder (`SPEC.md:11`).

## MVP Priorities (User-Facing)

If v1 is “recall UI only,” prioritize these in order:

1. Unified omnibox search with time range + modality + device filters (simple mode).
2. Timeline view with cross-modal lanes and “moments.”
3. Replay view (screen stepper + aligned events; audio optional but high value).
4. Devices page (online/offline, last seen, backlog, per-stream toggles).
5. Privacy dashboard (pause, per-stream disable, time-range deletion, retention presets).
6. Templates for cross-modal queries, then an advanced builder/DSL.

Everything else can be framed as “module health pages” but should not be the primary navigation.

## Concrete UI Changes Suggested for the Current Codebase (Directional, Not Implementation)

Based on what’s present in `interface/src/components/*`:

- Reframe `FeatureTabs` (`interface/src/components/FeatureTabs.tsx`) from “Modules” to “Explore”:
  - Explore: Timeline, Search, Replay
  - Manage: Devices, Privacy, Settings
  - Keep logger dashboards under an “Advanced” section.
- Replace “Account” with “Profile (optional)” and move notifications to a system Notification Center.
- Remove hard-coded device assumptions:
  - `interface/src/components/ScreenDashboard.tsx` should not pin to a specific collector id; that breaks the multi-device promise.

## Open Questions (Worth Resolving Early)

1. Is “Interface” primarily a web app served by backend (`SPEC.md:11`), or a desktop Tauri app? Current repo suggests both. The UX and onboarding differ substantially.
2. What is the canonical query representation exposed to the UI? The spec defines a compiler/planner, but the UI needs stable primitives: streams, predicates, correlation windows, and a “return” stream.
3. What are the exact privacy guarantees around “keystrokes” and “clipboard”? These are high-risk modalities; onboarding and privacy UI must handle them carefully (`SPEC.md:3.1`, `SPEC.md:12`).

