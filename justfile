# Standard workflows for Lifelog

# --- Context Engineering ---
IS_LLM_AGENT := env_var_or_default("IS_LLM_AGENT", "0")

# --- Per-worktree isolation ---
# Worktrees get a .env with CARGO_TARGET_DIR, LIFELOG_TEST_DB, LIFELOG_PORT.
# `just` silently skips when .env is absent (main repo).
set dotenv-load := true

# Run all checks (excluding Tauri UI)
check:
    @if [ "{{IS_LLM_AGENT}}" = "1" ]; then \
        ./tools/ai/check_digest.sh; \
    else \
        nix develop --command cargo check --all-targets; \
    fi

# Alias for agent-specific digest checks
check-digest: check

# Create a new SQLx migration (server crate)
sqlx-migrate-add name:
    nix develop --command sh -c 'cd server && sqlx migrate add {{name}}'

# Apply SQLx migrations (server crate)
sqlx-migrate-run database_url:
    DATABASE_URL={{database_url}} nix develop --command sh -c 'cd server && sqlx migrate run'

# Run all tests (excluding Tauri UI)
test:
    @if [ "{{IS_LLM_AGENT}}" = "1" ]; then \
        ./tools/ai/run_and_digest.sh "nix develop --command cargo nextest run --all-targets"; \
    else \
        nix develop --command cargo nextest run --all-targets; \
    fi

# Alias for agent-specific digest tests
test-digest: test

# Run the integration validation suite
test-e2e:
    nix develop --command cargo test -p lifelog-server --test validation_suite -- --include-ignored --nocapture
    nix develop --command cargo test -p lifelog-server --test multi_device -- --include-ignored --nocapture
    nix develop --command cargo test -p lifelog-server --test sync_scenarios -- --include-ignored --nocapture

# Run only the sync scenario tests
test-sync:
    nix develop --command cargo test -p lifelog-server --test sync_scenarios -- --include-ignored --nocapture

# E2E tests with file lock (prevents concurrent SurrealDB conflicts)
test-e2e-exclusive:
    flock /tmp/lifelog-e2e.lock nix develop --command cargo test -p lifelog-server --test validation_suite -- --include-ignored --nocapture
    flock /tmp/lifelog-e2e.lock nix develop --command cargo test -p lifelog-server --test multi_device -- --include-ignored --nocapture

# Full validation gate — run before reporting work done
validate:
    @if [ "{{IS_LLM_AGENT}}" = "1" ]; then \
        ./tools/ai/run_and_digest.sh "just validate-raw"; \
    else \
        just validate-raw; \
    fi

# Internal recipe for raw validation
validate-raw:
    nix develop --command cargo fmt
    nix develop --command cargo fmt -- --check
    nix develop --command cargo check --all-targets
    nix develop --command cargo clippy --all-targets -- -D warnings
    nix develop --command cargo test --all-targets

# Start the server
run-server:
    nix develop --command cargo run -p lifelog-server --bin lifelog-server

# Start the server with TLS (requires cert/key files)
run-server-tls cert_path key_path:
    LIFELOG_TLS_CERT_PATH={{cert_path}} LIFELOG_TLS_KEY_PATH={{key_path}} nix develop --command cargo run -p lifelog-server --bin lifelog-server

# Start the collector
run-collector:
    nix develop --command cargo run -p lifelog-collector --bin lifelog-collector

# Run tests with nextest (parallel, per-process isolation)
test-fast:
    nix develop --command cargo nextest run --all-targets

# Continuous check-on-save (requires bacon in nix shell)
watch:
    nix develop --command bacon

# Run Tier 2 container chaos tests (requires Docker)
test-chaos:
    docker compose -f tests/docker/docker-compose.chaos.yml up --build --abort-on-container-exit
    docker compose -f tests/docker/docker-compose.chaos.yml down -v

# Run frontend unit tests (Vitest)
test-ui:
    cd interface && npx vitest run

# Full validation including frontend
validate-all:
    just validate
    just test-ui

# Clean temporary test data
clean-tests:
    rm -rf /tmp/lifelog-test-*

# --- Worktree lifecycle ---

