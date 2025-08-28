#!/bin/bash

# Terminal 256-Color Visualization Script
# Displays all 256 terminal colors with indices for easy color selection
# Usage: ./visualize_colors.sh [compact|full|search N]

set -euo pipefail

# Function to get theme color description
get_theme_color_desc() {
    case $1 in
        32) echo "Haiku Model (Teal)" ;;
        33) echo "Git Manager (Dark Cyan)" ;;
        69) echo "Activity/Icons (Bright Blue-Cyan)" ;;
        75) echo "Documentation Writer (Light Blue)" ;;
        82) echo "Success/Complete (Bright Green)" ;;
        121) echo "Sonnet Model (Light Purple)" ;;
        197) echo "Error/Frustrated (Bright Red/Pink)" ;;
        183) echo "Thinking/Processing (Soft Purple-Pink)" ;;
        202) echo "Happy/Success (Orange)" ;;
        222) echo "Hyperfocused (Light Yellow/Cream)" ;;
        226) echo "Opus Model (Yellow)" ;;
        227) echo "Searching/Detective (Light Yellow)" ;;
        231) echo "Directory/File (Bright White)" ;;
        234) echo "Separators (Dark Gray)" ;;
        254) echo "Base/Neutral (Very Light Gray)" ;;
        *) echo "" ;;
    esac
}

# Check if color is used in theme
is_theme_color() {
    local color_desc
    color_desc=$(get_theme_color_desc "$1")
    [[ -n "$color_desc" ]]
}

# ANSI escape sequences
ESC=$'\e'
RESET="${ESC}[0m"
BOLD="${ESC}[1m"

# Function to display a color block with index
show_color() {
    local index=$1
    local fg_color="${ESC}[38;5;${index}m"
    local bg_color="${ESC}[48;5;${index}m"
    
    # Check if this color is used in theme
    local theme_usage=""
    local color_desc
    color_desc=$(get_theme_color_desc "$index")
    if [[ -n "$color_desc" ]]; then
        theme_usage=" ← $color_desc"
    fi
    
    printf "${fg_color}${BOLD}%3d${RESET} ${bg_color}   ${RESET} ${fg_color}Sample Text${RESET}%s\n" \
        "$index" "$theme_usage"
}

# Function to display color range with title
show_color_range() {
    local title=$1
    local start=$2
    local end=$3
    local cols=${4:-16}
    
    echo -e "\n${BOLD}=== $title ===${RESET}"
    
    local count=0
    for ((i=start; i<=end; i++)); do
        if ((count % cols == 0 && count > 0)); then
            echo
        fi
        
        local fg_color="${ESC}[38;5;${i}m"
        local bg_color="${ESC}[48;5;${i}m"
        
        # Highlight theme colors
        if is_theme_color "$i"; then
            printf "${BOLD}${fg_color}%3d${RESET}${bg_color} ${RESET} " "$i"
        else
            printf "${fg_color}%3d${RESET}${bg_color} ${RESET} " "$i"
        fi
        
        ((count++))
    done
    echo
}

# Function to show detailed color info
show_detailed_colors() {
    echo -e "${BOLD}=== Detailed Color Display ===${RESET}\n"
    
    for ((i=0; i<=255; i++)); do
        show_color "$i"
        
        # Add spacing between logical groups
        case $i in
            15|231|255) echo ;;
        esac
    done
}

