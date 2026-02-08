#!/usr/bin/env bash
# PostToolUse async hook: Run cargo check after .rs file edits.
# Runs in background, reports result to Claude on next turn.

INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')

# Only check Rust source files
if ! echo "$FILE_PATH" | grep -qE '\.rs$'; then
  exit 0
fi

# Skip proto generated files
if echo "$FILE_PATH" | grep -q 'OUT_DIR'; then
  exit 0
fi

# Run cargo check and report
RESULT=$(cd "$CLAUDE_PROJECT_DIR" && nix develop --command cargo check --all-targets 2>&1)
EXIT_CODE=$?

if [ $EXIT_CODE -eq 0 ]; then
  echo '{"suppressOutput": true}'
else
  ERROR_COUNT=$(echo "$RESULT" | grep -c '^error')
  FIRST_ERRORS=$(echo "$RESULT" | grep '^error' | head -3)
  jq -n --arg count "$ERROR_COUNT" --arg errors "$FIRST_ERRORS" --arg file "$FILE_PATH" '{
    systemMessage: ("cargo check failed after editing " + $file + ": " + $count + " error(s). First errors: " + $errors)
  }'
fi
