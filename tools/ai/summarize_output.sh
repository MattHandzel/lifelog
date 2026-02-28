#!/usr/bin/env bash
# tools/ai/summarize_output.sh
# Clusters and summarizes repetitive log output to reduce context bloat.

echo "--- Log Output Summary ---"

# Use awk to cluster similar lines (ignoring timestamps/hashes if possible)
# This is a basic implementation that can be refined
cluster_logs() {
    # Remove common timestamp patterns and hex hashes to cluster similar events
    sed -E 's/[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}//g' | \
    sed -E 's/0x[0-9a-fA-F]+//g' | \
    sed -E 's/[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}/<uuid>/g' | \
    sort | uniq -c | sort -rn | head -n 50
}

if [ -p /dev/stdin ]; then
    cluster_logs
elif [ -f "$1" ]; then
    cat "$1" | cluster_logs
else
    echo "Usage: some_command | tools/ai/summarize_output.sh OR tools/ai/summarize_output.sh <log_file>"
    exit 1
fi
