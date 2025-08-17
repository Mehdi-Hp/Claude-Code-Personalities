#!/bin/bash

# Version of Claude Code Personalities
VERSION="0.0.7"  # Default fallback

# Try to read version from installation
VERSION_FILE="$HOME/.claude/.personalities_version"
if [[ -f "$VERSION_FILE" ]]; then
    VERSION=$(cat "$VERSION_FILE")
fi

# Nerd Font icons (UTF-8 byte sequences) 
# Activity icons
ICON_EDITING=$(printf '\xef\x81\x84')      # pencil
ICON_WRITING=$(printf '\xef\x85\x95')      # file-text
ICON_EXECUTING=$(printf '\xef\x83\xa7')    # lightning
ICON_READING=$(printf '\xef\x81\xae')      # eye
ICON_SEARCHING=$(printf '\xef\x80\x82')    # search
ICON_DEBUGGING=$(printf '\xef\x86\x88')    # bug
ICON_TESTING=$(printf '\xef\x92\x9b')      # flask
ICON_REVIEWING=$(printf '\xef\x81\xae')    # eye
ICON_THINKING=$(printf '\xef\x83\xab')     # lightbulb
ICON_BUILDING=$(printf '\xef\x83\xa9')     # hammer
ICON_INSTALLING=$(printf '\xef\x92\x86')   # package
ICON_IDLE=$(printf '\xef\x88\xb6')         # zzz
ICON_WORKING=$(printf '\xef\x84\xb5')      # rocket

# Status icons
ICON_WARNING=$(printf '\xef\x81\xb1')      # warning
ICON_ERROR=$(printf '\xef\x81\x97')        # error
ICON_FIRE=$(printf '\xef\x81\xad')         # fire
ICON_UPDATE=$(printf '\xef\x80\xa2')       # arrow up (update available)

# Read input
input=$(cat)
session_id=$(echo "$input" | jq -r '.session_id // "unknown"' 2>/dev/null || echo "unknown")
model_name=$(echo "$input" | jq -r '.model.display_name // "Claude"' 2>/dev/null || echo "Claude")

# State file
STATE_FILE="/tmp/claude_activity_${session_id}.json"

# Default values
personality="( ˘ ³˘) Booting Up"
activity="idle"
current_job=""
activity_icon="$ICON_IDLE"
activity_text="idle"
error_count=0

# Read state if exists
if [[ -f "$STATE_FILE" ]]; then
  personality=$(jq -r '.personality // "( ˘ ³˘) Booting Up"' "$STATE_FILE" 2>/dev/null)
  activity=$(jq -r '.activity // "idle"' "$STATE_FILE" 2>/dev/null)
  current_job=$(jq -r '.current_job // ""' "$STATE_FILE" 2>/dev/null)
  error_count=$(jq -r '.error_count // 0' "$STATE_FILE" 2>/dev/null)
  consecutive=$(jq -r '.consecutive_actions // 0' "$STATE_FILE" 2>/dev/null)

  # Set activity icon and text based on activity type
  activity_lower=$(echo "$activity" | tr '[:upper:]' '[:lower:]')
  activity_text="$activity_lower"
  
  case "$activity_lower" in
    editing)
      activity_icon="$ICON_EDITING"
      (( consecutive > 5 )) && activity_icon="$ICON_FIRE" && activity_text="intense editing"
      ;;
    writing)
      activity_icon="$ICON_WRITING"
      activity_text="writing"
      ;;
    executing)
      activity_icon="$ICON_EXECUTING" 
      activity_text="executing"
      ;;
    reading)
      activity_icon="$ICON_READING"
      activity_text="reading"
      ;;
    searching)
      activity_icon="$ICON_SEARCHING"
      activity_text="searching"
      ;;
    debugging)
      activity_icon="$ICON_DEBUGGING"
      activity_text="debugging"
      ;;
    testing)
      activity_icon="$ICON_TESTING"
      activity_text="testing"
      ;;
    reviewing)
      activity_icon="$ICON_REVIEWING"
      activity_text="reviewing"
      ;;
    thinking)
      activity_icon="$ICON_THINKING"
      activity_text="thinking"
      ;;
    building)
      activity_icon="$ICON_BUILDING"
      activity_text="building"
      ;;
    installing)
      activity_icon="$ICON_INSTALLING"
      activity_text="installing"
      ;;
    idle)
      activity_icon="$ICON_IDLE"
      activity_text="idle"
      ;;
    *)
      activity_icon="$ICON_WORKING"
      activity_text="working"
      ;;
  esac
fi

# Simple update check (once per session, no caching)
UPDATE_CHECK_FILE="/tmp/claude_personalities_session_${session_id}.update"
update_available=false
latest_version=""

# Check for updates once per session
if [[ ! -f "$UPDATE_CHECK_FILE" ]]; then
  # Mark that we've checked this session
  touch "$UPDATE_CHECK_FILE" 2>/dev/null || true
  
  # Quick check with very short timeout to not slow down statusline
  if latest=$(curl -sL --max-time 1 https://api.github.com/repos/Mehdi-Hp/claude-code-personalities/releases/latest 2>/dev/null | jq -r ".tag_name" 2>/dev/null); then
    if [[ -n "$latest" ]] && [[ "$latest" != "null" ]]; then
      latest_clean="${latest#v}"
      current_clean="${VERSION#v}"
      
      # Simple version comparison
      if [[ "$latest_clean" != "$current_clean" ]]; then
        update_available=true
        latest_version="$latest"
      fi
    fi
  fi
fi

# Build status line: [personality] [activity] [job/file] [model] [update]
printf "\033[1m%s\033[0m" "$personality"

# Display activity with optional job/task details  
if [[ -n "$current_job" ]]; then
  printf " \033[90m•\033[0m %s %s \033[93m%s\033[0m" "$activity_icon" "$activity_text" "$current_job"
else
  printf " \033[90m•\033[0m %s %s" "$activity_icon" "$activity_text"
fi

# Errors
if (( error_count >= 3 )); then
  printf " \033[31m%s\033[0m" "$ICON_ERROR"
elif (( error_count > 0 )); then
  printf " \033[33m%s\033[0m" "$ICON_WARNING"
fi

# Model with version (using North Star icon)
ICON_NORTH_STAR=$(printf '\xef\x93\xb5')
case "$model_name" in
  *[Oo]pus*4.1*) printf " \033[90m•\033[0m \033[35m[%s Opus 4.1]\033[0m" "$ICON_NORTH_STAR" ;;
  *[Oo]pus*) printf " \033[90m•\033[0m \033[35m[%s Opus]\033[0m" "$ICON_NORTH_STAR" ;;
  *[Ss]onnet*3.5*) printf " \033[90m•\033[0m \033[36m[%s Sonnet 3.5]\033[0m" "$ICON_NORTH_STAR" ;;
  *[Ss]onnet*) printf " \033[90m•\033[0m \033[36m[%s Sonnet]\033[0m" "$ICON_NORTH_STAR" ;;
  *[Hh]aiku*) printf " \033[90m•\033[0m \033[32m[%s Haiku]\033[0m" "$ICON_NORTH_STAR" ;;
  *) printf " \033[90m•\033[0m [%s]" "$model_name" ;;
esac

# Update notification
if [[ "$update_available" == true ]] && [[ -n "$latest_version" ]]; then
  printf " \033[90m•\033[0m \033[33m[%s Update %s]\033[0m" "$ICON_UPDATE" "$latest_version"
fi