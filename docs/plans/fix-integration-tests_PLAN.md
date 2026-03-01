# Plan: Fix & Enhance Integration Tests (Security Migration)

## Objective
Update the existing integration test suite to support mandatory TLS and token-based authentication. Ensure tests are comprehensive, high-quality, and verify the new security invariants.

## Phase 1: Research & Strategy
1.  **Map Harness:** Identify all points in `server/tests/harness/` and `server/tests/` where gRPC channels are created.
2.  **Cert Generation Strategy:** Plan a utility to generate self-signed temporary certificates for test isolation.
3.  **Token Injection:** Define how `LIFELOG_AUTH_TOKEN` and `LIFELOG_ENROLLMENT_TOKEN` will be injected into simulated collectors and the test server.

## Phase 2: Execution
1.  **Harness Update:** 
    - Modify `TestHarness` to generate and manage temporary TLS certs.
    - Update `DeviceClient` to use HTTPS endpoints and provide auth metadata.
2.  **Test Case Migration:**
    - Update `validation_suite.rs`, `multi_device.rs`, and `sync_scenarios.rs` to provide valid security credentials.
3.  **New Security Tests:**
    - **Rejection Test:** Verify that plaintext connections are rejected by the server.
    - **Auth Failure Test:** Verify that incorrect or missing tokens result in `Unauthenticated` errors.
    - **Pairing Test:** Verify the automated `ENROLLMENT_TOKEN -> AUTH_TOKEN` pairing handshake.
4.  **Quality Pass:** 
    - Ensure all tests use `tracing` for observability.
    - Remove any remaining `sleep` based synchronization in favor of polling/assertions.

## Phase 3: Verification
1.  Run `just test-e2e` (Ensure SurrealDB is accessible).
2.  Run `just validate` to ensure everything is green.

## AI Token-Efficient Guidelines
- Use `just diff-digest` to summarize changes.
- Focus on `server/tests/harness/` first as it cascades to all tests.
