/// Context-aware color selection for the Default theme
///
/// This module handles dynamic color assignment based on Claude's current state,
/// including personality mood, model type, and activity context.
use crate::theme::colors::Color;

/// Map personality names to their mood-based color indices for Default theme
pub fn get_personality_color_256(personality: &str) -> u8 {
    // Special cases first (highest priority)
    if personality.contains("Git Manager") {
        return 33; // Dark cyan
    }
    if personality.contains("Documentation Writer") {
        return 75; // Light blue
    }

    // Error/Frustrated personalities (red)
    if personality.contains("Table Flipper")
        || personality.contains("Error Warrior")
        || personality.contains("Compilation Warrior")
        || personality.contains("Security Analyst")
        || personality.contains("Permission Police")
        || personality.contains("Code Janitor")
        || personality.contains("Task Assassin")
        || personality.contains("Deployment Guard")
    {
        return 124; // Red
    }

    // Searching/Detective personalities (light yellow)
    if personality.contains("Bug Hunter")
        || personality.contains("Search Maestro")
        || personality.contains("System Detective")
        || personality.contains("Code Historian")
        || personality.contains("Research King")
    {
        return 227; // Light yellow
    }

    // Thinking/Processing personalities (purple)
    if personality.contains("Gentle Refactorer")
        || personality.contains("Quality Auditor")
        || personality.contains("Test Taskmaster")
        || personality.contains("Dependency Wrangler")
        || personality.contains("Environment Enchanter")
        || personality.contains("Compression Chef")
        || personality.contains("String Surgeon")
    {
        return 139; // Purple
    }

    // Happy/Success personalities (orange)
    if personality.contains("Code Wizard")
        || personality.contains("JS Master")
        || personality.contains("UI Developer")
        || personality.contains("Markup Wizard")
        || personality.contains("Style Artist")
        || personality.contains("Performance Tuner")
        || personality.contains("Container Captain")
        || personality.contains("Database Expert")
        || personality.contains("Network Sentinel")
        || personality.contains("Hyperfocused")
        || personality.contains("Berserker")
    {
        return 202; // Orange
    }

    // Success/Complete personalities (bright green)
    if personality.contains("Success") || personality.contains("Complete") {
        return 82; // Bright green
    }

    // Base/Neutral personalities (very light gray) - default
    253
}

/// Create a context-aware color for personality based on mood
pub fn get_context_aware_personality_color(personality: &str) -> Color {
    let color_index = get_personality_color_256(personality);
    Color::from_terminal_256(color_index)
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
        253 // Default to neutral color
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
        // Test special cases
        assert_eq!(get_personality_color_256("┗(▀̿Ĺ̯▀̿ ̿)┓ Git Manager"), 33);
        assert_eq!(
            get_personality_color_256("φ(．．) Documentation Writer"),
            75
        );

        // Test error personalities
        assert_eq!(
            get_personality_color_256("(╯°□°)╯︵ ┻━┻ Table Flipper"),
            124
        );
        assert_eq!(get_personality_color_256("(ノಠ益ಠ)ノ Error Warrior"), 124);

        // Test searching personalities
        assert_eq!(get_personality_color_256("(つ◉益◉)つ Bug Hunter"), 227);
        assert_eq!(get_personality_color_256("Search Maestro"), 227);

        // Test happy personalities
        assert_eq!(get_personality_color_256("ʕ•ᴥ•ʔ Code Wizard"), 202);
        assert_eq!(get_personality_color_256("JS Master"), 202);

        // Test default (base/neutral)
        assert_eq!(get_personality_color_256("Booting Up"), 253);
        assert_eq!(get_personality_color_256("Editor User"), 253);
    }

    #[test]
    fn test_model_color_mapping() {
        assert_eq!(get_model_color_256("Opus"), 226);
        assert_eq!(get_model_color_256("Claude-3-Opus"), 226);
        assert_eq!(get_model_color_256("Sonnet"), 121);
        assert_eq!(get_model_color_256("Claude-3.5-Sonnet"), 121);
        assert_eq!(get_model_color_256("Haiku"), 32);
        assert_eq!(get_model_color_256("Claude-3-Haiku"), 32);
        assert_eq!(get_model_color_256("GPT-4"), 253); // Unknown model
    }
}
