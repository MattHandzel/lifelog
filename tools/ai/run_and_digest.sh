#!/usr/bin/env bash
set -euo pipefail

out() { printf '%s\n' "$*" 2>/dev/null || true; }

SHOW_FULL=0

if [ "$#" -eq 0 ]; then
  echo "Usage: run_and_digest.sh [--full] -- <command> [args...]" >&2
  exit 2
fi

if [ "${1:-}" = "--full" ]; then
  SHOW_FULL=1
  shift
fi

if [ "${1:-}" = "--" ]; then
  shift
fi

if [ "$#" -eq 0 ]; then
  echo "No command provided." >&2
  exit 2
fi

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT

set +e
"$@" > "$tmp" 2>&1
exit_code=$?
set -e

out "[run] cmd=$*"
out "[run] exit_code=$exit_code"

if [ "$SHOW_FULL" -eq 1 ]; then
  cat "$tmp"
else
  "$SCRIPT_DIR/summarize_output.sh" < "$tmp"
fi

exit "$exit_code"
