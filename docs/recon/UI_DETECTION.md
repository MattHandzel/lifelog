# UI Detection Report

## UI Type

**Tauri v2 Desktop Application** (WebKitGTK on Linux)

- Identifier: `com.lifelog.app`
- Product name: `lifelog`
- Version: 0.1.0
- Default window: 1920x1080, resizable

## Frontend Stack

- Vite dev server on `http://localhost:1420`
- React 18 + TypeScript
- Tailwind CSS + Radix UI primitives (shadcn/ui)
- Tauri v2 API (`@tauri-apps/api`, `@tauri-apps/plugin-*`)

## Views / Screens (from App.tsx)

| View ID | Component | Description |
|---------|-----------|-------------|
| `dashboard` | `FeatureTabs` | Main modality dashboards (tabbed) |
| `search` | `SearchDashboard` | Full-text + semantic search |
| `network` | `NetworkTopologyDashboard` | Device/collector topology |
| `settings` | `SettingsDashboard` | Configuration management |

## Dashboard Components (modality tabs)

| Component | File |
|-----------|------|
| ScreenDashboard | `interface/src/components/ScreenDashboard.tsx` |
| CameraDashboard | `interface/src/components/CameraDashboard.tsx` |
| MicrophoneDashboard | `interface/src/components/MicrophoneDashboard.tsx` |
| ProcessesDashboard | `interface/src/components/ProcessesDashboard.tsx` |
| DevicesDashboard | `interface/src/components/DevicesDashboard.tsx` |
| TextUploadDashboard | `interface/src/components/TextUploadDashboard.tsx` |
| ReplayDashboard | `interface/src/components/ReplayDashboard.tsx` |
| TimelineDashboard | `interface/src/components/TimelineDashboard.tsx` |
| SearchDashboard | `interface/src/components/SearchDashboard.tsx` |
| NetworkTopologyDashboard | `interface/src/components/NetworkTopologyDashboard.tsx` |
| SettingsDashboard | `interface/src/components/SettingsDashboard.tsx` |

## Navigation

- Sidebar with icon+label links (Dashboard, Search, Network, Settings)
- Global pause/unpause toggle (broadcasts to all collectors via Tauri invoke)
- Login gate component (`Login.tsx`)

## Launch Methods

### Development
```bash
# Terminal 1: Vite dev server only (browser)
cd interface && npm run dev
# Opens at http://localhost:1420

# Terminal 2: Full Tauri app (desktop window)
cd interface && npm run tauri dev
```

### Production
```bash
nix develop --command cargo build --release -p lifelog-server-frontend
# Binary at target/release/lifelog-server-frontend
```

## CSP / Network

- Connects to gRPC server at `https://YOUR_SERVER_IP:7182` (Tailscale) or `http://localhost:8080`
- WebSocket for Vite HMR at `ws://localhost:1420`
- MinIO/S3 at `http://localhost:9000`

## Tauri Plugins Used

- `@tauri-apps/plugin-dialog` - Native file/message dialogs
- `@tauri-apps/plugin-fs` - Filesystem access (scoped)
- `@tauri-apps/plugin-opener` - URL/file opener
- `@tauri-apps/plugin-shell` - Shell command execution
