<state_snapshot>
    <overall_goal>
        Implement optional security-hardening (TLS, Enrollment, Auth) as per PLAN.md.
    </overall_goal>

    <what_to_do>
        - Implement optional TLS in server and collector (plaintext fallback with warnings).
        - Implement server-side gRPC auth interceptor (allows unauthenticated if no tokens set, but logs warnings).
        - Add `generate-token` subcommand to `lifelog-server` for secure token generation.
        - Add explicit warnings when connecting without tokens if configured.
        - Update collector to provide auth tokens and warn if missing.
        - Update integration tests to pass in default (optional security) mode.
    </what_to_do>

    <why>
        - Security should be built-in but optional for users with external protection (Tailscale).
        - Authentication prevents unauthorized data injection when exposed to the network.
        - Utility commands make it easy for users to adopt secure configurations.
    </why>

    <how>
        - Refactored `server/src/main.rs` to use `clap` for subcommands.
        - Added `GenerateToken` subcommand using `Uuid` for secure random strings.
        - Improved `check_auth` to log `tracing::warn!` on rejected connections.
        - Added startup warnings suggesting `generate-token` if auth is missing.
        - Modified `collector/src/collector.rs` and `upload_manager.rs` to log warnings if `LIFELOG_AUTH_TOKEN` is missing.
        - Updated tests to provide dummy tokens or use plaintext as appropriate.
    </how>

    <validation_steps>
        - Ran `nix develop --command cargo test --all-targets`.
        - Verified `test_collector_with_multiple_modalities_uploads_to_server` passes.
        - Verified `server_binary_e2e_audio_and_keystrokes_roundtrip` passes.
        - Verified `collector_binary_starts_with_all_modules_disabled` passes.
        - Manually verified `cargo run -p lifelog-server -- generate-token` outputs a token.
    </validation_steps>
</state_snapshot>
