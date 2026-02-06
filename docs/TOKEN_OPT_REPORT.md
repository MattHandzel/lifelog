# Token Optimization Report

## What Changed

Added:

- `docs/REPO_MAP.md`: stable repo navigation map.
- `docs/AI_CONTEXT.md`: on-demand agent workflow guide.
- `tools/ai/summarize_output.sh`: turns arbitrary stdout/stderr into a small digest.
- `tools/ai/run_and_digest.sh`: runs a command and prints a digest by default.
- `tools/ai/git_diff_digest.sh`: prints `git diff --stat` plus a summarized diff preview.
- `prompts/agent_token_budget.md`: reusable prompt snippet enforcing strict budgets.
- `prompts/agent_repo_scout.md`: read-only scout prompt for producing a short file shortlist.

Updated:

- `CLAUDE.md`: reduced to a minimal always-loaded rule set.
- `README.md`: added a short workflow section pointing to the new assets.

## Why This Reduces Tokens

- Smaller always-loaded instructions: `CLAUDE.md` now points to on-demand docs instead of embedding architecture and long command references.
- Fewer accidental token bombs: `docs/REPO_MAP.md` and the prompts bias agents toward opening fewer files and avoiding `target/` and large root docs.
- Less log spam: `tools/ai/run_and_digest.sh` prevents raw test/build logs from entering chat context by default.
- Smaller diff sharing: `tools/ai/git_diff_digest.sh` encourages `--stat` plus previews instead of full patches.

## How To Use

Navigation:
Start with `docs/REPO_MAP.md`. Use `docs/AI_CONTEXT.md` only when you need the workflow details.

Running commands:
Use `tools/ai/run_and_digest.sh -- <cmd> [args...]`. Use `--full` only when needed.

Sharing diffs:
Use `tools/ai/git_diff_digest.sh`.

Scouting:
Use `prompts/agent_repo_scout.md` to generate a shortlist before opening files.

## Rollback

Revert by removing the added files and restoring the previous docs:

1. Delete `docs/REPO_MAP.md`.
2. Delete `docs/AI_CONTEXT.md`.
3. Delete `docs/TOKEN_OPT_REPORT.md`.
4. Delete `tools/ai/`.
5. Delete `prompts/`.
6. Revert edits to `CLAUDE.md`.
7. Revert edits to `README.md`.

If using git, a single `git revert` or reset of the commit(s) containing these changes is sufficient.
