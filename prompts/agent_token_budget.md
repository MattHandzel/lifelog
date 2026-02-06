# Agent Token Budget Snippet

Use `docs/REPO_MAP.md` first. Only open additional files with explicit justification.

## Budgets

- Initial file opens: 1 to 3 files.
- Scout shortlist: 5 to 12 file paths max.
- Logs: never paste raw output over ~150 lines.
- Diffs: prefer `--stat` and small patch excerpts.

## Output Rules

- For any command output, run via `tools/ai/run_and_digest.sh` and paste only the digest.
- Use `tools/ai/git_diff_digest.sh` to summarize changes.
- If you must show full output, use `--full` and include only the smallest relevant excerpt.

## Working Style

- Search before opening: `rg` over broad file reads.
- Keep responses short and reference file paths.
- If context is polluted, stop and restart with a compact state: goal, shortlist, latest digest, diff stat.
