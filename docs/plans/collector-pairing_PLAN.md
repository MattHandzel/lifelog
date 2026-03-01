# Plan: Collector Deployment & Pairing

## Objective
Deploy the collector for the demo and pair it with the hardened server.

## Phase 1: Pairing
1. Run `lifelog-collector join` against the demo server URL (e.g., `https://localhost:7182`).
2. Provide the `LIFELOG_ENROLLMENT_TOKEN` when prompted.
3. Verify that the server's CA certificate is saved to `~/.config/lifelog/tls/server-ca.pem`.

## Phase 2: Data Ingest
1. Start the collector with the paired configuration.
2. Verify that screen frames and process logs are being uploaded to the server.
3. Check server logs to confirm indexing (OCR) is occurring.
