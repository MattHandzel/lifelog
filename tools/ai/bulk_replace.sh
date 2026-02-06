#!/usr/bin/env bash
# Replace a pattern across multiple files, reporting only what changed.
# Usage: bulk_replace.sh <old_pattern> <new_pattern> <file_or_glob>...
# Example: bulk_replace.sh 'println!' 'tracing::info!' src/**/*.rs
set -euo pipefail

if [ "$#" -lt 3 ]; then
  echo "Usage: bulk_replace.sh <old_pattern> <new_pattern> <file_or_glob>..." >&2
  exit 2
fi

OLD="$1"; shift
NEW="$1"; shift

changed=0
skipped=0
total_replacements=0

for file in "$@"; do
  [ -f "$file" ] || continue
  count=$(grep -cF "$OLD" "$file" 2>/dev/null || true)
  if [ "$count" -gt 0 ]; then
    sed -i "s|$(printf '%s' "$OLD" | sed 's/[&/\]/\\&/g')|$(printf '%s' "$NEW" | sed 's/[&/\]/\\&/g')|g" "$file"
    printf "  %-60s %d replacements\n" "$file" "$count"
    changed=$((changed + 1))
    total_replacements=$((total_replacements + count))
  else
    skipped=$((skipped + 1))
  fi
done

echo "[bulk_replace] files_changed=$changed files_skipped=$skipped total_replacements=$total_replacements"
