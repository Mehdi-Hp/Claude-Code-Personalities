#!/bin/bash
input=$(cat)
session_id=$(echo "$input" | jq -r '.session_id // "unknown"')
rm -f "/tmp/claude_activity_${session_id}.json"
rm -f "/tmp/claude_errors_${session_id}.count"
exit 0