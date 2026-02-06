#!/usr/bin/env bash
# Show a compact summary of a Rust file without reading the full contents.
# Usage: file_summary.sh <file.rs> [file2.rs ...]
# Output: line count, use/mod imports, pub signatures, TODO/FIXME counts
set -euo pipefail

if [ "$#" -eq 0 ]; then
  echo "Usage: file_summary.sh <file.rs> [file2.rs ...]" >&2
  exit 2
fi

for file in "$@"; do
  [ -f "$file" ] || { echo "[skip] $file (not found)"; continue; }

  lines=$(wc -l < "$file" | tr -d ' ')
  todo_count=$(grep -ciE 'TODO|FIXME|HACK|XXX' "$file" 2>/dev/null || echo 0)
  unsafe_count=$(grep -c 'unsafe' "$file" 2>/dev/null || echo 0)
  expect_count=$(grep -cE '\.(expect|unwrap)\(' "$file" 2>/dev/null || echo 0)
  println_count=$(grep -cE '(println!|eprintln!)\(' "$file" 2>/dev/null || echo 0)

  echo "=== $file ($lines lines) ==="
  echo "  todos=$todo_count unsafe=$unsafe_count expect/unwrap=$expect_count println=$println_count"

  # Show pub items (signatures only, no body)
  echo "  [pub items]"
  grep -nE '^\s*pub\s+(fn|struct|enum|trait|type|mod|const|static|async fn)' "$file" \
    | sed 's/^/    /' | head -30 || true

  # Show use/mod declarations
  echo "  [imports]"
  grep -nE '^\s*(use |mod |pub mod |pub use )' "$file" \
    | sed 's/^/    /' | head -20 || true

  echo ""
done
