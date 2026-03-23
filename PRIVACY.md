# Privacy and Security Model

Lifelog is a personal data capture and retrieval system. All data is stored locally by default. This document describes what is collected, how it is stored, what controls exist over data flow, and what leaves the machine.

---

## Table of Contents

1. [Data Collected Per Modality](#data-collected-per-modality)
2. [Storage Model](#storage-model)
3. [Privacy Tiers](#privacy-tiers)
4. [Transform Pipeline Privacy](#transform-pipeline-privacy)
5. [Network Egress Controls](#network-egress-controls)
6. [Authentication and Access Control](#authentication-and-access-control)
7. [Transport Security (TLS)](#transport-security-tls)
8. [What Leaves the Machine](#what-leaves-the-machine)
9. [Recommendations](#recommendations)

---

## Data Collected Per Modality

Each modality is enabled independently in the collector config. No modality is enabled by default; you opt in explicitly.

| Modality | What Is Captured | Notes |
|----------|-----------------|-------|
| **Screen** (`screen`) | Periodic screenshots of the display | Configurable interval (e.g. every 10 s). Captured via a tool like `grim`. |
| **Browser** (`browser`) | Browser history entries (URL, title, timestamp) | Reads browser history file; supports Chrome and Firefox. |
| **Camera** (`camera`) | Periodic frames from a webcam | Resolution and FPS are configurable. |
| **Microphone / Audio** (`microphone`) | Raw audio chunks from the microphone | WAV format, configurable sample rate and chunk duration. |
| **Keyboard / Keystrokes** (`keyboard`) | Key events (scan codes / symbols) | Records which keys are pressed, not application-level text composition. |
| **Process List** (`processes`) | Snapshot of running processes | Name, PID, CPU/mem usage at collection time. |
| **Window Manager State** (`hyprland`) | Active window, workspace, monitor, connected clients | Specific to Hyprland; logs are configurable per field. |

Collectors run on the device where data originates. They transmit frames to the server over gRPC.

---

## Storage Model

All data is stored locally on the machine running the Lifelog server.

### Content-Addressable Store (CAS)

Binary payloads (screenshots, audio chunks, camera frames) are stored in a content-addressable store on disk. The default path is `~/lifelog/cas`, configurable via `casPath` in `lifelog-config.toml`. Each blob is referenced by its `blob_hash` (SHA-256). Files with identical content share a single on-disk copy.

### PostgreSQL — Unified `frames` Table

All modality metadata is stored in a single `frames` table in PostgreSQL. Each row contains:

- Origin identifier (device + modality)
- Timestamp
- `blob_hash` — reference into the CAS for binary data
- `payload` — JSONB column holding modality-specific structured fields (e.g. OCR text, browser URL, process list)

A `catalog` table tracks registered data origins (devices and their modalities).

Transform outputs (OCR text, speech transcriptions, LLM summaries) are stored as additional frames linked to the source frame, never replacing the original.

### Retention

Per-modality retention policies are configurable in `lifelog-config.toml`:

```toml
[server]
retentionPolicyDays = { "Screen" = 90, "Audio" = 30 }
```

Frames older than the configured window are eligible for pruning.

---

## Privacy Tiers

Every data modality is assigned a privacy tier that controls which transforms may process it. This is enforced at the transform worker level — a transform whose `privacyLevel` does not satisfy the source tier's requirements is rejected before execution.

| Tier | Modalities | Allowed Transform Privacy Levels |
|------|-----------|----------------------------------|
| **Sensitive** | Keystrokes, Audio, Clipboard, Microphone | `local_only` only |
| **Moderate** | Screen, Browser, OCR output, WindowActivity, Camera | `local_only` or `zdr` |
| **Low** | Weather, Processes, ShellHistory, Mouse, Hyprland state, and all others | Any (`local_only`, `zdr`, or `standard`) |

The enforcement rule in code:

- `Sensitive` data: only transforms marked `local_only` may run. No external service contact permitted.
- `Moderate` data: `local_only` or `zdr` (zero-data-retention) transforms permitted. `standard` transforms (which may contact arbitrary external hosts) are blocked.
- `Low` data: all privacy levels permitted.

This means that even if a user misconfigures a transform to send keystroke data to an external API, the server will reject the transform at runtime before any data is sent.

---

## Transform Pipeline Privacy

Transforms enrich raw frames (e.g. OCR on screenshots, speech-to-text on audio, LLM summarisation). Each transform declares a `privacyLevel`:

| Level | Meaning |
|-------|---------|
| `local_only` | Data never leaves the machine. Only localhost or LAN endpoints are contacted. |
| `zdr` | External services may be contacted but must operate with zero data retention. |
| `standard` | External services may be contacted per the `allowedHosts` allowlist. |

Additional pipeline controls:

- **Idempotency**: Transforms are tracked by source frame + transform ID. A frame that has already been successfully transformed will not be re-processed, preventing duplicate data and redundant external calls.
- **Cost controls**: Per-transform budget limits prevent runaway API usage.
- **Validation**: Transform output is validated before being committed to the frames table.
- **No silent failures**: If a transform fails, an error is surfaced and logged. There is no silent fallback that discards failures.

---

## Network Egress Controls

By default, the Lifelog server operates in **local-only mode**. No outbound connections to external hosts are made unless explicitly configured.

External hosts must be allowlisted in `lifelog-config.toml`:

```toml
[server]
allowedHosts = ["localhost:47770", "localhost:11434"]
```

The egress check runs at startup and at transform execution time. A transform whose `serviceEndpoint` resolves to a host not in `allowedHosts` is blocked. An empty `allowedHosts` list means all external connections are denied.

`local_only` transforms additionally enforce that the endpoint hostname is `localhost` or a loopback/LAN address, regardless of what appears in `allowedHosts`.

---

## Authentication and Access Control

The gRPC API requires a bearer token for all calls. The token is configured via:

```
LIFELOG_AUTH_TOKEN=<secret>
```

Collectors authenticate to the server using an enrollment token:

```
LIFELOG_ENROLLMENT_TOKEN=<secret>
```

These are environment variable overrides. Store them outside the config file (e.g. via a secrets manager or a file with restricted permissions).

There is no multi-user model. Lifelog is a single-owner system; the assumption is that the server is operated by and for one person.

---

## Transport Security (TLS)

The gRPC server supports TLS. Configure the certificate and key paths:

```toml
[server]
tlsCertPath = "/etc/lifelog/cert.pem"
tlsKeyPath  = "/etc/lifelog/key.pem"
```

Plaintext (unencrypted) connections are disabled by default. To allow plaintext (e.g. during local development):

```toml
[server]
allowPlaintext = true
```

TLS should be enabled for any deployment where the collector and server communicate over a network, including a local area network.

---

## What Leaves the Machine

By default, **nothing leaves the machine**. All capture, storage, and query happens locally.

The only data that can leave the machine is data passed to transform service endpoints that you have explicitly configured and allowlisted. In a typical local-only deployment:

| Service | What Is Sent | Condition |
|---------|-------------|-----------|
| **Faster Whisper** (`http://localhost:47770`) | Audio chunks for speech-to-text | Only if the `stt` transform is enabled and endpoint is allowlisted |
| **Ollama** (`http://localhost:11434`) | Text (OCR output, transcriptions) for LLM processing | Only if an `llm` transform is enabled and endpoint is allowlisted |

Both of these services run locally by default. If you configure a remote endpoint (e.g. an OpenAI API URL), that host must appear in `allowedHosts` and the transform's `privacyLevel` must satisfy the source modality's tier requirement.

There is:
- No telemetry
- No crash reporting
- No analytics
- No cloud backup
- No automatic updates that phone home

---

## Recommendations

### Full-Disk Encryption

Enable full-disk encryption (e.g. LUKS on Linux) on the machine hosting both the server and the CAS directory. Screenshots, audio chunks, and browser history on disk are not encrypted at the application layer; OS-level encryption provides the protection-at-rest layer.

### CAS Directory Permissions

Restrict access to the CAS directory to the user running the Lifelog server:

```bash
chmod 700 ~/lifelog/cas
```

This prevents other local users from accessing captured binary data directly.

### PostgreSQL Access

The PostgreSQL database should be accessible only by the Lifelog server process. Use a dedicated database user with a strong password and bind PostgreSQL to localhost only (`listen_addresses = 'localhost'` in `postgresql.conf`).

### TLS Between Collector and Server

Enable TLS even on a local area network. Plaintext gRPC over a LAN exposes authentication tokens and frame data to passive interception. Use a private CA or a tool like `mkcert` to issue certificates for internal hostnames.

### Token Management

Store `LIFELOG_AUTH_TOKEN` and `LIFELOG_ENROLLMENT_TOKEN` in a file with restricted permissions (mode `600`) and source it at startup, rather than placing the values in `lifelog-config.toml`. The config file may be committed to version control or shared; the token file should not be.

### API Keys for External Transform Services

If you configure a transform that uses an external API key (e.g. OpenRouter), use the `@` file reference syntax to keep the key out of the config file:

```toml
[transforms.params]
api_key = "@/run/secrets/openrouter-key"
```

The referenced file should have mode `600` and be owned by the server process user.
