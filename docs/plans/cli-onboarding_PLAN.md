# Plan: CLI Onboarding (Hero Command)

## Objective
Create a frictionless interactive setup experience for new users via a `lifelog` CLI.

## Phase 1: Infrastructure
1.  **Crate Setup:** Ensure the `lifelog-server` or a new `lifelog-cli` crate can act as the main entry point.
2.  **Native TLS:** Implement `CertificateGenerator` using the `rcgen` crate to remove the `openssl` dependency for users.

## Phase 2: `lifelog init` (Server Setup)
1.  **Interactive TUI:** Use `inquire` to ask for storage paths and device aliases.
2.  **Secret Gen:** Generate `LIFELOG_AUTH_TOKEN` and `LIFELOG_ENROLLMENT_TOKEN`.
3.  **Config Write:** Create a valid `lifelog-config.toml` and `.env` file in `~/.config/lifelog/`.

## Phase 3: `lifelog join <url>` (Collector Setup)
1.  **Handshake:** Fetch server public cert, verify fingerprint with user.
2.  **Auto-Pairing:** Execute `PairCollector` RPC to get a stable `collector_id`.
3.  **Service Install:** Offer to install `systemd --user` units for persistence.

## Phase 4: Verification
1.  Verify a fresh user can go from zero to "Capturing" in < 60 seconds.
