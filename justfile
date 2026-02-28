# Standard workflows for Lifelog

# --- Context Engineering ---
IS_LLM_AGENT := env_var_or_default("IS_LLM_AGENT", "0")

# Run all checks (excluding Tauri UI)
check:
    @if [ "{{IS_LLM_AGENT}}" = "1" ]; then \
        ./tools/ai/check_digest.sh; \
    else \
        nix develop --command cargo check --all-targets; \
    fi

# Run all tests (excluding Tauri UI)
test:
    @if [ "{{IS_LLM_AGENT}}" = "1" ]; then \
        ./tools/ai/run_and_digest.sh "nix develop --command cargo nextest run --all-targets"; \
    else \
        nix develop --command cargo nextest run --all-targets; \
    fi

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
    nix develop --command cargo fmt -- --check
    nix develop --command cargo check --all-targets
    nix develop --command cargo clippy --all-targets -- -D warnings
    nix develop --command cargo test --all-targets

# Start the server
run-server:
    nix develop --command cargo run -p lifelog-server --bin lifelog-server-backend

# Start the server with TLS (requires cert/key files)
run-server-tls cert_path key_path:
    LIFELOG_TLS_CERT_PATH={{cert_path}} LIFELOG_TLS_KEY_PATH={{key_path}} nix develop --command cargo run -p lifelog-server --bin lifelog-server-backend

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
    @echo "1. START A FEATURE:   just use-gemini-to-develop-a-new-feature {name}"
    @echo "2. CHECK ON AGENTS:   just status-all"
    @echo "3. SHIP & PUSH:       just ship-feature {name}"
    @echo "4. INIT AI SESSION:   just init-session (run this first in new sessions)"
    @echo "5. MANUAL CLEANUP:    just worktree-remove {name}"
    @echo "------------------------------------------------------------------------"

    @echo "💡 Tip: Use 'just status-all' to see exactly what each agent is doing."
    @echo "========================================================================"

# Create a worktree for an agent task
worktree-create name branch_base="main":
    git branch agent/{{name}} {{branch_base}}
    git worktree add worktrees/feature/{{name}} agent/{{name}}
    @echo "Worktree created at worktrees/feature/{{name}} on branch agent/{{name}}"

# Remove a worktree and its branch (Manual Cleanup)
worktree-remove name:
    @echo "🧹 Removing worktree and branch for '{{name}}'..."
    @git worktree remove --force worktrees/feature/{{name}} 2>/dev/null || true
    @git branch -D agent/{{name}} 2>/dev/null || true

# List active worktrees and their current branch status
status-all:
    @echo "=== Active Worktree Status ==="
    @git worktree list --porcelain | grep "^worktree" | cut -d' ' -f2 | while read wt; do \
        echo "\n📍 $$wt"; \
        (cd "$$wt" && git status -s && git log -n 1 --oneline); \
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
    @git worktree list | grep $(basename $(pwd)) || true
    @git log -n 3 --oneline
    @if [ "{{IS_LLM_AGENT}}" = "1" ]; then \
        echo "⚡ Agent Mode: Assuming repository is healthy, skipping initial check."; \
    else \
        ./tools/ai/check_digest.sh; \
    fi
    @echo "======================================="

# Get a high-signal digest of code changes
diff-digest ref="main":
    @./tools/ai/git_diff_digest.sh {{ref}}

# Get a structural summary of a file
summary file:
    @./tools/ai/file_summary.sh {{file}}

# Automate new feature development with Gemini
use-gemini-to-develop-a-new-feature name plan_file=("docs/plans/" + name + "_PLAN.md") model="gemini-3.1-pro-preview":
    @just worktree-create {{name}}
    @cp {{plan_file}} worktrees/feature/{{name}}/PLAN.md || true
    @printf "# Agent Task: Implement {{name}}\n\n## Objective\nImplement the feature \"{{name}}\" according to the plan.\n\n## Context\n- **Plan Document:** PLAN.md\n- **Reference:** @SPEC.md and @SPEC_TODOLIST.md\n- **Goal:** Autonomous implementation, testing, and verification.\n\n## Initialization Sequence (MANDATORY)\n1. Read \`GEMINI.md\` and \`docs/REPO_MAP.md\` to refresh architecture and file context.\n2. Read the Plan Document (PLAN.md).\n3. Check \`git status\`, \`git worktree list\`, and \`git log -n 5\` to understand the current work state.\n4. Run \`just check\` to verify the baseline is green.\n\n## Instructions\n- Work strictly within this worktree.\n- Follow the \"Research -> Strategy -> Execution\" lifecycle.\n- Prioritize empirical reproduction of any related issues before fixing.\n- Ensure all changes are verified with \`just validate\` or targeted tests.\n- If you encounter significant ambiguity, use \`ask_user\`. Otherwise, proceed autonomously.\n\n## Handoff Report (AGENT MUST COMPLETE THIS)\n(When finished, replace this section with a summary of changes, verification results, and any manual steps the user needs to take.)\n\n## Completion\n- Once the feature is implemented and verified, prepare a commit (do not push).\n- Summarize the work done in the Handoff Report above.\n" > worktrees/feature/{{name}}/AGENT_TASK.md
    @echo "Feature '{{name}}' workspace prepared at worktrees/feature/{{name}}"
    @if [ -n "$TMUX" ]; then \
        tmux new-window -n "{{name}}" "bash -c 'export IS_LLM_AGENT=1 && cd worktrees/feature/{{name}} && gemini --model {{model}} -y -i \"\$(cat ../../../.gemini/AGENT_SYSTEM_PROMPT.md)\n\n### TASK\n\nPlease read AGENT_TASK.md and PLAN.md and begin executing the tasks.\"; exec bash'"; \
        echo "Opened new tmux window '{{name}}' with Gemini running (Model: {{model}})."; \
    else \
        echo "No tmux detected. Start the agent manually:"; \
        echo "  export IS_LLM_AGENT=1 && cd worktrees/feature/{{name}} && gemini --model {{model}} -y -i \"\$(cat ../../../.gemini/AGENT_SYSTEM_PROMPT.md)\n\n### TASK\n\nPlease read AGENT_TASK.md and PLAN.md and begin executing the tasks.\""; \
    fi


# --- Deployment ---

# Build release binaries and copy to /usr/local/bin
install:
    nix develop --command cargo build --release -p lifelog-server -p lifelog-collector
    sudo cp target/release/lifelog-server-backend /usr/local/bin/
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