# Display the developer dashboard and agent management guide
work:
    @echo "========================================================================"
    @echo "🚀 LIFELOG DEVELOPER DASHBOARD"
    @echo "========================================================================"
    @echo "📍 CURRENT WORKSPACE: $(basename $(pwd))"
    @git status -s
    @echo "\n🕒 RECENT HISTORY:"
    @git log -n 3 --oneline
    @echo "\n🤖 ACTIVE AGENTS & WORKTREES:"
    @git worktree list | sed 's/^/  /'
    @echo "------------------------------------------------------------------------"
    @echo "🛠️  COMMAND REFERENCE:"
    @echo "------------------------------------------------------------------------"
    @echo "1. START A FEATURE:   just use-ai-to-implement-feature {name} [ai=claude|gemini]"
    @echo "2. CHECK ON AGENTS:   just status-all"
    @echo "3. SHIP & PUSH:       just ship-feature {name}"
    @echo "4. INIT AI SESSION:   just init-session (run this first in new sessions)"
    @echo "5. MANUAL CLEANUP:    just worktree-remove {name}"
    @echo "------------------------------------------------------------------------"

    @echo "💡 Tip: Use 'just status-all' to see exactly what each agent is doing."
    @echo "========================================================================"

# Create a worktree for an agent task
worktree-create name branch_base="main":
    @just worktree-remove {{name}}
    git branch -f agent/{{name}} {{branch_base}}
    git worktree add worktrees/feature/{{name}} agent/{{name}}
    @echo "📦 Isolating frontend dependencies (Hardlink copy)..."
    @mkdir -p worktrees/feature/{{name}}/interface/node_modules
    @if [ -d "interface/node_modules" ]; then \
        cp -al interface/node_modules/. worktrees/feature/{{name}}/interface/node_modules/; \
    fi
    @echo "🔧 Writing per-worktree .env for build isolation..."
    @printf 'CARGO_TARGET_DIR=target-wt\nLIFELOG_TEST_DB=lifelog_test_%s\nLIFELOG_PORT=0\n' "{{name}}" \
        > worktrees/feature/{{name}}/.env
    @echo "Worktree created at worktrees/feature/{{name}} on branch agent/{{name}}"
    @echo "  CARGO_TARGET_DIR = target-wt (isolated)"
    @echo "  LIFELOG_TEST_DB  = lifelog_test_{{name}}"
    @echo "  LIFELOG_PORT     = 0 (auto-assign)"

# Remove a worktree and its branch (Manual Cleanup)
worktree-remove name:
    @echo "🧹 Removing worktree and branch for '{{name}}'..."
    @git worktree remove --force worktrees/feature/{{name}} 2>/dev/null || true
    @rm -rf worktrees/feature/{{name}}
    @git branch -D agent/{{name}} 2>/dev/null || true

# List active worktrees and their current branch status
status-all:
    @echo "=== Active Worktree Status ==="
    @git worktree list | while read wt branch commit; do \
        echo "\n📍 $(basename $wt) ($branch)"; \
        (cd "$wt" && git status -s && git log -n 1 --oneline); \
    done


# Validate, merge, and ship a feature to main
ship-feature name:
    @echo "=== Shipping '{{name}}' ==="
    @echo "🛠️  Running final validation suite in worktree..."
    @cd worktrees/feature/{{name}} && ../../../tools/ai/run_and_digest.sh "just validate"
    @echo "🔄 Updating feature branch with latest main..."
    @cd worktrees/feature/{{name}} && git fetch origin main && git merge origin/main --no-edit
    @echo "🔀 Merging 'agent/{{name}}' into $(git branch --show-current)..."
    @git merge --no-ff agent/{{name}} -m "feat: ship {{name}}"
    @echo "🚀 Pushing changes to remote..."
    @git push origin $(git branch --show-current)
    @if [ -n "$TMUX" ]; then \
        tmux kill-window -t "{{name}}" 2>/dev/null || true; \
    fi
    @echo "✨ Feature '{{name}}' is LIVE."
    @echo "📌 AUDIT NOTE: Worktree preserved at worktrees/feature/{{name}}."
    @echo "💡 To clean up later, run: just worktree-remove {{name}}"

# Merge an agent branch with verification (standalone)
merge-agent name:
    git merge --no-ff agent/{{name}}
    just check
    just test

