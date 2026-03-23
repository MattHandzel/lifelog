# Lifelog Configuration Reference

All configuration lives in a single TOML file. Default path: `~/.config/lifelog/lifelog-config.toml`.

Override with: `LIFELOG_CONFIG_PATH=/path/to/config.toml`

## Precedence

**Environment variable > TOML config > built-in default**

All env vars remain supported as overrides for backward compatibility.

## File References

Any string value can use `@` prefix to read from a file:

```toml
[transforms.params]
api_key = "@/run/agenix/openrouter-key"
```

The file contents are read at startup, trimmed, and used as the value.

## `[server]`

| Key | Type | Default | Env Override | Description |
|-----|------|---------|-------------|-------------|
| `host` | string | `"localhost"` | — | Bind address |
| `port` | u32 | `7182` | — | gRPC listen port |
| `databaseEndpoint` | string | `"postgresql://lifelog@127.0.0.1:5432/lifelog"` | — | Legacy DB endpoint field |
| `databaseName` | string | `"main"` | — | Legacy DB name field |
| `serverName` | string | `"LifelogServer"` | — | Server display name |
| `casPath` | string | `"~/lifelog/cas"` | — | Content-addressable store path |
| `defaultCorrelationWindowMs` | u64 | `30000` | — | Default temporal correlation window |
| `retentionPolicyDays` | map | `{}` | — | Per-modality retention (`"Screen" = 90`) |
| `postgresUrl` | string | — | `LIFELOG_POSTGRES_INGEST_URL` | PostgreSQL connection string (required) |
| `postgresMaxConnections` | usize | `16` | `LIFELOG_POSTGRES_INGEST_MAX_CONNECTIONS` | Connection pool size |
| `tlsCertPath` | string | — | `LIFELOG_TLS_CERT_PATH` | TLS certificate path |
| `tlsKeyPath` | string | — | `LIFELOG_TLS_KEY_PATH` | TLS private key path |
| `allowPlaintext` | bool | `false` | `LIFELOG_ALLOW_PLAINTEXT` | Allow unencrypted gRPC |
| `allowedHosts` | string[] | `[]` | — | External hosts transforms may contact. Empty = local-only |

## `[postgres]`

Optional section. Settings here are fallbacks for postgres-related keys not set under `[server]`.

| Key | Type | Default | Env Override | Description |
|-----|------|---------|-------------|-------------|
| `maxConnections` | usize | `16` | `LIFELOG_POSTGRES_INGEST_MAX_CONNECTIONS` | Connection pool size (also settable as `server.postgresMaxConnections`) |

## `[[transforms]]`

Each transform is an entry in the `[[transforms]]` array.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `id` | string | required | Unique transform identifier |
| `enabled` | bool | required | Enable/disable this transform |
| `transformType` | string | same as `id` | Transform type: `ocr`, `stt`, `llm`, `activity-classifier`, `browser-topic`, `sound-classifier` |
| `sourceOrigin` | string | required | Source data pattern (`"*:Screen"`, `"device-id:Audio"`) |
| `serviceEndpoint` | string | `""` | HTTP endpoint for the transform service |
| `language` | string | `"eng"` | Language code (for OCR) |
| `priority` | u32 | `0` | Execution priority (lower = earlier) |
| `destinationModality` | string | — | Override output modality name |
| `privacyLevel` | string | `"standard"` | `local_only`, `zdr`, or `standard` (see Privacy) |

### `[transforms.params]`

Key-value pairs passed to the transform executor. Common params:

| Key | Used By | Description |
|-----|---------|-------------|
| `model` | stt, llm, activity, browser-topic | Model name/ID |
| `system_prompt` | llm | System prompt for LLM transforms |
| `timeout_secs` | stt, llm | Request timeout |
| `api_key` | llm | API key (supports `@` file reference) |

## `[collectors.<id>]`

Each collector is keyed by its device ID.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `host` | string | required | Server address to connect to |
| `port` | u32 | required | Server port |
| `timestampFormat` | string | required | Timestamp format string |
| `id` | string | section key | Collector identifier |

