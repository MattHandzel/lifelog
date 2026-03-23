#!/usr/bin/env bash
# Wrapper to start AI agents with a prompt from a file.
AI=$1
MODEL=$2
PROMPT_FILE=$3

if [ -z "$AI" ] || [ -z "$PROMPT_FILE" ]; then
    echo "Usage: $0 <ai_type> [model] <prompt_file>"
    exit 1
fi

if [ ! -f "$PROMPT_FILE" ]; then
    echo "Error: Prompt file '$PROMPT_FILE' not found."
    exit 1
fi

PROMPT=$(cat "$PROMPT_FILE")

case "$AI" in
    codex)
        if [ -n "$MODEL" ]; then
            codex --model "$MODEL" --dangerously-bypass-approvals-and-sandbox -- "$PROMPT"
        else
            codex --dangerously-bypass-approvals-and-sandbox -- "$PROMPT"
        fi
        ;;
    gemini)
        if [ -n "$MODEL" ]; then
            gemini --model "$MODEL" -y -i "$PROMPT" --
        else
            gemini -y -i "$PROMPT" --
        fi
        ;;
    claude)
        if [ -n "$MODEL" ]; then
            claude --model "$MODEL" "$PROMPT"
        else
            claude "$PROMPT"
        fi
        ;;
    *)
        echo "Error: Unknown AI type '$AI'"
        exit 1
        ;;
esac
