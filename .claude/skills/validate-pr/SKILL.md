---
name: validate-pr
description: Run full validation and generate a PR-ready summary of all changes
context: fork
agent: general-purpose
disable-model-invocation: true
---

## Current branch state
- Branch: !`git branch --show-current`
- Diff from main: !`git diff main --stat 2>/dev/null || git diff HEAD~5 --stat`
- Unstaged changes: !`git status --short`

## Your task

Validate this branch is ready for a pull request:

1. Run `just validate` and report pass/fail for each step (fmt, check, clippy, test)
2. If any step fails, list the specific errors
3. Summarize the changes in PR format:
   - **Title**: concise 1-line summary
   - **Summary**: 3-5 bullet points of what changed and why
   - **Files changed**: group by logical area (server, collector, common, proto, config)
   - **Risk assessment**: any files in conflict zones? proto changes? breaking changes?
   - **Test coverage**: are the changes tested? what's missing?
4. If validation passes, output the exact `gh pr create` command with title and body
