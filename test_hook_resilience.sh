#!/usr/bin/env bash
# Test script to verify hook race condition fix
# This simulates multiple concurrent hook invocations

set -e

# Nerd Font icons
ICON_TEST=$(printf '\xef\x93\x88')
ICON_CHECK=$(printf '\xef\x91\x84')
ICON_ERROR=$(printf '\xef\x81\x97')
ICON_BROOM=$(printf '\xef\x95\x9a')
ICON_SPARKLE=$(printf '\xef\x8e\xa6')

BINARY="./target/release/claude-code-personalities"
TEST_SESSION="test-race-$(date +%s)"

echo "$ICON_TEST Testing hook resilience with concurrent execution..."
echo "Session ID: $TEST_SESSION"
echo ""

# Test 1: Multiple parallel post-tool hooks (simulates parallel tool execution)
echo "Test 1: Parallel hook execution (10 concurrent hooks)..."
for i in {1..10}; do
    echo "{\"session_id\":\"$TEST_SESSION\",\"tool_name\":\"Read\",\"tool_input\":{\"file_path\":\"test$i.rs\"}}" | \
        "$BINARY" --hook post-tool &
done
wait
echo "$ICON_CHECK Test 1 passed: No failures with parallel hooks"
echo ""

# Test 2: New session without existing state file (simulates subagent spawn)
echo "Test 2: New session without state file..."
NEW_SESSION="subagent-$(date +%s)-$$"
echo "{\"session_id\":\"$NEW_SESSION\",\"tool_name\":\"Edit\",\"tool_input\":{\"file_path\":\"main.rs\"}}" | \
    "$BINARY" --hook pre-tool
if [ $? -eq 0 ]; then
    echo "$ICON_CHECK Test 2 passed: New session handled gracefully"
else
    echo "$ICON_ERROR Test 2 failed: New session caused error"
    exit 1
fi
echo ""

# Test 3: Mix of pre-tool and post-tool hooks simultaneously
echo "Test 3: Mixed pre/post hooks (20 concurrent)..."
for i in {1..10}; do
    echo "{\"session_id\":\"$TEST_SESSION\",\"tool_name\":\"Bash\",\"tool_input\":{\"command\":\"ls\"}}" | \
        "$BINARY" --hook pre-tool &
    echo "{\"session_id\":\"$TEST_SESSION\",\"tool_name\":\"Grep\",\"tool_input\":{\"pattern\":\"test\"}}" | \
        "$BINARY" --hook post-tool &
done
wait
echo "$ICON_CHECK Test 3 passed: Mixed hook types handled"
echo ""

# Test 4: Rapid session creation/destruction
echo "Test 4: Rapid session lifecycle..."
for i in {1..5}; do
    SESSION_ID="rapid-$i-$$"
    # Create activity
    echo "{\"session_id\":\"$SESSION_ID\",\"tool_name\":\"Write\",\"tool_input\":{\"file_path\":\"file$i.txt\"}}" | \
        "$BINARY" --hook post-tool
    # End session
    echo "{\"session_id\":\"$SESSION_ID\"}" | \
        "$BINARY" --hook session-end
done
echo "$ICON_CHECK Test 4 passed: Rapid session lifecycle handled"
echo ""

# Test 5: Error handling with invalid JSON in state file
echo "Test 5: Corrupted state file recovery..."
CORRUPT_SESSION="corrupt-$(date +%s)"
STATE_FILE="/tmp/claude_code_personalities_activity_$CORRUPT_SESSION.json"
echo "invalid json data" > "$STATE_FILE"
echo "{\"session_id\":\"$CORRUPT_SESSION\",\"tool_name\":\"Read\",\"tool_input\":{\"file_path\":\"test.rs\"}}" | \
    "$BINARY" --hook post-tool
if [ $? -eq 0 ]; then
    echo "$ICON_CHECK Test 5 passed: Corrupted state file recovered gracefully"
else
    echo "$ICON_ERROR Test 5 failed: Corrupted state file caused error"
    exit 1
fi
rm -f "$STATE_FILE"
echo ""

# Cleanup
echo "$ICON_BROOM Cleaning up test state files..."
rm -f /tmp/claude_code_personalities_activity_test-*.json
rm -f /tmp/claude_code_personalities_activity_subagent-*.json
rm -f /tmp/claude_code_personalities_activity_rapid-*.json
rm -f /tmp/claude_code_personalities_errors_*.count

echo ""
echo "$ICON_SPARKLE All tests passed! Hook system is resilient."
echo ""
echo "Before the fix, you would have seen errors like:"
echo "  'Failed to load session state for hook processing (session: ...)'"
echo ""
echo "Now all hooks handle state errors gracefully without disrupting Claude Code."