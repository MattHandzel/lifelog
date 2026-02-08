---
name: debug-build
description: Diagnose and fix build failures systematically
disable-model-invocation: true
---

A build failure needs debugging. Context: $ARGUMENTS

## Steps

1. Run `tools/ai/check_digest.sh nix develop --command cargo check --all-targets` to get current errors
2. Categorize errors:
   - **Type errors**: mismatched types, missing trait impls
   - **Import errors**: unresolved imports, missing modules
   - **Lifetime errors**: borrow checker issues
   - **Proto errors**: generated code mismatch (may need `cargo build -p lifelog-proto` first)
3. Fix errors starting from the **dependency root** (errors in common/ before server/, proto before everything)
4. After each fix, re-run check_digest to verify progress
5. Once clean, run `just validate` to ensure clippy + tests also pass

IMPORTANT: If proto-generated code is stale, rebuild proto first: `nix develop --command cargo build -p lifelog-proto`
