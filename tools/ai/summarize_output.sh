#!/usr/bin/env bash
set -euo pipefail
out() { printf '%s\n' "$*" 2>/dev/null || true; }

LINES=120
MATCHES=40
TOP=20

while [ "$#" -gt 0 ]; do
  case "$1" in
    --lines)
      LINES="$2"; shift 2 ;;
    --matches)
      MATCHES="$2"; shift 2 ;;
    --top)
      TOP="$2"; shift 2 ;;
    -h|--help)
      cat <<'USAGE'
Usage: summarize_output.sh [--lines N] [--matches N] [--top N]
Reads from stdin and prints a compact digest.
USAGE
      exit 0 ;;
    *)
      echo "Unknown arg: $1" >&2
      exit 2 ;;
  esac
done

tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT

cat > "$tmp"

line_count=$(wc -l < "$tmp" | tr -d ' ')
byte_count=$(wc -c < "$tmp" | tr -d ' ')

out "[digest] lines=$line_count bytes=$byte_count"

# Error/warn counts (case-insensitive).
# Avoid grep exit-code semantics (1 when no matches) under pipefail.
err_count=$(
  awk 'BEGIN{IGNORECASE=1}
    {
      # Portable "word-ish" boundary match.
      if ($0 ~ /(^|[^[:alnum:]_])(error|exception|panic|failed|fail)([^[:alnum:]_]|$)/) c++
    }
    END{print c+0}' "$tmp" | tr -d ' '
)
warn_count=$(
  awk 'BEGIN{IGNORECASE=1}
    {
      if ($0 ~ /(^|[^[:alnum:]_])(warn|warning)([^[:alnum:]_]|$)/) c++
    }
    END{print c+0}' "$tmp" | tr -d ' '
)

if [ "$err_count" -gt 0 ] || [ "$warn_count" -gt 0 ]; then
  out "[digest] errors=$err_count warnings=$warn_count"
fi

# Top repeated lines
if [ "$TOP" -gt 0 ]; then
  out "[digest] top repeated lines"
  # `head` intentionally terminates early; silence upstream SIGPIPE noise.
  sort "$tmp" 2>/dev/null | uniq -c | sort -nr 2>/dev/null | head -n "$TOP" | sed 's/^/  /'
fi

# Error/exception highlights
if [ "$MATCHES" -gt 0 ]; then
  out "[digest] error/warn highlights"
  grep -inE "(error|exception|panic|failed|fail|warn|warning|stacktrace|backtrace)" "$tmp" \
    | head -n "$MATCHES" | sed 's/^/  /' || true
fi

# Tail of output
if [ "$LINES" -gt 0 ]; then
  out "[digest] last ${LINES} lines"
  tail -n "$LINES" "$tmp" | sed 's/^/  /'
fi
