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
pattern=$(echo "$tool_input" | jq -r '.pattern // ""' 2>/dev/null)

# Helper function to trim long filenames
trim_filename() {
  local name="$1"
  local max_len="${2:-20}"
  
  # Get just the filename without path
  name=$(basename "$name")
  
  # If filename is too long, truncate it but keep extension
  if [[ ${#name} -gt $max_len ]]; then
    local ext="${name##*.}"
    local base="${name%.*}"
    local keep_len=$((max_len - ${#ext} - 4))  # -4 for "..." and "."
    if [[ $keep_len -gt 0 ]]; then
      name="${base:0:$keep_len}...${ext}"
    else
      name="${name:0:$max_len}"
    fi
  fi
  
  echo "$name"
}

# Set activity and current job/task
current_job=""
case "$tool_name" in
  "Edit"|"MultiEdit") 
    activity="Editing"
    if [[ -n "$file" ]]; then
      current_job=$(trim_filename "$file")
    fi
    ;;
  "Write") 
    activity="Writing"
    if [[ -n "$file" ]]; then
      current_job=$(trim_filename "$file")
    fi
    ;;
  "Bash") 
    activity="Executing"
    if [[ -n "$cmd" ]]; then
      # Just show the command name, not the full command
      current_job=$(echo "$cmd" | cut -d' ' -f1 | cut -d'/' -f1)
    fi
    ;;
  "Read") 
    activity="Exploring"
    if [[ -n "$file" ]]; then
      current_job=$(trim_filename "$file")
    fi
    ;;
  "Grep")
    activity="Searching"
    if [[ -n "$pattern" ]]; then
      # Show truncated pattern for grep searches
      current_job=$(echo "$pattern" | head -c 20)
    fi
    ;;
  *) 
    activity="Thinking"
    current_job=""
    ;;
esac

[[ "$activity" == "$prev_activity" ]] && ((consecutive++)) || consecutive=1

personality="(◕‿◕) Claude Assistant"

# Frustration states based on error count
if (( errors >= 5 )); then
  personality="(╯°□°)╯︵ ┻━┻ Table Flipper"
elif (( errors >= 3 )); then
  personality="(ノಠ益ಠ)ノ Error Warrior"
# Git operations
elif [[ "$tool_name" == "Bash" ]] && echo "$cmd" | grep -q "^git "; then
  personality="┗(▀̿Ĺ̯▀̿ ̿)┓ Git Manager"
# Testing - only for actual test execution
elif [[ "$tool_name" == "Bash" ]] && echo "$cmd" | grep -qiE "^(npm|yarn|pnpm) test|^pytest|^jest|^mocha|^vitest|^cargo test|^go test"; then
  personality="( ദ്ദി ˙ᗜ˙ ) Test Taskmaster"
# Debugging with Grep
elif [[ "$tool_name" == "Grep" ]]; then
  personality="(つ◉益◉)つ Bug Hunter"
# File type based
elif echo "$file" | grep -qiE "readme|\.md$"; then
  personality="φ(．．) Documentation Writer"
elif echo "$file" | grep -qiE "\.(jsx?|tsx?)$"; then
  personality="(✿◠ᴗ◠) UI Developer"
elif echo "$file" | grep -qiE "auth|security"; then
  personality="ಠ_ಠ Security Analyst"
elif echo "$file" | grep -qiE "\.css$|\.scss$|\.sass$"; then
  personality="♥‿♥ Style Artist"
elif echo "$file" | grep -qiE "config|\.json$|\.yaml$|\.yml$"; then
  personality="(๑>؂•̀๑) Config Helper"
# Long sessions
elif (( consecutive > 20 )); then
  personality="【╯°□°】╯︵ ┻━┻ Code Berserker"
elif (( consecutive > 10 )); then
  personality="┌༼◉ل͟◉༽┐ Hyperfocused Coder"
