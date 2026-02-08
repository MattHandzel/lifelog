#!/usr/bin/env bash
# PreToolUse hook: Block edits that introduce `unsafe` in non-test code.
# Project policy: zero new unsafe code.

INPUT=$(cat)
NEW_STRING=$(echo "$INPUT" | jq -r '.tool_input.new_string // .tool_input.content // empty')
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')

# Only check Rust files
if ! echo "$FILE_PATH" | grep -qE '\.rs$'; then
  exit 0
fi

# Skip test files
if echo "$FILE_PATH" | grep -qE '(test|tests)/'; then
  exit 0
fi

# Check if the new content introduces unsafe
if echo "$NEW_STRING" | grep -qE '\bunsafe\b'; then
  # Allow if it's in a test block or comment
  if echo "$NEW_STRING" | grep -qE '(#\[cfg\(test\)\]|// unsafe|/// unsafe)'; then
    exit 0
  fi
  jq -n '{
    hookSpecificOutput: {
      hookEventName: "PreToolUse",
      permissionDecision: "deny",
      permissionDecisionReason: "BLOCKED: New unsafe code is not allowed. Project policy requires zero unsafe. Use safe alternatives or discuss with the user."
    }
  }'
  exit 0
fi

exit 0
