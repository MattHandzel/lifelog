# Plan: Security Hardening (TLS, Enrollment, Auth)

## Objective
Enforce transport security and implement a secure collector-to-server enrollment handshake.

## Phase 1: Research & Strategy
1. **Inventory:** Review `server/src/grpc_service.rs` and the opt-in TLS logic in `Phase 5` history.
2. **Interceptors:** Identify where to add gRPC interceptors for Authentication (Token-based).
3. **Enrollment Handshake:** Design the "Token-based Pairing" flow where a collector provides a pre-shared token to receive its identity.

## Phase 2: Execution
1. **Enforce TLS:** Update server and collector to fail startup if TLS is not configured (Section 12.1).
2. **Auth Interceptor:** Implement a server-side interceptor that validates the `Authorization: Bearer <token>` header on all data and control RPCs.
3. **Enrollment RPC:** Add a `PairCollector` RPC (or similar) to `proto/lifelog.proto` that exchanges a secret token for a persistent `collector_id`.
4. **Remove Hardcoded Creds:** Ensure SurrealDB credentials and gRPC tokens are only read from environment variables (Section 7.6).

## Phase 3: Verification
1. Run server and collector with and without valid tokens.
2. Verify that unauthorized collectors are rejected with `PermissionDenied`.
3. Verify that all traffic is encrypted (e.g., via `grpcurl` or by checking logs for TLS handshakes).

## Model Recommendation
**Gemini 3.1 Pro Preview** (Required for security-critical logic and gRPC interceptor patterns).
