# Standard workflows for Lifelog

# Run all checks (excluding Tauri UI)
check:
    nix develop --command cargo check --all-targets

# Run all tests (excluding Tauri UI)
test:
    nix develop --command cargo nextest run --all-targets

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
    nix develop --command cargo fmt -- --check
    nix develop --command cargo check --all-targets
    nix develop --command cargo clippy --all-targets -- -D warnings
    nix develop --command cargo test --all-targets

# Start the server
run-server:
    nix develop --command cargo run -p lifelog-server --bin lifelog-server-backend

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

# Clean temporary test data
clean-tests:
    rm -rf /tmp/lifelog-test-*

# --- Worktree lifecycle ---

# Create a worktree for an agent task
worktree-create name branch_base="refactor/proto-first-completion":
    git branch agent/{{name}} {{branch_base}}
    git worktree add ../lifelog-worktrees/{{name}} agent/{{name}}
    @echo "Worktree created at ../lifelog-worktrees/{{name}} on branch agent/{{name}}"

# Remove a worktree and its branch
worktree-remove name:
    git worktree remove ../lifelog-worktrees/{{name}}
    git branch -d agent/{{name}}

# List active worktrees
worktree-list:
    git worktree list

# Merge an agent branch with verification
merge-agent name:
    git merge --no-ff agent/{{name}}
    just check
    just test

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
