# Golden Workflows

These are the core user journeys through the lifelog system, ranked by importance. Each workflow represents a documentation + demo target.

## Tier 1: Essential (must document first)

### 1. First-Time Setup
**Persona:** New user
**Flow:** Clone repo -> `nix develop` -> configure `lifelog-config.toml` -> start PostgreSQL -> `just run-server` -> verify health
**Key screens:** Terminal output showing server startup, gRPC reflection
**Docs needed:** Quickstart guide, config reference

### 2. Start Collecting Data
**Persona:** User with running server
**Flow:** Configure collector modalities (screen, camera, mic, processes) -> `just run-collector` -> verify data flowing -> check frame count
**Key screens:** Terminal logs showing collector upload, server frame counts
**Docs needed:** Collector setup, modality enable/disable guide

### 3. Search Your Lifelog
**Persona:** User with collected data
**Flow:** Open UI -> navigate to Search -> enter query -> view results with screenshots/text -> filter by date/modality
**Key screens:** SearchDashboard with results, ResultCard detail view
**Docs needed:** Search guide, query syntax reference

### 4. Browse Timeline / Replay
**Persona:** User reviewing their day
**Flow:** Open UI -> Dashboard -> select date range -> browse TimelineDashboard -> switch to ReplayDashboard for video-like playback
**Key screens:** TimelineDashboard, ReplayDashboard with frame scrubbing
**Docs needed:** Timeline usage, replay controls

## Tier 2: Important (document next)

### 5. Deploy Server + Collector (Split)
**Persona:** Power user with home server
**Flow:** Install NixOS module on server -> configure TLS -> set up collector on laptop -> verify Tailscale connectivity -> enable systemd services
**Key screens:** systemctl status outputs, config file examples
**Docs needed:** Deployment runbook (partially exists in USAGE.md section 11)

### 6. Export Data
**Persona:** User wanting data portability
**Flow:** Run `lifelog-export` -> select date range / modalities -> export to local files
**Key screens:** CLI output, exported file structure
**Docs needed:** Export CLI reference

### 7. OCR Transform Pipeline
**Persona:** User wanting searchable screenshots
**Flow:** Enable OCR transform in config -> collect screenshots -> server auto-runs Tesseract -> search for text visible in screenshots
**Key screens:** Config snippet, search results with OCR text highlights
**Docs needed:** Transform configuration guide

### 8. AI Integration (MCP)
**Persona:** Developer integrating lifelog with AI tools
**Flow:** Start `lifelog-mcp` -> connect Claude/other MCP client -> query lifelog data via natural language
**Key screens:** MCP client showing lifelog tools
**Docs needed:** MCP setup guide

## Tier 3: Supplementary

### 9. Monitor Network Topology
**Persona:** Multi-device user
**Flow:** Open UI -> Network view -> see connected collectors -> check device status
**Key screens:** NetworkTopologyDashboard
**Docs needed:** Brief section in UI guide

### 10. Manage Settings
**Persona:** Any user
**Flow:** Open UI -> Settings -> configure server connection, privacy tiers, retention
**Key screens:** SettingsDashboard
**Docs needed:** Settings reference
