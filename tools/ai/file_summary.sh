#!/usr/bin/env bash
# tools/ai/file_summary.sh
# Summarizes a file's structure (symbols, functions) instead of reading full content.

FILE=$1
echo "--- Structure of $FILE ---"

if [[ $FILE == *.rs ]]; then
    grep -E "pub (fn|struct|enum|trait|type)" "$FILE" | head -n 100
elif [[ $FILE == *.ts ]] || [[ $FILE == *.tsx ]]; then
    grep -E "export (function|const|class|interface|type|enum)" "$FILE" | head -n 100
elif [[ $FILE == *.proto ]]; then
    grep -E "(message|service|rpc|enum)" "$FILE" | head -n 100
else
    echo "(Generic File) First 20 lines:"
    head -n 20 "$FILE"
fi
