#!/usr/bin/env bash
# tools/ai/bulk_replace.sh
# Safely replaces a string across the whole project and summarizes changes.

OLD_STRING=$1
NEW_STRING=$2

if [ -z "$OLD_STRING" ] || [ -z "$NEW_STRING" ]; then
    echo "Usage: tools/ai/bulk_replace.sh <old_string> <new_string>"
    exit 1
fi

echo "--- Bulk Replace Analysis ---"
echo "Replacing: '$OLD_STRING' -> '$NEW_STRING'"

# Find files that contain the old string
FILES=$(rg -l "$OLD_STRING" --type rust --type ts --type tsx --type proto --glob "!**/target/**" --glob "!**/node_modules/**")

if [ -z "$FILES" ]; then
    echo "No occurrences found."
    exit 0
fi

echo "Files to be modified:"
echo "$FILES"
echo "---"

# Perform the replacement using sed (handling different OS versions of sed)
for file in $FILES; do
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/$OLD_STRING/$NEW_STRING/g" "$file"
    else
        sed -i "s/$OLD_STRING/$NEW_STRING/g" "$file"
    fi
    echo "Modified: $file"
done

echo "--- Summary ---"
echo "Bulk replacement complete."
