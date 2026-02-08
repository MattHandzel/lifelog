---
name: review-changes
description: Review uncommitted changes for quality issues, security problems, and convention violations
context: fork
agent: general-purpose
allowed-tools: Read, Grep, Glob, Bash
---

## Changes to review
!`git diff --stat`

## Detailed diff
!`git diff`

## Your task

Review these uncommitted changes. Check for:

1. **Security**: SQL injection (raw string interpolation in SurrealDB queries), unsafe code, secrets in code
2. **Panic paths**: unwrap(), expect(), panic!() in library code (only OK in tests and binary crates)
3. **Convention violations**: raw cargo (should use just/nix), println/eprintln (should use tracing)
4. **Proto compliance**: manual type definitions that should come from proto
5. **Error handling**: anyhow in library crates (should use thiserror), swallowed errors
6. **Dead code**: unused imports, unreachable code, commented-out code

For each issue found, report: file:line, severity (critical/warning/info), description, suggested fix.
Sort by severity (critical first).
