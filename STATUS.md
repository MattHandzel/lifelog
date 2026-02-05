# Status

## Current Objective

Implement a runnable, incremental validation suite for `SPEC.md` based on `VALIDATION_SUITE.md`.

## Last Verified

- `cargo test -p lifelog-core`
- `cargo test -p utils`

## How To Verify (Target)

- `nix develop -c cargo check`
- `nix develop -c cargo test`

## What Changed Last

- Started validation-suite work with build fixes to make default workspace checks lighter and proto builds self-contained.

## What's Next

- Add unit-testable “pure semantics” modules (time skew, replay steps, correlation) and associated tests.
- Add a `tests/validation_suite` integration test skeleton (ignored-by-default).

## Blockers

- None, assuming `nix develop` is available for native deps on Linux.
