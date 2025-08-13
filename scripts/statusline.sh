#!/bin/bash

# Nerd Font icons (UTF-8 byte sequences)
ICON_FOLDER=$(printf '\xef\x81\xbb')
ICON_CODE=$(printf '\xef\x84\xa1')
ICON_BUG=$(printf '\xef\x86\x88')
ICON_SEARCH=$(printf '\xef\x80\x82')
ICON_EDIT=$(printf '\xef\x81\x84')
ICON_RUN=$(printf '\xef\x83\xa7')
ICON_EYE=$(printf '\xef\x81\xae')
ICON_THINK=$(printf '\xef\x83\xab')
ICON_ROCKET=$(printf '\xef\x84\xb5')
ICON_WARNING=$(printf '\xef\x81\xb1')
ICON_ERROR=$(printf '\xef\x81\x97')
ICON_FIRE=$(printf '\xef\x81\xad')
ICON_TERMINAL=$(printf '\xef\x84\xa0')
ICON_GEAR=$(printf '\xef\x80\x93')

input=$(cat)
session_id=$(echo "$input" | jq -r '.session_id // "unknown"' 2>/dev/null || echo "unknown")
current_dir=$(echo "$input" | jq -r '.workspace.current_dir // ""' 2>/dev/null || echo "")
model_name=$(echo "$input" | jq -r '.model.display_name // "Claude"' 2>/dev/null || echo "Claude")

STATE_FILE="/tmp/claude_activity_${session_id}.json"

personality="( � ��) Booting Up"
activity_icon="$ICON_ROCKET"
activity_text="starting"
error_count=0

if [[ -f "$STATE_FILE" ]]; then
  personality=$(jq -r '.personality // "( � ��) Booting Up"' "$STATE_FILE" 2>/dev/null)
  activity=$(jq -r '.activity // "starting"' "$STATE_FILE" 2>/dev/null)
  error_count=$(jq -r '.error_count // 0' "$STATE_FILE" 2>/dev/null)
  consecutive=$(jq -r '.consecutive_actions // 0' "$STATE_FILE" 2>/dev/null)
  
  case "$activity" in
    editing)
      activity_icon="$ICON_EDIT"
      activity_text="editing"
      (( consecutive > 5 )) && activity_icon="$ICON_FIRE" && activity_text="intense"
      ;;
    executing) activity_icon="$ICON_RUN"; activity_text="running" ;;
    exploring) activity_icon="$ICON_SEARCH"; activity_text="searching" ;;
    debugging) activity_icon="$ICON_BUG"; activity_text="debugging" ;;
    reviewing) activity_icon="$ICON_EYE"; activity_text="reviewing" ;;
    thinking) activity_icon="$ICON_THINK"; activity_text="thinking" ;;
    *) activity_icon="$ICON_ROCKET"; activity_text="working" ;;
  esac
fi

dir_name=$(basename "${current_dir:-~}")

printf "\033[1m%s\033[0m" "$personality"
printf " \033[90m"\033[0m %s %s" "$ICON_FOLDER" "$dir_name"
printf " \033[90m"\033[0m %s %s" "$activity_icon" "$activity_text"

if (( error_count >= 3 )); then
  printf " \033[31m%s\033[0m" "$ICON_ERROR"
elif (( error_count > 0 )); then
  printf " \033[33m%s\033[0m" "$ICON_WARNING"
fi

case "$model_name" in
  *[Oo]pus*) printf " \033[90m"\033[0m \033[35m[%s Opus 4.1]\033[0m" "$ICON_GEAR" ;;
  *[Ss]onnet*) printf " \033[90m"\033[0m \033[36m[%s Sonnet 4]\033[0m" "$ICON_CODE" ;;
  *[Hh]aiku*) printf " \033[90m"\033[0m \033[32m[%s Haiku]\033[0m" "$ICON_TERMINAL" ;;
  *) printf " \033[90m"\033[0m [%s]" "$model_name" ;;
esac