# Prepare the workspace for a new AI session (run inside a worktree or root)
init-session:
    @echo "=== Session Initialization Protocol ==="
    @git status
    @just map-repo
    @git log -n 3 --oneline
    @if [ "{{IS_LLM_AGENT}}" = "1" ]; then \
        echo "⚡ Agent Mode: Assuming repository is healthy, skipping initial check."; \
    else \
        if [ -f "/tmp/lifelog-last-validate" ] && [ "$(( $(date +%s) - $(stat -c %Y /tmp/lifelog-last-validate) ))" -lt 600 ]; then \
            echo "✅ Main recently validated (less than 10m ago). Skipping."; \
        else \
            ./tools/ai/check_digest.sh && touch /tmp/lifelog-last-validate; \
        fi; \
    fi
    @echo "======================================="

# Generate a high-signal repository discovery map for agents
map-repo:
    @chmod +x tools/ai/generate_discovery_map.sh
    @./tools/ai/generate_discovery_map.sh

# Run RepoAtlas static analysis + LLM-assisted visualization composition
repoatlas:
    python3 tools/repoatlas/repoatlas.py --repo . --expected repoatlas/expected_arch.json --out docs/repoatlas --max-hops 4
    python3 tools/repoatlas/viz_compose.py --repo . --graph docs/repoatlas/graph.json --journeys docs/repoatlas/journeys.json --decisions docs/repoatlas/decisions.json --drift docs/repoatlas/drift.json --out docs/repoatlas/view_config.json --agent ${REPOATLAS_AGENT:-codex} --model "${REPOATLAS_MODEL:-}" --require-llm

# Static-only mode (no LLM overlay)
repoatlas-static:
    python3 tools/repoatlas/repoatlas.py --repo . --expected repoatlas/expected_arch.json --out docs/repoatlas --max-hops 4
    python3 tools/repoatlas/viz_compose.py --repo . --graph docs/repoatlas/graph.json --journeys docs/repoatlas/journeys.json --decisions docs/repoatlas/decisions.json --drift docs/repoatlas/drift.json --out docs/repoatlas/view_config.json

# Legacy: LLM multi-agent graph construction
repoatlas-agent-graph:
    python3 tools/repoatlas/repoatlas_agents.py --repo . --expected repoatlas/expected_arch.json --out docs/repoatlas --agent ${REPOATLAS_AGENT:-codex} --model "${REPOATLAS_MODEL:-}" --max-hops 4

# Serve the interactive graph explorer
repoatlas-view:
    python3 tools/repoatlas/viewer/serve_viewer.py --root . --host ${HOST:-127.0.0.1} --port ${PORT:-8123}

# Get a high-signal digest of code changes
diff-digest ref="main":
    @./tools/ai/git_diff_digest.sh {{ref}}

# Get a structural summary of a file
summary file:
    @./tools/ai/file_summary.sh {{file}}

