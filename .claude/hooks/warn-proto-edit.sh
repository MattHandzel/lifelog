#!/usr/bin/env bash
# PreToolUse hook: Warn before editing proto files.
# Proto changes cascade to every crate in the workspace.

INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')

if echo "$FILE_PATH" | grep -qE '\.proto$'; then
  jq -n '{
    hookSpecificOutput: {
      hookEventName: "PreToolUse",
      permissionDecision: "ask",
      permissionDecisionReason: "WARNING: Proto file changes cascade to every crate. This will trigger full workspace rebuild. Confirm this is intentional."
    }
  }'
  exit 0
fi

exit 0
