#!/usr/bin/env bash
# PreToolUse hook: Block raw `cargo` commands that aren't wrapped in nix/just.
# Lifelog requires nix for native deps (alsa, glib, tesseract).

INPUT=$(cat)
COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // empty')

# Allow: nix develop --command cargo, just *, cargo inside nix shell
# Block: bare `cargo build`, `cargo check`, etc.
if echo "$COMMAND" | grep -qE '^\s*cargo\s'; then
  # Check it's not wrapped in nix develop
  if ! echo "$COMMAND" | grep -q 'nix develop'; then
    echo "BLOCKED: Raw cargo commands will fail — native deps require nix. Use 'just <recipe>' or 'nix develop --command cargo ...'" >&2
    exit 2
  fi
fi

exit 0
