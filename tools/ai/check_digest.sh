#!/usr/bin/env bash
# tools/ai/check_digest.sh
# Specifically for 'cargo check' or 'npm check', extracting only actionable errors.

echo "--- Build Health Check ---"
TEMP_OUT=$(mktemp)

# Determine project type
if [ -f "Cargo.toml" ]; then
    nix develop --command cargo check --message-format=short > "$TEMP_OUT" 2>&1
elif [ -f "package.json" ]; then
    npm run typecheck > "$TEMP_OUT" 2>&1
fi

EXIT_CODE=$?

if [ $EXIT_CODE -eq 0 ]; then
    echo "✅ Project is healthy (buildable/type-safe)."
else
    echo "❌ Build Issues Found:"
    # Short format usually gives 1 line per error
    grep -E "error|error\[" "$TEMP_OUT" | head -n 30
    echo "---"
    echo "Total errors: $(grep -c "error:" "$TEMP_OUT")"
fi

rm "$TEMP_OUT"
exit $EXIT_CODE
