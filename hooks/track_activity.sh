#!/bin/bash

input=$(cat)
session_id=$(echo "$input" | jq -r '.session_id // "unknown"')
tool_name=$(echo "$input" | jq -r '.tool_name // ""')
tool_input=$(echo "$input" | jq -r '.tool_input // {}')

STATE_FILE="/tmp/claude_activity_${session_id}.json"
ERROR_FILE="/tmp/claude_errors_${session_id}.count"

if [[ -f "$STATE_FILE" ]]; then
  prev_activity=$(jq -r '.activity // "idle"' "$STATE_FILE")
  consecutive=$(jq -r '.consecutive_actions // 0' "$STATE_FILE")
else
  prev_activity="idle"
  consecutive=0
fi

errors=$(cat "$ERROR_FILE" 2>/dev/null || echo 0)
if echo "$input" | jq -e '.tool_response.error != null' >/dev/null 2>&1; then
  ((errors++))
  echo "$errors" > "$ERROR_FILE"
fi

cmd=$(echo "$tool_input" | jq -r '.command // ""' 2>/dev/null)
file=$(echo "$tool_input" | jq -r '.file_path // ""' 2>/dev/null)

case "$tool_name" in
  "Edit"|"MultiEdit") activity="editing" ;;
  "Write") activity="writing" ;;
  "Bash") activity="executing" ;;
  "Read"|"Grep") activity="exploring" ;;
  *) activity="thinking" ;;
esac

[[ "$activity" == "$prev_activity" ]] && ((consecutive++)) || consecutive=1

personality="⚡(◡ ‿ ◡ ✿) AI Engineer"

# Frustration states based on error count
if (( errors >= 5 )); then
  personality="(╯°□°)╯︵ ┻━┻ Table Flipper"
elif (( errors >= 3 )); then
  personality="(┛ಠДಠ)┛彡┻━┻ Frustrated Developer"
# Git operations
elif [[ "$tool_name" == "Bash" ]] && echo "$cmd" | grep -q "^git "; then
  personality="┗(▀̿Ĺ̯▀̿ ̿)┓ Git Manager"
# Testing
elif echo "$cmd" | grep -qiE "test|spec"; then
  personality="(¬_¬) Test Engineer"
# Debugging
elif echo "$cmd" | grep -qiE "debug|console\.log"; then
  personality="( ͡° ͜ʖ ͡°) Mischievous Debugger"
elif [[ "$tool_name" == "Grep" ]]; then
  personality="(つ◉益◉)つ Bug Hunter"
# File type based
elif echo "$file" | grep -qiE "readme|\.md$"; then
  personality="(͡• ͜໒ ͡• ) Documentation Writer"
elif echo "$file" | grep -qiE "\.(jsx?|tsx?)$"; then
  personality="ʕ•ᴥ•ʔ UI Developer"
elif echo "$file" | grep -qiE "auth|security"; then
  personality="ಠ_ಠ Security Analyst"
elif echo "$file" | grep -qiE "\.css$|\.scss$|\.sass$"; then
  personality="♥‿♥ Style Artist"
elif echo "$file" | grep -qiE "config|\.json$|\.yaml$|\.yml$"; then
  personality="♥‿♥ Config Helper"
# Long sessions
elif (( consecutive > 20 )); then
  personality="【╯°□°】╯︵ ┻━┻ Code Berserker"
elif (( consecutive > 10 )); then
  personality="┌༼◉ل͟◉༽┐ Hyperfocused Coder"
# Time-based personalities
else
  hour=$(date +%H)
  if (( hour >= 6 && hour < 12 )); then
    personality="☕(◡ ‿ ◡ ✿) Morning Engineer"
  elif (( hour >= 12 && hour < 17 )); then
    personality="【≽ܫ≼】 Afternoon Thinker"
  elif (( hour >= 17 && hour < 22 )); then
    personality="(◕‿◕) Evening Explorer"
  else
    personality="˙ ͜ʟ˙ Night Coder"
  fi
  
  # Override with tool-specific personalities
  case "$tool_name" in
    "Edit") personality="ʕ•ᴥ•ʔ Code Wizard" ;;
    "Write") personality="(• ε •) Gentle Refactorer" ;;
    "Delete") personality="(ง'̀-'́)ง Dead Code Remover" ;;
    "Review") personality="¯\_(ツ)_/¯ Casual Code Reviewer" ;;
    "Read") 
      if (( consecutive > 5 )); then
        personality="⋋| ◉ ͟ʖ ◉ |⋌ Search Maestro"
      else
        personality="【≽ܫ≼】 Research King"
      fi
      ;;
    "Bash") 
      if echo "$cmd" | grep -qiE "deploy|docker"; then
        personality="( ͡ _ ͡°)ﾉ⚲ Deployment Guard"
      elif echo "$cmd" | grep -qiE "database|sql|mongo|postgres"; then
        personality="⚆_⚆ Database Expert"
      else
        personality="( ͡ _ ͡°)ノ⚡ DevOps Engineer"
      fi
      ;;
  esac
fi

# Performance-related personalities
if echo "$cmd" | grep -qiE "profile|performance|benchmark"; then
  personality="'(ᗒᗣᗕ)՞ Performance Optimizer"
elif echo "$file" | grep -qiE "optimize|performance"; then
  personality="★⌒ヽ( ͡° ε ͡°) Performance Tuner"
fi

# Quality-related personalities
if echo "$file" | grep -qiE "lint|eslint|prettier"; then
  personality="(ㆆ_ㆆ) Quality Auditor"
fi

cat > "$STATE_FILE" <<JSON
{
  "session_id": "$session_id",
  "activity": "$activity",
  "personality": "$personality",
  "consecutive_actions": $consecutive,
  "error_count": $errors
}
JSON

exit 0