# Time-based personalities
else
  hour=$(date +%H)
  if (( hour >= 6 && hour < 12 )); then
    # Coffee icon: f0f4 (Font Awesome coffee)
    ICON_COFFEE=$(printf '\xef\x83\xb4')
    personality="( ˶˘ ³˘)${ICON_COFFEE} Coffee Powered"
  elif (( hour >= 12 && hour < 17 )); then
    personality="(つ°ヮ°)つ Afternoon Thinker"
  elif (( hour >= 17 && hour < 22 )); then
    personality="(￣ω￣;) Evening Explorer"
  else
    personality="˙ ͜ʟ˙ Night Coder"
  fi
  
  # Override with tool-specific personalities
  case "$tool_name" in
    "Edit") personality="(⌐■_■) Code Wizard" ;;
    "Write") personality="(• ε •) Gentle Refactorer" ;;
    "Delete") personality="(ง'̀-'́)ง Code Janitor" ;;
    "Review") personality="¯\_(ツ)_/¯ Casual Code Reviewer" ;;
    "Read") 
      if (( consecutive > 5 )); then
        personality="⋋| ◉ ͟ʖ ◉ |⋌ Search Maestro"
      else
        personality="╭༼ ººل͟ºº ༽╮ Research King"
      fi
      ;;
    "Bash") 
      # Deployment/Infrastructure
      if echo "$cmd" | grep -qiE "\bdeploy\b|docker-compose|rollout|release"; then
        personality="( ͡ _ ͡°)ﾉ⚲ Deployment Guard"
      # Database operations
      elif echo "$cmd" | grep -qiE "database|sql|mongo|postgres|mysql|redis|sqlite"; then
        personality="⚆_⚆ Database Expert"
      # File operations (with proper spacing and more commands)
      elif echo "$cmd" | grep -qiE "^(ls |cd |mkdir |rm |mv |cp |find |touch |tree |pwd |cat |less |more |head |tail )"; then
        personality="ᓚ₍ ^. .^₎ File Explorer"
      # Build/Run scripts
      elif echo "$cmd" | grep -qiE "^(npm|yarn|pnpm|cargo|pip|gem) (run|build|compile|dev|start|serve)"; then
        personality="ᕦ(ò_óˇ)ᕤ Compilation Warrior"
      # Package management
      elif echo "$cmd" | grep -qiE "^(npm|yarn|pnpm|pip|cargo|gem|brew|apt|yum|composer) (install|add|remove|update|upgrade|uninstall)"; then
        personality="^⎚-⎚^ Dependency Wrangler"
      # Build tools
      elif echo "$cmd" | grep -qiE "^(make |build |compile |webpack|vite|rollup|tsc |babel|gradle)"; then
        personality="ᕦ(ò_óˇ)ᕤ Compilation Warrior"
      # Process management
      elif echo "$cmd" | grep -qiE "^(ps |kill |killall |top|htop|jobs|fg |bg |nohup |pkill )"; then
        personality="(╬ ಠ益ಠ) Task Assassin"
      # Network operations
      elif echo "$cmd" | grep -qiE "^(curl |wget |ping |ssh |scp |rsync |netstat|nc |telnet|ftp )"; then
        personality="(⌐▀̯▀) Network Ninja"
      # System monitoring
      elif echo "$cmd" | grep -qiE "^(df |du |free|uname|whoami|which |hostname|uptime|lscpu)"; then
        personality="(◉_◉) System Detective"
      # System administration
      elif echo "$cmd" | grep -qiE "^(systemctl |service |journalctl |cron|launchctl)"; then
        personality="( ͡ಠ ʖ̯ ͡ಠ) System Admin"
      # Permissions
      elif echo "$cmd" | grep -qiE "^(chmod |chown |sudo |su |umask|passwd|usermod)"; then
        personality="(╯‵□′)╯ Permission Police"
      # Text processing (including modern tools)
      elif echo "$cmd" | grep -qiE "^(grep |sed |awk |cut |sort |uniq |wc |rg |fd |ag )"; then
        personality="(˘▾˘~) String Surgeon"
      # Text editors
      elif echo "$cmd" | grep -qiE "^(vi |vim |nano |emacs |code |subl |atom )"; then
        personality="( . .)φ Editor User"
      # Archive/Compression
      elif echo "$cmd" | grep -qiE "^(tar |zip |unzip |gzip |gunzip|7z |rar )"; then
        personality="(っ˘ڡ˘ς) Compression Chef"
      # Environment/Shell
      elif echo "$cmd" | grep -qiE "^(export |source |echo |env|set |alias |history)"; then
        personality="(∗´ര ᎑ ര\`∗) Environment Enchanter"
      # Version control (non-git)
      elif echo "$cmd" | grep -qiE "^(svn |hg |cvs |diff |patch )"; then
        personality="(╯︵╰,) Code Historian"
      # Container/Orchestration
      elif echo "$cmd" | grep -qiE "^(docker |podman |kubectl |helm |k9s )"; then
        personality="(づ｡◕‿‿◕｡)づ Container Captain"
      else
        personality="( ╹ -╹)? Command Wonderer"
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
  personality="৻( •̀ ᗜ •́ ৻) Quality Auditor"
fi

cat > "$STATE_FILE" <<JSON
{
  "session_id": "$session_id",
  "activity": "$activity",
  "current_job": "$current_job",
  "personality": "$personality",
  "consecutive_actions": $consecutive,
  "error_count": $errors
}
JSON

exit 0