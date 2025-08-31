/// Simplified color selection for the Default theme
///
/// This module provides consistent coloring with a single color per section.
use crate::theme::colors::Color;

/// Get color for personalities in Default theme - always returns 231 (bright white)
pub fn get_personality_color_256(_personality: &str) -> u8 {
    231 // Consistent bright white for all personalities
}

/// Create a consistent color for personality (always 231 - bright white)
pub fn get_context_aware_personality_color(_personality: &str) -> Color {
    Color::from_terminal_256(231)
}

/// Get model-specific color for the Default theme
pub fn get_model_color_256(model_name: &str) -> u8 {
    if model_name.to_lowercase().contains("opus") {
        226 // Yellow
    } else if model_name.to_lowercase().contains("sonnet") {
        121 // Light purple
    } else if model_name.to_lowercase().contains("haiku") {
        32 // Teal
    } else {
        254 // Default to neutral color
    }
}

/// Create a context-aware color for model indicator
pub fn get_context_aware_model_color(model_name: &str) -> Color {
    let color_index = get_model_color_256(model_name);
    Color::from_terminal_256(color_index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_personality_color_mapping() {
        // All personalities now return the same color (231 - bright white)
        assert_eq!(get_personality_color_256("┗(▀̿Ĺ̯▀̿ ̿)┓ Git Manager"), 231);
        assert_eq!(
            get_personality_color_256("φ(．．) Documentation Writer"),
            231
        );
        assert_eq!(
            get_personality_color_256("(╯°□°)╯︵ ┻━┻ Table Flipper"),
            231
        );
        assert_eq!(get_personality_color_256("(ノಠ益ಠ)ノ Error Warrior"), 231);
        assert_eq!(get_personality_color_256("(つ◉益◉)つ Bug Hunter"), 231);
        assert_eq!(get_personality_color_256("Search Maestro"), 231);
        assert_eq!(get_personality_color_256("ლ(╹◡╹ლ) Cowder"), 231);
        assert_eq!(get_personality_color_256("JS Master"), 231);
        assert_eq!(get_personality_color_256("Chillin"), 231);
        assert_eq!(get_personality_color_256("Editor User"), 231);
    }

    #[test]
    fn test_model_color_mapping() {
        assert_eq!(get_model_color_256("Opus"), 226);
        assert_eq!(get_model_color_256("Claude-3-Opus"), 226);
        assert_eq!(get_model_color_256("Sonnet"), 121);
        assert_eq!(get_model_color_256("Claude-3.5-Sonnet"), 121);
        assert_eq!(get_model_color_256("Haiku"), 32);
        assert_eq!(get_model_color_256("Claude-3-Haiku"), 32);
        assert_eq!(get_model_color_256("GPT-4"), 254); // Unknown model
    }
}