# Function to show compact color grid
show_compact_colors() {
    echo -e "${BOLD}Terminal 256-Color Palette${RESET}\n"
    
    # System colors (0-15)
    show_color_range "System Colors (0-15)" 0 15 16
    
    # 6x6x6 color cube (16-231) - show in blocks for easier browsing
    echo -e "\n${BOLD}=== 6×6×6 RGB Color Cube (16-231) ===${RESET}"
    echo "Colors organized in 6×6 blocks by hue families:"
    
    # Show in 4 rows of 54 colors each for better visual grouping
    for ((row=0; row<4; row++)); do
        echo -e "\n${BOLD}Colors $((16 + row*54))-$((16 + (row+1)*54 - 1)):${RESET}"
        local count=0
        for ((i=16 + row*54; i<16 + (row+1)*54 && i<=231; i++)); do
            if ((count % 18 == 0 && count > 0)); then
                echo
            fi
            
            local bg_color="${ESC}[48;5;${i}m"
            if is_theme_color "$i"; then
                printf "${BOLD}${bg_color}%3d${RESET} " "$i"
            else
                printf "${bg_color}%3d${RESET} " "$i"
            fi
            ((count++))
        done
        echo
    done
    
    # Grayscale ramp (232-255)
    show_color_range "Grayscale Ramp (232-255)" 232 255 12
}

# Function to search for specific color
search_color() {
    local search_index=$1
    
    if ((search_index < 0 || search_index > 255)); then
        echo "Error: Color index must be between 0 and 255"
        exit 1
    fi
    
    echo -e "${BOLD}=== Color $search_index Details ===${RESET}\n"
    
    show_color "$search_index"
    
    # Show surrounding colors
    echo -e "\n${BOLD}Nearby Colors:${RESET}"
    local start=$((search_index - 5))
    local end=$((search_index + 5))
    
    [[ $start -lt 0 ]] && start=0
    [[ $end -gt 255 ]] && end=255
    
    for ((i=start; i<=end; i++)); do
        if ((i == search_index)); then
            printf "${BOLD}>>> "
        else
            printf "    "
        fi
        show_color "$i"
    done
}

# Function to show theme colors only
show_theme_colors() {
    echo -e "${BOLD}=== Current Default Theme Colors ===${RESET}\n"
    
    # Show theme colors in order
    local theme_colors=(32 33 69 75 82 121 197 183 202 222 226 227 231 234 254)
    for index in "${theme_colors[@]}"; do
        show_color "$index"
    done
    
    echo -e "\n${BOLD}Color Categories:${RESET}"
    echo "• Mood Colors: 254(Neutral), 202(Happy), 222(Hyperfocused), 227(Searching), 183(Thinking), 197(Error), 82(Success)"
    echo "• Model Colors: 226(Opus), 121(Sonnet), 32(Haiku)"  
    echo "• Special Colors: 33(Git), 75(Documentation)"
    echo "• UI Colors: 69(Activity), 231(Text), 234(Separators)"
}

# Function to show usage
show_help() {
    echo "Terminal 256-Color Visualization Script"
    echo
    echo "Usage:"
    echo "  $0                    Show compact color grid"
    echo "  $0 compact           Show compact color grid"
    echo "  $0 full              Show detailed color list"
    echo "  $0 theme             Show current theme colors only"
    echo "  $0 search N          Show details for color N and nearby colors"
    echo "  $0 help              Show this help"
    echo
    echo "Legend:"
    echo "  • Bold numbers indicate colors used in Default theme"
    echo "  • Color format: [Index] [Background] [Foreground Sample]"
    echo "  • Use Ctrl+C to interrupt long displays"
}

# Main script logic
main() {
    local mode=${1:-compact}
    
    case $mode in
        compact|"")
            show_compact_colors
            echo -e "\n${BOLD}Current Theme Colors:${RESET}"
            show_theme_colors
            ;;
        full|detailed)
            show_detailed_colors
            ;;
        theme)
            show_theme_colors
            ;;
        search)
            if [[ -z ${2:-} ]]; then
                echo "Error: search requires a color index"
                echo "Usage: $0 search N"
                exit 1
            fi
            search_color "$2"
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            echo "Error: Unknown option '$mode'"
            show_help
            exit 1
            ;;
    esac
    
    echo -e "\n${BOLD}Tips:${RESET}"
    echo "• Run '$0 search N' to examine color N in detail"
    echo "• Colors 16-231 form a 6×6×6 RGB cube for systematic color selection"
    echo "• Colors 232-255 are grayscale for neutral tones"
    echo "• Bold numbers show colors currently used in your Default theme"
}

# Run main function with all arguments
main "$@"