#!/usr/bin/env bash
# tools/ai/git_diff_digest.sh
# Summarizes changes in a diff without printing every single line.

REF=${1:-main}
echo "--- Change Digest against $REF ---"

# List files changed and their status
git diff --stat "$REF"

echo "\n--- Summary of Modified Logic ---"
# Show only lines that changed significantly (ignoring imports/comments)
# Using a grep pattern to find interesting changes
git diff -U0 "$REF" | grep -E "^\+|^\-" | grep -vE "^\+\+\+|^\-\-\-" | grep -vE "import |use |//|/\*" | head -n 50
echo "..."
