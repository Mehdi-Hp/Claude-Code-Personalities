#!/bin/bash

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

# Read input
input=$(cat)
session_id=$(echo "$input" | jq -r '.session_id // "unknown"' 2>/dev/null || echo "unknown")
current_dir=$(echo "$input" | jq -r '.workspace.current_dir // ""' 2>/dev/null || echo "")
model_name=$(echo "$input" | jq -r '.model.display_name // "Claude"' 2>/dev/null || echo "Claude")

# State file
STATE_FILE="/tmp/claude_activity_${session_id}.json"

# Default values
personality="( ˘ ³˘) Booting Up"
activity="starting"
current_job=""
activity_icon="$ICON_ROCKET"
activity_text="starting"
error_count=0

# Read state if exists
if [[ -f "$STATE_FILE" ]]; then
  personality=$(jq -r '.personality // "( ˘ ³˘) Booting Up"' "$STATE_FILE" 2>/dev/null)
  activity=$(jq -r '.activity // "starting"' "$STATE_FILE" 2>/dev/null)
  current_job=$(jq -r '.current_job // ""' "$STATE_FILE" 2>/dev/null)
  error_count=$(jq -r '.error_count // 0' "$STATE_FILE" 2>/dev/null)
  consecutive=$(jq -r '.consecutive_actions // 0' "$STATE_FILE" 2>/dev/null)

  # Set activity icon and text
  case "$activity" in
    editing)
      activity_icon="$ICON_EDIT"
      activity_text="editing"
      (( consecutive > 5 )) && activity_icon="$ICON_FIRE" && activity_text="intense"
      ;;
    writing) activity_icon="$ICON_EDIT"; activity_text="creating" ;;
    executing) activity_icon="$ICON_RUN"; activity_text="running" ;;
    exploring) activity_icon="$ICON_SEARCH"; activity_text="searching" ;;
    debugging) activity_icon="$ICON_BUG"; activity_text="debugging" ;;
    testing) activity_icon="$ICON_RUN"; activity_text="testing" ;;
    reviewing) activity_icon="$ICON_EYE"; activity_text="reviewing" ;;
    thinking) activity_icon="$ICON_THINK"; activity_text="thinking" ;;
    *) activity_icon="$ICON_ROCKET"; activity_text="working" ;;
  esac
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

# Model
case "$model_name" in
  *[Oo]pus*) printf " \033[90m•\033[0m \033[35m[%s opus]\033[0m" "$ICON_GEAR" ;;
  *[Ss]onnet*) printf " \033[90m•\033[0m \033[36m[%s sonnet]\033[0m" "$ICON_CODE" ;;
  *[Hh]aiku*) printf " \033[90m•\033[0m \033[32m[%s haiku]\033[0m" "$ICON_TERMINAL" ;;
  *) printf " \033[90m•\033[0m [%s]" "$model_name" ;;
esac