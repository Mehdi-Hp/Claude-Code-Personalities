#!/bin/bash

# Version of Claude Code Personalities
VERSION="0.0.6"  # Default fallback

# Try to read version from installation
VERSION_FILE="$HOME/.claude/.personalities_version"
if [[ -f "$VERSION_FILE" ]]; then
    VERSION=$(cat "$VERSION_FILE")
fi

# Nerd Font icons (UTF-8 byte sequences)
ICON_FOLDER=$(printf '\xef\x81\xbb')       # folder
ICON_CODE=$(printf '\xef\x84\xa1')         # code
ICON_GIT=$(printf '\xef\x84\xa6')          # git branch
ICON_BUG=$(printf '\xef\x86\x88')          # bug
ICON_SEARCH=$(printf '\xef\x80\x82')       # search
ICON_EDIT=$(printf '\xef\x81\x84')         # edit
ICON_RUN=$(printf '\xef\x83\xa7')          # lightning
ICON_CLEAN=$(printf '\xef\x87\xb8')        # trash
ICON_EYE=$(printf '\xef\x81\xae')          # eye
ICON_THINK=$(printf '\xef\x83\xab')        # lightbulb
ICON_ROCKET=$(printf '\xef\x84\xb5')       # rocket
ICON_SLEEP=$(printf '\xef\x86\x86')        # moon
ICON_WARNING=$(printf '\xef\x81\xb1')      # warning
ICON_ERROR=$(printf '\xef\x81\x97')        # error
ICON_FIRE=$(printf '\xef\x81\xad')         # fire
ICON_TARGET=$(printf '\xef\x85\x80')       # target
ICON_CHART=$(printf '\xef\x88\x81')        # chart
ICON_TERMINAL=$(printf '\xef\x84\xa0')     # terminal
ICON_GEAR=$(printf '\xef\x80\x93')         # gear
ICON_UPDATE=$(printf '\xef\x80\xa2')       # arrow up (update available)

# Read input
input=$(cat)
session_id=$(echo "$input" | jq -r '.session_id // "unknown"' 2>/dev/null || echo "unknown")
current_dir=$(echo "$input" | jq -r '.workspace.current_dir // ""' 2>/dev/null || echo "")
model_name=$(echo "$input" | jq -r '.model.display_name // "Claude"' 2>/dev/null || echo "Claude")

# State file
STATE_FILE="/tmp/claude_activity_${session_id}.json"

# Default values
personality="( ˘ ³˘) Booting Up"
activity="Starting"
current_job=""
activity_icon="$ICON_ROCKET"
activity_text="Starting"
error_count=0

# Read state if exists
if [[ -f "$STATE_FILE" ]]; then
  personality=$(jq -r '.personality // "( ˘ ³˘) Booting Up"' "$STATE_FILE" 2>/dev/null)
  activity=$(jq -r '.activity // "Starting"' "$STATE_FILE" 2>/dev/null)
  current_job=$(jq -r '.current_job // ""' "$STATE_FILE" 2>/dev/null)
  error_count=$(jq -r '.error_count // 0' "$STATE_FILE" 2>/dev/null)
  consecutive=$(jq -r '.consecutive_actions // 0' "$STATE_FILE" 2>/dev/null)

  # Set activity icon and text (use activity directly as it's already capitalized)
  activity_text="$activity"
  case "${activity,,}" in  # Convert to lowercase for matching only
    editing)
      activity_icon="$ICON_EDIT"
      (( consecutive > 5 )) && activity_icon="$ICON_FIRE" && activity_text="Intense"
      ;;
    writing) activity_icon="$ICON_EDIT"; activity_text="Creating" ;;
    executing) activity_icon="$ICON_RUN"; activity_text="Running" ;;
    exploring) activity_icon="$ICON_SEARCH" ;;
    searching) activity_icon="$ICON_SEARCH" ;;
    debugging) activity_icon="$ICON_BUG" ;;
    testing) activity_icon="$ICON_RUN" ;;
    reviewing) activity_icon="$ICON_EYE" ;;
    thinking) activity_icon="$ICON_THINK" ;;
    *) activity_icon="$ICON_ROCKET"; activity_text="Working" ;;
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

# Directory name
dir_name=$(basename "${current_dir:-~}")

# Build status line
printf "\033[1m%s\033[0m" "$personality"
printf " \033[90m•\033[0m %s %s" "$ICON_FOLDER" "$dir_name"

# Display activity with optional job/task details
if [[ -n "$current_job" ]]; then
  printf " \033[90m•\033[0m %s %s on \033[93m%s\033[0m" "$activity_icon" "$activity_text" "$current_job"
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