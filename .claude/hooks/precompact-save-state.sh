#!/usr/bin/env bash
# PreCompact hook: Save session state before context compaction.
# Writes a summary of modified files and git status to a temp file
# that can be referenced after compaction.

cd "$CLAUDE_PROJECT_DIR" 2>/dev/null || exit 0

STATE_FILE="/tmp/lifelog-session-state-$(date +%s).txt"

{
  echo "=== Pre-compaction state snapshot ==="
  echo "Date: $(date -Iseconds)"
  echo ""
  echo "=== Git status ==="
  git status --short 2>/dev/null
  echo ""
  echo "=== Recent commits this session ==="
  git log --oneline -10 2>/dev/null
  echo ""
  echo "=== Modified .rs files ==="
  git diff --name-only 2>/dev/null | grep '\.rs$'
} > "$STATE_FILE"

# The state file path is just for manual reference if needed
exit 0
