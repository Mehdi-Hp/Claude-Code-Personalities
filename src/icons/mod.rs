//! Centralized Nerd Font icon management module
//!
//! This module provides a single source of truth for all Nerd Font icons used
//! throughout the application, organized by category and purpose.

use crate::types::Activity;

// Re-export all icon categories
pub mod activity;
pub mod models;
pub mod status;
pub mod ui;

// Re-export commonly used icons
pub use activity::*;
pub use models::*;
pub use status::*;
pub use ui::*;

/// Get the appropriate icon for an activity (only for Executing, Reading, and Idle)
pub fn get_activity_icon(activity: &Activity) -> &'static str {
    match activity {
        Activity::Executing => ICON_EXECUTING,
        Activity::Reading => ICON_READING,
        Activity::Idle => ICON_IDLE,
        // All other activities show no icon to reduce visual clutter
        _ => "",
    }
}

/// Get the appropriate icon for a model
pub fn get_model_icon(model_name: &str) -> &'static str {
    match model_name.to_lowercase().as_str() {
        name if name.contains("opus") => ICON_OPUS,
        name if name.contains("sonnet") => ICON_SONNET,
        name if name.contains("haiku") => ICON_HAIKU,
        _ => ICON_CLAUDE_DEFAULT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_activity_icon() {
        // Activities that should have icons
        assert_eq!(get_activity_icon(&Activity::Executing), ICON_EXECUTING);
        assert_eq!(get_activity_icon(&Activity::Reading), ICON_READING);
        assert_eq!(get_activity_icon(&Activity::Idle), ICON_IDLE);

        // Activities that should have no icon (empty string)
        assert_eq!(get_activity_icon(&Activity::Editing), "");
        assert_eq!(get_activity_icon(&Activity::Coding), "");
        assert_eq!(get_activity_icon(&Activity::Configuring), "");
        assert_eq!(get_activity_icon(&Activity::Navigating), "");
        assert_eq!(get_activity_icon(&Activity::Writing), "");
        assert_eq!(get_activity_icon(&Activity::Searching), "");
        assert_eq!(get_activity_icon(&Activity::Debugging), "");
        assert_eq!(get_activity_icon(&Activity::Testing), "");
        assert_eq!(get_activity_icon(&Activity::Reviewing), "");
        assert_eq!(get_activity_icon(&Activity::Thinking), "");
        assert_eq!(get_activity_icon(&Activity::Building), "");
        assert_eq!(get_activity_icon(&Activity::Installing), "");
        assert_eq!(get_activity_icon(&Activity::Working), "");
        assert_eq!(get_activity_icon(&Activity::Refactoring), "");
        assert_eq!(get_activity_icon(&Activity::Documenting), "");
        assert_eq!(get_activity_icon(&Activity::Deploying), "");
    }

    #[test]
    fn test_get_model_icon() {
        assert_eq!(get_model_icon("Opus"), ICON_OPUS);
        assert_eq!(get_model_icon("opus"), ICON_OPUS);
        assert_eq!(get_model_icon("Claude-3-Opus"), ICON_OPUS);

        assert_eq!(get_model_icon("Sonnet"), ICON_SONNET);
        assert_eq!(get_model_icon("sonnet"), ICON_SONNET);
        assert_eq!(get_model_icon("Claude-3.5-Sonnet"), ICON_SONNET);

        assert_eq!(get_model_icon("Haiku"), ICON_HAIKU);
        assert_eq!(get_model_icon("haiku"), ICON_HAIKU);
        assert_eq!(get_model_icon("Claude-3-Haiku"), ICON_HAIKU);

        assert_eq!(get_model_icon("Unknown"), ICON_CLAUDE_DEFAULT);
        assert_eq!(get_model_icon("GPT-4"), ICON_CLAUDE_DEFAULT);
    }
}
