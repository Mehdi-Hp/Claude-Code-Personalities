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

/// Get the appropriate icon for an activity
pub fn get_activity_icon(activity: &Activity) -> &'static str {
    match activity {
        Activity::Editing => ICON_EDITING,
        Activity::Coding => ICON_CODE,
        Activity::Configuring => ICON_GEAR,
        Activity::Navigating => ICON_FOLDER,
        Activity::Writing => ICON_WRITING,
        Activity::Executing => ICON_EXECUTING,
        Activity::Reading => ICON_READING,
        Activity::Searching => ICON_SEARCHING,
        Activity::Debugging => ICON_DEBUGGING,
        Activity::Testing => ICON_TESTING,
        Activity::Reviewing => ICON_REVIEWING,
        Activity::Thinking => ICON_THINKING,
        Activity::Building => ICON_BUILDING,
        Activity::Installing => ICON_INSTALLING,
        Activity::Idle => ICON_IDLE,
        Activity::Working => ICON_WORKING,
        Activity::Refactoring => ICON_REFACTORING,
        Activity::Documenting => ICON_DOCUMENTING,
        Activity::Deploying => ICON_DEPLOYING,
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

/// Error level for status icon selection
#[derive(Debug, Clone, PartialEq)]
pub enum StatusLevel {
    Success,
    Info,
    Warning,
    Error,
}

/// Get the appropriate status icon based on level
pub fn get_status_icon(level: StatusLevel) -> &'static str {
    match level {
        StatusLevel::Success => ICON_CHECK,
        StatusLevel::Info => ICON_INFO,
        StatusLevel::Warning => ICON_WARNING,
        StatusLevel::Error => ICON_ERROR,
    }
}

/// Convert a Unicode icon to printf format for shell scripts
/// e.g., "\u{f044}" -> "\xef\x81\x84"
pub fn icon_to_printf_format(icon: &str) -> String {
    // The icon is a Unicode string, so we can get its UTF-8 bytes directly
    let utf8_bytes = icon.as_bytes();
    let hex_bytes: Vec<String> = utf8_bytes.iter().map(|b| format!("\\x{:02x}", b)).collect();
    hex_bytes.join("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_activity_icon() {
        assert_eq!(get_activity_icon(&Activity::Editing), ICON_EDITING);
        assert_eq!(get_activity_icon(&Activity::Coding), ICON_CODE);
        assert_eq!(get_activity_icon(&Activity::Configuring), ICON_GEAR);
        assert_eq!(get_activity_icon(&Activity::Navigating), ICON_FOLDER);
        assert_eq!(get_activity_icon(&Activity::Writing), ICON_WRITING);
        assert_eq!(get_activity_icon(&Activity::Executing), ICON_EXECUTING);
        assert_eq!(get_activity_icon(&Activity::Reading), ICON_READING);
        assert_eq!(get_activity_icon(&Activity::Searching), ICON_SEARCHING);
        assert_eq!(get_activity_icon(&Activity::Debugging), ICON_DEBUGGING);
        assert_eq!(get_activity_icon(&Activity::Testing), ICON_TESTING);
        assert_eq!(get_activity_icon(&Activity::Reviewing), ICON_REVIEWING);
        assert_eq!(get_activity_icon(&Activity::Thinking), ICON_THINKING);
        assert_eq!(get_activity_icon(&Activity::Building), ICON_BUILDING);
        assert_eq!(get_activity_icon(&Activity::Installing), ICON_INSTALLING);
        assert_eq!(get_activity_icon(&Activity::Idle), ICON_IDLE);
        assert_eq!(get_activity_icon(&Activity::Working), ICON_WORKING);
        assert_eq!(get_activity_icon(&Activity::Refactoring), ICON_REFACTORING);
        assert_eq!(get_activity_icon(&Activity::Documenting), ICON_DOCUMENTING);
        assert_eq!(get_activity_icon(&Activity::Deploying), ICON_DEPLOYING);
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

    #[test]
    fn test_get_status_icon() {
        assert_eq!(get_status_icon(StatusLevel::Success), ICON_CHECK);
        assert_eq!(get_status_icon(StatusLevel::Info), ICON_INFO);
        assert_eq!(get_status_icon(StatusLevel::Warning), ICON_WARNING);
        assert_eq!(get_status_icon(StatusLevel::Error), ICON_ERROR);
    }

    #[test]
    fn test_icon_to_printf_format() {
        // Test the pencil icon (f044)
        let printf_format = icon_to_printf_format("\u{f044}");
        assert_eq!(printf_format, "\\xef\\x81\\x84");

        // Test the folder icon (f07b)
        let printf_format = icon_to_printf_format("\u{f07b}");
        assert_eq!(printf_format, "\\xef\\x81\\xbb");
    }
}