### Modality sub-sections

Each modality is a sub-table of the collector (e.g., `[collectors.laptop.screen]`).

**Common fields** (all modalities):

| Key | Type | Description |
|-----|------|-------------|
| `enabled` | bool | Enable/disable collection |
| `interval` | f64 | Collection interval in seconds |
| `outputDir` | string | Local output directory (supports `~`) |

**`screen`**: `program` (screenshot tool, e.g. `"grim"`)

**`browser`**: `inputFile`, `outputFile`, `browserType` (`"chrome"` or `"firefox"`)

**`camera`**: `device`, `resolutionX`, `resolutionY`, `fps`

**`microphone`**: `sampleRate`, `chunkDurationSecs`, `bitsPerSample`, `channels`, `captureIntervalSecs`

**`keyboard`**: (no extra fields)

**`hyprland`**: `logClients`, `logActivewindow`, `logWorkspace`, `logActiveMonitor`, `logDevices`

**`processes`**: (no extra fields)

## Privacy

### Privacy Levels (per transform)

| Level | Meaning |
|-------|---------|
| `local_only` | Data never leaves the machine. Only local endpoints (localhost/LAN) allowed. |
| `zdr` | Zero data retention. External services allowed but must not store data. |
| `standard` | Default. External services allowed per `allowedHosts`. |

### Privacy Tiers (per modality)

| Tier | Modalities | Allowed Transforms |
|------|------------|-------------------|
| **Sensitive** | Keystrokes, Audio, Clipboard, Microphone | `local_only` only |
| **Moderate** | Screen, Browser, OCR, WindowActivity, Camera | `local_only` or `zdr` |
| **Low** | Weather, Processes, ShellHistory, Mouse, Hyprland, etc. | Any |

## Runtime Environment Variables

| Variable | Description |
|----------|-------------|
| `LIFELOG_CONFIG_PATH` | Override config file path |
| `LIFELOG_COLLECTOR_ID` | Select which collector section to use |
| `LIFELOG_TRANSFORMS_JSON` | Override transforms from JSON (overrides TOML) |
| `LIFELOG_AUTH_TOKEN` | Authentication token |
| `LIFELOG_ENROLLMENT_TOKEN` | Collector enrollment token |

## Example

```toml
[server]
host = "0.0.0.0"
port = 7182
serverName = "LifelogServer"
casPath = "/home/user/lifelog/cas"
postgresUrl = "postgresql://lifelog@127.0.0.1:5432/lifelog"
tlsCertPath = "/etc/lifelog/cert.pem"
tlsKeyPath = "/etc/lifelog/key.pem"
allowedHosts = ["api.openai.com"]

[collectors.my-laptop]
host = "100.118.206.104"
port = 7190
timestampFormat = "%Y-%m-%d_%H-%M-%S.%3f%Z"

[collectors.my-laptop.screen]
enabled = true
interval = 10.0
outputDir = "~/lifelog/data/screen"
program = "grim"

[collectors.my-laptop.microphone]
enabled = true
outputDir = "~/lifelog/data/microphone"
sampleRate = 16000
chunkDurationSecs = 300
bitsPerSample = 16
channels = 1
captureIntervalSecs = 300

[[transforms]]
id = "ocr"
enabled = true
sourceOrigin = "*:Screen"
language = "eng"
privacyLevel = "local_only"

[[transforms]]
id = "stt"
enabled = true
transformType = "stt"
sourceOrigin = "*:Audio"
serviceEndpoint = "http://localhost:47770"
privacyLevel = "local_only"
priority = 1
[transforms.params]
model = "Systran/faster-whisper-large-v3"

[[transforms]]
id = "llm-cleanup"
enabled = true
transformType = "llm"
sourceOrigin = "*:Transcription"
serviceEndpoint = "http://localhost:11434"
privacyLevel = "local_only"
priority = 2
[transforms.params]
model = "gemma3:4b-it-qat"
api_key = "@/run/agenix/openrouter-key"
```
