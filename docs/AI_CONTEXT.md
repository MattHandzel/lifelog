# AI Context Guide (On-Demand)

This file is intentionally not "always loaded". Open it only when you need workflow guidance.

Goal: reduce LLM token usage 2 to 3x by keeping the default context tiny, and using short digests for any noisy output.

## Default Workflow (Token-Efficient)

1. Scout (read-only): produce a shortlist of files (5 to 12) and the next 1 to 3 files to open.
2. Open minimal files: open only the shortlist items needed to implement the change.
3. Implement: keep diffs tight; avoid wholesale rewrites unless required.
4. Verify: run commands through a digest wrapper.
5. Report: summarize what changed and how to validate. Reference file paths.

## File-Open Budget

- Prefer opening 1 to 3 files initially.
- Avoid opening large "overview" docs by default.
- If a file is large, search inside it (`rg`) before opening.

Heuristics:

- Need how to run/build: open `justfile`, `README.md`, `AGENTS.md`, or `docs/REPO_MAP.md`.
- Need architecture background: search `docs/` for the specific concept, then open one doc.
- Need code entry: start with an entry point listed in `docs/REPO_MAP.md`.

Avoid:

- `target/` (build artifacts).
- giant root docs unless explicitly required: `SPEC*.md`, `VALIDATION_SUITE.md`, large `*_ANALYSIS.md`.

## Output Discipline (Prevent Token Bombs)

Never paste raw command output larger than roughly 150 lines. Use digests.

Tools:

Digest arbitrary output: `tools/ai/summarize_output.sh < output.txt`

Run a command and print a digest by default: `tools/ai/run_and_digest.sh -- <cmd> [args...]`

Show raw output only when necessary: `tools/ai/run_and_digest.sh --full -- <cmd> [args...]`

Digest diffs: `tools/ai/git_diff_digest.sh` (or `--cached`)

Recommended pattern:

- When running tests/builds: always use `tools/ai/run_and_digest.sh`.
- When sharing results: copy the digest, not the full logs.

## Searching Instead of Opening

Use `rg` to locate the smallest relevant surface:

- `rg \"ThingName\" server/`
- `rg \"grpc\" docs/`
- `rg --files | rg \"collector\"`

If a file is large, pull a small excerpt:

- `sed -n 'START,ENDp' path/to/file`

## Subagent Scout Prompt

Use `prompts/agent_repo_scout.md` when you need exploration without contaminating main context.

Expected output from the scout:

- 5 to 12 file shortlist with 1-line rationale each.
- 1 to 3 "open next" files.
- Any commands to run (through `tools/ai/run_and_digest.sh`).

## Troubleshooting Flow

Build/test fails:

- Re-run with digest.
- Search the digest for the first error, not the last.
- Use `rg` to find the symbol or module where the error originates.

Large diff or widespread changes:

- Use `tools/ai/git_diff_digest.sh` to keep the conversation scoped.
- Consider splitting into multiple small PRs/steps if possible.

## Reset/Compact Guidance

When the thread becomes long or polluted with logs:

Start a new thread with:

1. Current goal (1 sentence)
2. File shortlist (5 to 12 paths)
3. Current failing digest (only)
4. What changed (diff stat)
