# Standard workflows for Lifelog

# Run all checks (excluding Tauri UI)
check:
    nix develop --command cargo check --all-targets

# Run all tests (excluding Tauri UI)
test:
    nix develop --command cargo test --all-targets

# Run the integration validation suite
test-e2e:
    nix develop --command cargo test -p lifelog-server --test validation_suite -- --nocapture

# Start the server
run-server:
    nix develop --command cargo run -p lifelog-server --bin lifelog-server-backend

# Start the collector
run-collector:
    nix develop --command cargo run -p lifelog-collector --bin lifelog-collector

# Clean temporary test data
clean-tests:
    rm -rf /tmp/lifelog-test-*
