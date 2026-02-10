#!/usr/bin/env bash
# Specialized digest for cargo check/clippy/test output.
# Usage: check_digest.sh <command> [args...]
# Example: check_digest.sh cargo check --all-targets
#          check_digest.sh cargo clippy --all-targets -- -D warnings
#          check_digest.sh cargo nextest run
set -euo pipefail

if [ "$#" -eq 0 ]; then
  echo "Usage: check_digest.sh <command> [args...]" >&2
  exit 2
fi

tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT

set +e
"$@" > "$tmp" 2>&1
exit_code=$?
set -e

line_count=$(wc -l < "$tmp" | tr -d ' ')
# grep -c prints "0" but exits 1 when there are no matches; don't append a second "0".
err_count=$(grep -ciE '^error' "$tmp" 2>/dev/null || true)
warn_count=$(grep -ciE '^warning' "$tmp" 2>/dev/null || true)
err_count=${err_count:-0}
warn_count=${warn_count:-0}

echo "[check] cmd=$*"
echo "[check] exit_code=$exit_code lines=$line_count errors=$err_count warnings=$warn_count"

if [ "$exit_code" -ne 0 ]; then
  echo "[check] FAILED — first 5 errors:"
  grep -E '^error' "$tmp" | head -5 | sed 's/^/  /'
  echo ""
  echo "[check] last 10 lines:"
  tail -10 "$tmp" | sed 's/^/  /'
elif [ "$warn_count" -gt 0 ]; then
  echo "[check] PASSED with warnings — first 5:"
  grep -E '^warning' "$tmp" | head -5 | sed 's/^/  /'
else
  echo "[check] PASSED (clean)"
fi

exit "$exit_code"
