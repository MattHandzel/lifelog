#!/usr/bin/env bash
set -euo pipefail

# Self-chaining SurrealDB removal plan runner.
# Each phase launches Claude, which commits its work and spawns the next phase.
#
# Usage:
#   ./tools/ai/surrealdb-removal/run_phase.sh <phase>          # run in current terminal
#   ./tools/ai/surrealdb-removal/run_phase.sh <phase> --tmux   # run in new tmux window

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
PHASE="${1:?Usage: run_phase.sh <phase-number> [--tmux]}"
TMUX_MODE="${2:-}"
TOTAL_PHASES=4

if [ "$PHASE" -gt "$TOTAL_PHASES" ]; then
    echo "All phases complete!"
    exit 0
fi

PROMPT_FILE="$SCRIPT_DIR/phase${PHASE}.md"
if [ ! -f "$PROMPT_FILE" ]; then
    echo "Error: $PROMPT_FILE not found"
    exit 1
fi

run_phase() {
    cd "$REPO_ROOT"
    echo "=== Starting SurrealDB Removal Phase $PHASE ==="
    echo "Prompt: $PROMPT_FILE"
    echo "========================================="; echo ""
    PROMPT="$(cat "$PROMPT_FILE")"
    claude --dangerously-skip-permissions -p "$PROMPT" --output-format text
    EXIT_CODE=$?
    echo ""; echo "=== Phase $PHASE finished (exit $EXIT_CODE) ==="
    # Keep window open so user can read output
    echo "Press Enter to close..."
    read -r
}

if [ "$TMUX_MODE" = "--tmux" ]; then
    WINDOW_NAME="surrealdb-phase-${PHASE}"
    tmux new-window -n "$WINDOW_NAME" "$SCRIPT_DIR/run_phase.sh $PHASE"
    echo "Launched phase $PHASE in tmux window '$WINDOW_NAME'"
else
    run_phase
fi
