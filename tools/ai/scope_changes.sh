#!/usr/bin/env bash
# tools/ai/scope_changes.sh
# Performs impact analysis by finding all references to a symbol across the project.

SYMBOL=$1
if [ -z "$SYMBOL" ]; then
    echo "Usage: tools/ai/scope_changes.sh <symbol_name>"
    exit 1
fi

echo "--- Impact Analysis for Symbol: $SYMBOL ---"

# Search for the symbol in Rust, TypeScript, and Proto files
# Excluding tests and common noisy directories
rg -w "$SYMBOL" --type rust --type ts --type tsx --type proto \
    --glob "!**/tests/**" \
    --glob "!**/target/**" \
    --glob "!**/node_modules/**" \
    --line-number --column --context 2 \
    | head -n 100

echo "\n--- Summary ---"
echo "Total occurrences found: $(rg -w "$SYMBOL" --count --glob "!**/tests/**" --glob "!**/target/**" --glob "!**/node_modules/**")"
