---
name: commit-changes
description: Validate, stage, and commit current changes with a conventional commit message
disable-model-invocation: true
---

Commit the current changes. Additional context: $ARGUMENTS

## Pre-commit validation
!`git status --short`

## Steps

1. Run `just validate` — if it fails, fix the issues first
2. Review the diff with `tools/ai/git_diff_digest.sh`
3. Stage the appropriate files (never stage .env, credentials, or secrets)
4. Write a commit message following the convention: `type: short description`
   - Types: feat, fix, refactor, docs, tests, build
   - Keep the first line under 72 characters
   - Add a blank line then details if needed
5. Create the commit
6. Show the final `git log --oneline -1` to confirm

IMPORTANT: Never use `git add -A` or `git add .` — stage specific files only.
