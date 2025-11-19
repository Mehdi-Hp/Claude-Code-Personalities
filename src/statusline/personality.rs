//! Personality determination logic
//!
//! This module determines Claude's personality based on current activity,
//! mood state, file types, and usage patterns.

use crate::kaomoji::{
    get_default_tool_kaomoji, get_file_kaomoji, get_mood_kaomoji, get_pattern_kaomoji,
    get_time_kaomoji, get_tool_kaomoji,
};
use crate::state::{PersonalityModifier, SessionState};

/// Determine the appropriate personality based on context and state
pub fn determine_personality(
    state: &SessionState,
    tool_name: &str,
    file_path: Option<&str>,
    command: Option<&str>,
) -> String {
    // Check for frustrated mood first (highest priority)
    if let PersonalityModifier::Frustrated = state.mood.get_personality_modifier() {
        let kaomoji = get_mood_kaomoji(
            &state.mood.get_personality_modifier(),
            state.mood.frustration_level,
        );
        return kaomoji.personality();
    }

    // Check tool-specific personalities (only high-priority ones like Bash git commands, Grep)
    if let Some(kaomoji) = get_tool_kaomoji(tool_name, command) {
        return kaomoji.personality();
    }

    // Check file-type specific personalities
    if let Some(kaomoji) = get_file_kaomoji(file_path.unwrap_or("")) {
        return kaomoji.personality();
    }

    // Check consecutive action patterns (including extreme cases like Code Berserker)
    if let Some(kaomoji) = get_pattern_kaomoji(state.consecutive_actions) {
        return kaomoji.personality();
    }

    // Check for time-based personalities (before defaults)
    if let Some(kaomoji) = get_time_kaomoji() {
        return kaomoji.personality();
    }

    // Check for InTheZone mood (lower priority than pattern personalities)
    if let PersonalityModifier::InTheZone = state.mood.get_personality_modifier() {
        let kaomoji = get_mood_kaomoji(
            &state.mood.get_personality_modifier(),
            state.mood.frustration_level,
        );
        return kaomoji.personality();
    }

    // Default tool personalities (lowest priority)
    let kaomoji = get_default_tool_kaomoji(tool_name, state.consecutive_actions);
    kaomoji.personality()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Activity;

    // Helper function to create test state
    fn create_test_state(error_count: u32, consecutive_actions: u32) -> SessionState {
        // Convert old error_count to new mood system
        let mut mood = crate::state::MoodState::default();

        // Simulate errors to get the right frustration level
        for _ in 0..error_count {
            mood.update(true);
        }

        // If we have consecutive actions, simulate successes to build momentum
        for _ in 0..consecutive_actions {
            mood.update(false);
        }

        SessionState {
            session_id: "test".to_string(),
            activity: Activity::Idle,
            current_job: None,
            current_file: None,
            git_branch: None,
            git_dirty: None,
            git_dirty_count: None,
            git_status_checked_at: None,
            personality: "Test".to_string(),
            previous_personality: None,
            consecutive_actions,
            error_count,
            recent_activities: Vec::new(),
            mood,
        }
    }

    #[test]
    fn test_frustration_states() {
        let state = create_test_state(5, 0);
        let personality = determine_personality(&state, "Edit", None, None);
        assert_eq!(personality, "(╯°□°)╯︵ ┻━┻ Table Flipper");

        let state = create_test_state(3, 0);
        let personality = determine_personality(&state, "Edit", None, None);
        assert_eq!(personality, "(ノಠ益ಠ)ノ Error Warrior");

        let state = create_test_state(2, 0);
        let personality = determine_personality(&state, "Edit", None, None);
        assert_eq!(personality, "ლ(╹◡╹ლ) Cowder");
    }

    #[test]
    fn test_git_operations() {
        let state = create_test_state(0, 0);

        let personality = determine_personality(&state, "Bash", None, Some("git status"));
        assert_eq!(personality, "┗(▀̿Ĺ̯▀̿ ̿)┓ Git Manager");

        let personality = determine_personality(&state, "Bash", None, Some("git commit -m 'test'"));
        assert_eq!(personality, "┗(▀̿Ĺ̯▀̿ ̿)┓ Git Manager");

        let personality = determine_personality(&state, "Bash", None, Some("ls -la"));
        assert_ne!(personality, "┗(▀̿Ĺ̯▀̿ ̿)┓ Git Manager");
    }

    #[test]
    fn test_file_type_personalities() {
        let state = create_test_state(0, 0);

        // JavaScript files
        let personality = determine_personality(&state, "Edit", Some("test.js"), None);
        assert_eq!(personality, "(▀̿Ĺ̯▀̿ ̿) JS Master");

        let personality = determine_personality(&state, "Edit", Some("component.tsx"), None);
        assert_eq!(personality, "(✿◠ᴗ◠) UI Developer");

        // Documentation
        let personality = determine_personality(&state, "Edit", Some("README.md"), None);
        assert_eq!(personality, "φ(．．) Documentation Writer");

        // Auth files
        let personality = determine_personality(&state, "Edit", Some("auth.js"), None);
        assert_eq!(personality, "ಠ_ಠ Security Analyst");
    }

    #[test]
    fn test_consecutive_actions() {
        // Hyperfocused coder
        let state = create_test_state(0, 15);
        let personality = determine_personality(&state, "Edit", None, None);
        assert_eq!(personality, "┌༼◉ل͟◉༽┐ Hyperfocused Coder");

        // Code berserker
        let state = create_test_state(0, 25);
        let personality = determine_personality(&state, "Edit", None, None);
        assert_eq!(personality, "【╯°□°】╯︵ ┻━┻ Code Berserker");
    }

    #[test]
    fn test_grep_debugging() {
        let state = create_test_state(0, 0);
        let personality = determine_personality(&state, "Grep", None, None);
        assert_eq!(personality, "(つ◉益◉)つ Bug Hunter");
    }

    #[test]
    fn test_priority_order() {
        // Error states should override everything
        let state = create_test_state(5, 0);
        let personality = determine_personality(&state, "Bash", None, Some("git status"));
        assert_eq!(personality, "(╯°□°)╯︵ ┻━┻ Table Flipper");

        // Git should override file types
        let state = create_test_state(0, 0);
        let personality =
            determine_personality(&state, "Bash", Some("test.js"), Some("git status"));
        assert_eq!(personality, "┗(▀̿Ĺ̯▀̿ ̿)┓ Git Manager");

        // File types should override tool defaults
        let state = create_test_state(0, 0);
        let personality = determine_personality(&state, "Edit", Some("README.md"), None);
        assert_eq!(personality, "φ(．．) Documentation Writer");

        // Pattern should override time (not easily testable with system time dependency, but conceptually true)
    }

    #[test]
    fn test_time_based_fallback() {
        // This test is a bit tricky because it depends on the system time.
        // However, we can verify that if no other personality matches, we fall back to EITHER
        // a time-based personality OR the default personality.
        
        let state = create_test_state(0, 0);
        // Using a tool that doesn't have a specific high-priority personality
        let personality = determine_personality(&state, "OtherTool", None, None);
        
        // It should be one of the time-based ones OR the default "Booting up..."
        // or specific tool default if "OtherTool" maps to something (it maps to Booting Up in default_tool_kaomoji)
        
        let possible_personalities = vec![
            "(ʘ,ʘ) Night Owl",
            "( -_-)旦~ Caffeinated",
            "ヽ(⌐■_■)ノ♪♬ TGIFFFFF",
            "( ˘ ³˘) Chillin",
        ];
        
        assert!(possible_personalities.contains(&personality.as_str()));
    }
}
