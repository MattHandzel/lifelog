# Lifelog Deployment AI Prompt

Copy and paste this prompt into an AI assistant (like Claude, Gemini, or ChatGPT) along with your NixOS/Home Manager configuration files to set up Lifelog.

---

## Role: Expert NixOS & Rust Systems Engineer
## Task: Deploy Lifelog Distributed Capture System

I want to deploy Lifelog (https://github.com/MattHandzel/lifelog) across my infrastructure.

### My Setup:
1. **Server:** Remote NixOS server (Configuration attached).
2. **Collector:** Local NixOS/Home Manager laptop (Configuration attached).

### Objective:
Guide me through setting up the **Lifelog Server** on my server and the **Lifelog Collector** on my laptop using the provided Nix modules in the repository.

### Steps to Automate:
1. **Security:** Use the `lifelog init` command to generate TLS certificates and authentication tokens.
2. **Server Configuration:**
   - Integrate the NixOS module: `inputs.lifelog.nixosModules.default` (bundles server, collector, and postgres submodules).
   - Enable `services.lifelog.server` and `services.lifelog.postgres`.
   - Configure the firewall to allow gRPC traffic (default port 7182).
   - Set `services.lifelog.server.environmentFile` to inject secrets (TLS paths, auth tokens).
3. **Collector Configuration:**
   - Enable `services.lifelog.collector` on the laptop NixOS/Home Manager config.
   - Use `lifelog join <server-url>` to pair the laptop.
   - Configure captured modalities (Screen, Audio, Keystrokes, etc.).
4. **Verification:**
   - Ensure services are running via `systemctl`.
   - Verify that the interactive dashboard shows both nodes as healthy.

### Instructions for AI:
- Be precise. Provide exact Nix code snippets for `configuration.nix` and `home.nix`.
- Explain how to handle sensitive tokens (e.g., using `sops-nix`, `agenix`, or simple environment files).
- Walk me through the initial pairing handshake.
- If any file paths (like `cert.pem`) are needed, tell me exactly where to place them.

---
**[ATTACH YOUR SERVER AND LAPTOP NIX FILES HERE]**
