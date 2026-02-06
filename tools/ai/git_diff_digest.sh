#!/usr/bin/env bash
set -euo pipefail
out() { printf '%s\n' "$*" 2>/dev/null || true; }

DIFF_ARGS=""

if [ "${1:-}" = "--cached" ]; then
  DIFF_ARGS="--cached"
  shift
fi

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

out "[git] diff --stat $DIFF_ARGS"
# shellcheck disable=SC2086
 git diff --stat $DIFF_ARGS

# shellcheck disable=SC2086
 git diff $DIFF_ARGS | "$SCRIPT_DIR/summarize_output.sh" --lines 120 --matches 80 --top 20
