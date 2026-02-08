#!/usr/bin/env bash
# SessionStart hook: Inject git context at session start.
# Gives Claude immediate awareness of branch, status, and recent work.

cd "$CLAUDE_PROJECT_DIR" 2>/dev/null || exit 0

BRANCH=$(git branch --show-current 2>/dev/null || echo "unknown")
STATUS=$(git status --short 2>/dev/null | head -20)
RECENT=$(git log --oneline -5 2>/dev/null)

CONTEXT="Current branch: $BRANCH"

if [ -n "$STATUS" ]; then
  CHANGED=$(echo "$STATUS" | wc -l | tr -d ' ')
  CONTEXT="$CONTEXT | $CHANGED uncommitted changes"
fi

if [ -n "$RECENT" ]; then
  CONTEXT="$CONTEXT
Recent commits:
$RECENT"
fi

jq -n --arg ctx "$CONTEXT" '{
  hookSpecificOutput: {
    hookEventName: "SessionStart",
    additionalContext: $ctx
  }
}'
