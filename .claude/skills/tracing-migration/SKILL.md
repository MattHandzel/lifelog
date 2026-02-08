---
name: tracing-migration
description: Replace println/eprintln with structured tracing in a Rust file
disable-model-invocation: true
---

Migrate the following file(s) to structured tracing: $ARGUMENTS.

Rules:

- `println!(...)` -> `tracing::info!(...)` or `tracing::debug!(...)`
- `eprintln!(...)` -> `tracing::warn!(...)` or `tracing::error!(...)` depending on severity
- Use structured fields where possible: `tracing::info!(key = %value, "message")`
- NEVER replace `println!("cargo:...")` in build.rs files (these are Cargo directives)
- Skip `#[cfg(test)]` blocks
- Ensure `tracing` is in the crate's Cargo.toml dependencies

After changes, run `just check` to verify compilation.
