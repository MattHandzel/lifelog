#!/usr/bin/env bash
# Generate a High-Signal Repository Map for AI agents to prevent discovery waste.

OUTPUT="docs/REPO_DISCOVERY_MAP.json"

echo "🔍 Indexing repository for agent discovery..."

{
  echo "{"
  echo "  "tauri_commands": ["
  rg --no-heading --line-number 'tauri::command' interface/src-tauri/src/main.rs | sed 's/.*fn \([a-zA-Z0-9_]*\).*/    "\1",/' | sed '$s/,$//'
  echo "  ],"
  echo "  "grpc_services": ["
  rg --no-heading --line-number 'rpc [a-zA-Z0-9_]*' proto/lifelog.proto | sed 's/.*rpc \([a-zA-Z0-9_]*\).*/    "\1",/' | sed '$s/,$//'
  echo "  ],"
  echo "  "react_components": ["
  fd -e tsx -e ts . interface/src/components --exec echo '    "{}",' | sed 's|interface/src/components/||g' | sed '$s/,$//'
  echo "  ],"
  echo "  "rust_modules": ["
  fd -e rs . server/src collector/src --exec echo '    "{}",' | sed '$s/,$//'
  echo "  ]"
  echo "}"
} > "$OUTPUT"

echo "✅ Discovery map generated at $OUTPUT"