# Automate new feature development with AI (gemini, claude, or codex)
# Parameters:
#   name: The name of the feature/task
#   ai: The AI provider to use (default: "claude")
#   model: Optional specific model name
#   plan_file: Path to the plan file (defaults to docs/plans/{name}_PLAN.md)
use-ai-to-implement-feature name ai="claude" model="" plan_file=("docs/plans/" + name + "_PLAN.md"):
    @just worktree-create {{name}}
    @just map-repo
    @cp {{plan_file}} worktrees/feature/{{name}}/PLAN.md || true
    @printf "# Agent Task: Implement {{name}}\n\n## Objective\nImplement the feature \"{{name}}\" according to the plan.\n" > worktrees/feature/{{name}}/AGENT_TASK.md
    @echo "Feature '{{name}}' workspace prepared at worktrees/feature/{{name}}"
    @if [ -n "$TMUX" ]; then \
        tmux kill-window -t "{{name}}" 2>/dev/null || true; \
        echo "--- SYSTEM PROMPT ---" > worktrees/feature/{{name}}/INITIAL_PROMPT.md; \
        cat .gemini/AGENT_SYSTEM_PROMPT.md >> worktrees/feature/{{name}}/INITIAL_PROMPT.md; \
        echo "\n--- GROUND TRUTH (SPEC.md) ---" >> worktrees/feature/{{name}}/INITIAL_PROMPT.md; \
        cat SPEC.md >> worktrees/feature/{{name}}/INITIAL_PROMPT.md; \
        echo "\n--- REPO MAP ---" >> worktrees/feature/{{name}}/INITIAL_PROMPT.md; \
        cat docs/REPO_DISCOVERY_MAP.json >> worktrees/feature/{{name}}/INITIAL_PROMPT.md; \
        echo "\n--- STATUS & GAPS ---" >> worktrees/feature/{{name}}/INITIAL_PROMPT.md; \
        cat STATUS.md >> worktrees/feature/{{name}}/INITIAL_PROMPT.md; \
        echo "\n--- TASK & PLAN ---" >> worktrees/feature/{{name}}/INITIAL_PROMPT.md; \
        cat worktrees/feature/{{name}}/PLAN.md >> worktrees/feature/{{name}}/INITIAL_PROMPT.md; \
        echo "\n--- STATE HISTORY (LATEST) ---" >> worktrees/feature/{{name}}/INITIAL_PROMPT.md; \
        tail -n 50 STATE_HISTORY.md >> worktrees/feature/{{name}}/INITIAL_PROMPT.md 2>/dev/null || echo 'No previous history.' >> worktrees/feature/{{name}}/INITIAL_PROMPT.md; \
        echo "\n### INSTRUCTIONS\nYou have been provided with the full SPEC, STATUS, and PLAN. DO NOT waste turns reading these files. Begin implementation immediately. If this is a UI task, the discovery map contains component paths." >> worktrees/feature/{{name}}/INITIAL_PROMPT.md; \
        if [ "{{ai}}" != "gemini" ] && [ "{{ai}}" != "claude" ] && [ "{{ai}}" != "codex" ]; then \
            echo "Error: Unknown AI '{{ai}}'. Use 'gemini', 'claude' or 'codex'."; exit 1; \
        fi; \
        PROJECT_ROOT="$(pwd)"; \
        tmux new-window -n "{{name}}" "bash -c 'export IS_LLM_AGENT=1 && cd worktrees/feature/{{name}} && $PROJECT_ROOT/tools/ai/start_agent.sh {{ai}} \"{{model}}\" INITIAL_PROMPT.md; exec bash'"; \
        echo "Opened new tmux window '{{name}}' with {{ai}} running and hyper-injected context."; \
    else \
        echo "No tmux detected. Start the agent manually:"; \
        echo "  export IS_LLM_AGENT=1 && cd worktrees/feature/{{name}} && [claude|gemini] ..."; \
    fi


# --- Deployment ---

# Build release binaries and copy to /usr/local/bin
install:
    nix develop --command cargo build --release -p lifelog-server -p lifelog-collector
    sudo cp target/release/lifelog-server /usr/local/bin/
    sudo cp target/release/lifelog-collector /usr/local/bin/
    @echo "Binaries installed to /usr/local/bin/"

# Install systemd service files and reload daemon
install-services:
    sudo cp deploy/lifelog-server.service /etc/systemd/system/
    sudo cp deploy/lifelog-collector.service /etc/systemd/system/
    sudo cp deploy/surrealdb.service /etc/systemd/system/
    sudo systemctl daemon-reload
    @echo "Service files installed. Use 'sudo systemctl start lifelog-server' to start."

# Remove systemd service files
uninstall-services:
    sudo systemctl stop lifelog-server lifelog-collector surrealdb 2>/dev/null || true
    sudo rm -f /etc/systemd/system/lifelog-server.service
    sudo rm -f /etc/systemd/system/lifelog-collector.service
    sudo rm -f /etc/systemd/system/surrealdb.service
    sudo systemctl daemon-reload
    @echo "Service files removed."

# --- Hooks ---

# Install pre-commit hook (shared across worktrees)
install-hooks:
    #!/usr/bin/env bash
    set -euo pipefail
    hook=".git/hooks/pre-commit"
    mkdir -p .git/hooks
    printf '%s\n' '#!/usr/bin/env bash' \
      'echo "[pre-commit] Running cargo fmt check..."' \
      'nix develop --command cargo fmt -- --check || {' \
      '    echo "ERROR: cargo fmt check failed. Run nix develop --command cargo fmt to fix."' \
      '    exit 1' \
      '}' \
      'echo "[pre-commit] Running cargo check..."' \
      'nix develop --command cargo check --all-targets || {' \
      '    echo "ERROR: cargo check failed."' \
      '    exit 1' \
      '}' \
      'echo "[pre-commit] All checks passed."' \
      > "$hook"
    chmod +x "$hook"
    echo "Pre-commit hook installed at $hook"
