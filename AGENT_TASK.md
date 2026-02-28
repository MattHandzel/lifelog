# Agent Task: Implement security-hardening

## Objective
Implement the feature "security-hardening" according to the plan.

## Context
- **Plan Document:** PLAN.md
- **Reference:** @SPEC.md and @SPEC_TODOLIST.md
- **Goal:** Autonomous implementation, testing, and verification.

## Initialization Sequence (MANDATORY)
1. Read `GEMINI.md` and `docs/REPO_MAP.md` to refresh architecture and file context.
2. Read the Plan Document (PLAN.md).
3. Check `git status`, `git worktree list`, and `git log -n 5` to understand the current work state.
4. Run `just check` to verify the baseline is green.

## Instructions
- Work strictly within this worktree.
- Follow the "Research -> Strategy -> Execution" lifecycle.
- Prioritize empirical reproduction of any related issues before fixing.
- Ensure all changes are verified with `just validate` or targeted tests.
- If you encounter significant ambiguity, use `ask_user`. Otherwise, proceed autonomously.

## Handoff Report
- **Optional TLS:** Updated `server/src/main.rs` and `collector/src/collector.rs` to support optional TLS. If certificates are missing or `http://` is used, the system falls back to plaintext but logs a prominent warning.
- **Optional Authentication:** Added a `check_auth` interceptor to `server/src/main.rs`. If `LIFELOG_AUTH_TOKEN` is not set, it allowing all connections with a startup warning. If tokens are set, it strictly enforces them.
- **Token Generation:** Added a `generate-token` subcommand to `lifelog-server`. Users can run `cargo run -p lifelog-server -- generate-token` to get a secure random string.
- **Improved Logging:** Added explicit `tracing::warn!` messages on the server when authentication fails or is missing. Added warnings to the collector when connecting without a token.
- **Test Hardening:** Updated all integration tests to pass in the new optional-security mode while still verifying that authentication logic works correctly when enabled.
- **Verification:** Ran `nix develop --command cargo test --all-targets`. All tests passed.

## Completion
- Feature implemented as optional and verified.
- Token generation utility added.
- Tests updated and passing.
- Handoff report completed above.
