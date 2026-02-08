---
name: scope-refactor
description: Scope a refactoring task by counting affected files, lines, and dependencies before starting
context: fork
agent: Explore
allowed-tools: Grep, Glob, Bash, Read
---

Scope the following refactoring task: $ARGUMENTS

Produce a refactoring impact report:

1. **Matches**: Use `tools/ai/scope_changes.sh` to count affected files and lines
2. **File list**: Every file that needs to change, grouped by crate
3. **Dependency order**: Which files should be changed first? (common/ before server/, proto before everything)
4. **Risk assessment**:
   - Does this touch conflict zones? (proto, Cargo.toml, server.rs)
   - Could this break existing tests?
   - Does this require proto regeneration?
5. **Effort estimate**: How many subagents would this need? (use rule: >10 changes across >3 files = subagents)
6. **Recommendation**: Direct edit, subagent delegation, or separate branch?

Be concise. Use counts and file lists, not full file contents.
