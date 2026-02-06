#!/usr/bin/env bash
# Show how many matches exist per file for a pattern, to scope bulk work.
# Usage: scope_changes.sh <pattern> [path]
# Example: scope_changes.sh 'println!\|eprintln!' src/
#          scope_changes.sh 'unwrap()' common/
set -euo pipefail

if [ "$#" -lt 1 ]; then
  echo "Usage: scope_changes.sh <grep_pattern> [path]" >&2
  exit 2
fi

PATTERN="$1"
PATH_ARG="${2:-.}"

total=0
file_count=0

while IFS=: read -r file count; do
  [ "$count" -gt 0 ] || continue
  printf "  %-60s %d\n" "$file" "$count"
  total=$((total + count))
  file_count=$((file_count + 1))
done < <(grep -rcE "$PATTERN" "$PATH_ARG" --include='*.rs' 2>/dev/null | sort -t: -k2 -nr)

echo "[scope] total=$total files=$file_count"
