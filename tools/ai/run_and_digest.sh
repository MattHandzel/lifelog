#!/usr/bin/env bash
# tools/ai/run_and_digest.sh
# Runs a command and provides a high-signal digest of its output to the AI.

COMMAND="$@"
TEMP_OUT=$(mktemp)

echo "--- Executing: $COMMAND ---"
eval "$COMMAND" > "$TEMP_OUT" 2>&1
EXIT_CODE=$?

if [ $EXIT_CODE -eq 0 ]; then
    echo "✅ SUCCESS: Command finished with exit code 0."
    echo "--- Digest ---"
    grep -E "Finished|Compiling|Success|Done|Ready" "$TEMP_OUT" | tail -n 5
else
    echo "❌ FAILURE: Command exited with code $EXIT_CODE."
    echo "--- Error Digest (Last 20 lines of unique errors) ---"
    # Filter for common error patterns to reduce noise
    grep -Ei "error|fail|exception|panic|fatal" "$TEMP_OUT" | head -n 40 | uniq
    echo "..."
    tail -n 10 "$TEMP_OUT"
fi

rm "$TEMP_OUT"
exit $EXIT_CODE
