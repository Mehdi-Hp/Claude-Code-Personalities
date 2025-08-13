#!/bin/bash
input=$(cat)
session_id=$(echo "$input" | jq -r '.session_id // "unknown"')
echo "0" > "/tmp/claude_errors_${session_id}.count"
exit 0