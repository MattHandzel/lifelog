#!/bin/bash
# Wrapper to start Codex with a prompt from a file to avoid quoting issues.
MODEL=$1
PROMPT_FILE=$2

if [ -z "$MODEL" ] || [ -z "$PROMPT_FILE" ]; then
    echo "Usage: $0 <model> <prompt_file>"
    exit 1
fi

if [ ! -f "$PROMPT_FILE" ]; then
    echo "Error: Prompt file '$PROMPT_FILE' not found."
    exit 1
fi

PROMPT=$(cat "$PROMPT_FILE")
exec codex --model "$MODEL" "$PROMPT"
