# Plan: Release Official v1 (Production Ready)

## Objective
Finalize the codebase for the first official release by removing non-deterministic sleeps, hardening tests, and verifying deployment modules.

## Phase 1: Performance & Determinism (Remove Sleeps)
1.  **Event-Driven ACK Gating:** 
    - Replace the polling loop in `Server::handle_upload_chunks` with a `tokio::sync::watch` or `broadcast` channel that signals when indexing is complete.
    - This will eliminate the "Wait 100ms" cycles in ingestion.
2.  **Test Harness Hardening:**
    - Replace `tokio::time::sleep` in `server/tests/` with event-based synchronization.
    - Example: `TestContext` should expose a way to "Wait for Ingest" or "Wait for Transform".

## Phase 2: Deployment Verification
1.  **Nix Module Audit:** 
    - Verify `deploy/nix/nixos-module.nix` and `deploy/nix/home-manager-module.nix` (once available).
    - Ensure systemd units correctly use the `lifelog` binary subcommands.
2.  **Onboarding Polish:**
    - Verify `lifelog init` produces a config that works out-of-the-box with the Nix modules.

## Phase 3: Final Quality Gate
1.  **Zero-Failure Validation:**
    - Fix the flaky `sync_scenarios` and `all_modalities_dataflow` tests.
    - Ensure `just validate` passes 5 times in a row without flakes.

## Phase 4: Release Artifacts
1.  **Documentation:** Finalize `README.md` and `USAGE.md`.
2.  **Release Tag:** Prepare `v1.0.0` metadata.
