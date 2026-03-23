# Plan: Demo Environment Bootstrap

## Objective
Initialize the demo environment with valid credentials, persistent storage, and required transformations.

## Phase 1: Credential Generation
1. Run `lifelog-server init` (interactive) to generate:
   - Self-signed TLS certificates in `~/.config/lifelog/tls/`.
   - `LIFELOG_AUTH_TOKEN` and `LIFELOG_ENROLLMENT_TOKEN`.
   - The unified `lifelog-config.toml`.

## Phase 2: Services
1. Start SurrealDB with file-backed storage: `surreal start --bind 127.0.0.1:7183 file:/tmp/lifelog-demo.db`.
2. Ensure `LIFELOG_DB_USER` and `LIFELOG_DB_PASS` are set to `root`.

## Phase 3: Server Configuration
1. Verify `lifelog-config.toml` has the `ocr` transform enabled.
2. Ensure the `cas_path` is correctly set to a persistent directory.
