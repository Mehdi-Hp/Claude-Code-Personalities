pub mod personality;

use anyhow::Result;
use serde::Deserialize;
use std::io::{self, Read};

use crate::config::PersonalityPreferences;
use crate::icons::{ICON_ERROR, ICON_FOLDER, ICON_WARNING, get_activity_icon, get_model_icon};
use crate::state::SessionState;

#[derive(Debug, Deserialize)]
pub struct ClaudeInput {
    pub session_id: Option<String>,
    pub model: Option<ModelInfo>,
    pub workspace: Option<WorkspaceInfo>,
}

#[derive(Debug, Deserialize)]
pub struct ModelInfo {
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WorkspaceInfo {
    pub current_dir: Option<String>,
    pub project_dir: Option<String>,
}

/// Run the statusline generator, reading JSON from stdin and outputting formatted statusline.
///
/// # Errors
///
/// This function will return an error if:
/// - No input is received from Claude Code via stdin
/// - The input JSON is malformed or cannot be parsed
/// - Session state cannot be loaded from disk
/// - Personality preferences cannot be loaded
pub async fn run_statusline() -> Result<()> {
    use anyhow::Context;
    use colored::control;

    // Force colors to be enabled even when output is piped (Claude Code expects ANSI codes)
    control::set_override(true);

    // Read JSON from stdin
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .with_context(|| "Failed to read input from stdin")?;

    if input.trim().is_empty() {
        return Err(anyhow::anyhow!("No input received from Claude Code"))
            .with_context(|| "Claude Code should pass JSON input via stdin. Check that statusline is configured correctly.");
    }

    let claude_input: ClaudeInput = serde_json::from_str(&input).with_context(|| {
        let preview = if input.len() > 100 {
            format!("{}...", &input[..100])
        } else {
            input.clone()
        };
        format!("Failed to parse JSON input from Claude Code. Received: {preview}")
    })?;

    let session_id = claude_input
        .session_id
        .unwrap_or_else(|| "unknown".to_string());
    let model_name = claude_input
        .model
        .and_then(|m| m.display_name)
        .unwrap_or_else(|| "Claude".to_string());

    // Load session state and preferences
    let state = SessionState::load(&session_id)
        .await
        .with_context(|| format!("Failed to load session state for session '{session_id}'"))?;
    let prefs = PersonalityPreferences::load_or_default()
        .await
        .with_context(|| "Failed to load personality preferences")?;

    // Use static renderer
    let statusline = build_statusline(&state, &model_name, &prefs, claude_input.workspace.as_ref());

    println!("{statusline}");

    Ok(())
}

#[must_use]
pub fn build_statusline(
    state: &SessionState,
    model_name: &str,
    prefs: &PersonalityPreferences,
    workspace: Option<&WorkspaceInfo>,
) -> String {
    let mut parts = Vec::new();

    // Personality (bold)
    if prefs.show_personality {
        let personality_text = if prefs.use_colors {
            prefs.theme.apply_personality(&state.personality)
        } else {
            state.personality.clone()
        };
        parts.push(personality_text);
    }

    // Workspace information
    if prefs.show_current_dir {
        if let Some(workspace) = workspace {
            let workspace_text = format_workspace_info(workspace, prefs);
            if !workspace_text.is_empty() {
                let separator = if prefs.display.show_separators {
                    if prefs.use_colors {
                        prefs.theme.apply_separator("•")
                    } else {
                        "•".to_string()
                    }
                } else {
                    String::new()
                };

                let spacing = if prefs.display.compact_mode { "" } else { " " };

                if parts.is_empty() {
                    parts.push(workspace_text);
                } else if prefs.display.show_separators {
                    parts.push(format!("{spacing}{separator}{spacing}{workspace_text}"));
                } else {
                    parts.push(format!("{spacing}{workspace_text}"));
                }
            }
        }
    }

    // Activity with icon
    if prefs.show_activity {
        let activity_icon = if prefs.use_icons {
            get_activity_icon(&state.activity)
        } else {
            ""
        };

        let mut activity_parts = Vec::new();
        if !activity_icon.is_empty() {
            activity_parts.push(activity_icon.to_string());
        }
        let activity_str = if prefs.use_colors {
            prefs.theme.apply_activity(&state.activity.to_string())
        } else {
            state.activity.to_string()
        };
        activity_parts.push(activity_str);

        // Current job/file
        if prefs.show_current_job {
            if let Some(job) = &state.current_job {
                if !job.is_empty() {
                    let job_text = if prefs.use_colors {
                        prefs.theme.apply_file(job)
                    } else {
                        job.clone()
                    };
                    activity_parts.push(job_text);
                }
            }
        }

        let activity_text = activity_parts.join(" ");
        let separator = if prefs.display.show_separators {
            if prefs.use_colors {
                prefs.theme.apply_separator("•")
            } else {
                "•".to_string()
            }
        } else {
            String::new()
        };

        let spacing = if prefs.display.compact_mode { "" } else { " " };

        if parts.is_empty() {
            parts.push(activity_text);
        } else if prefs.display.show_separators {
            parts.push(format!("{spacing}{separator}{spacing}{activity_text}"));
        } else {
            parts.push(format!("{spacing}{activity_text}"));
        }
    }

    // Error indicators
    if prefs.show_error_indicators {
        if state.error_count >= 3 {
            let error_icon = if prefs.use_colors {
                prefs.theme.apply_error(ICON_ERROR)
            } else {
                ICON_ERROR.to_string()
            };
            parts.push(format!(" {error_icon}"));
        } else if state.error_count > 0 {
            let warning_icon = if prefs.use_colors {
                prefs.theme.apply_warning(ICON_WARNING)
            } else {
                ICON_WARNING.to_string()
            };
            parts.push(format!(" {warning_icon}"));
        }
    }

    // Model indicator
    if prefs.show_model {
        let model_icon = if prefs.use_icons {
            get_model_icon(model_name)
        } else {
            ""
        };

        let model_text = if model_icon.is_empty() {
            format!("[{model_name}]")
        } else {
            format!("[{model_icon} {model_name}]")
        };

        let colored_model = if prefs.use_colors {
            prefs.theme.apply_model_color(&model_text, model_name)
        } else {
            model_text
        };

        let separator = if prefs.display.show_separators {
            if prefs.use_colors {
                prefs.theme.apply_separator("•")
            } else {
                "•".to_string()
            }
        } else {
            String::new()
        };

        let spacing = if prefs.display.compact_mode { "" } else { " " };

        if parts.is_empty() {
            parts.push(colored_model);
        } else if prefs.display.show_separators {
            parts.push(format!("{spacing}{separator}{spacing}{colored_model}"));
        } else {
            parts.push(format!("{spacing}{colored_model}"));
        }
    }

    // Debug info (if enabled)
    if prefs.display.show_debug_info {
        let debug_info = format!(
            "[E:{} C:{} S:{}]",
            state.error_count, state.consecutive_actions, state.session_id
        );

        let separator = if prefs.display.show_separators {
            if prefs.use_colors {
                prefs.theme.apply_separator("•")
            } else {
                "•".to_string()
            }
        } else {
            String::new()
        };

        let spacing = if prefs.display.compact_mode { "" } else { " " };

        let debug_text = if prefs.use_colors {
            prefs.theme.apply_separator(&debug_info)
        } else {
            debug_info
        };

        if parts.is_empty() {
            parts.push(debug_text);
        } else if prefs.display.show_separators {
            parts.push(format!("{spacing}{separator}{spacing}{debug_text}"));
        } else {
            parts.push(format!("{spacing}{debug_text}"));
        }
    }

    parts.join("")
}

/// Format workspace information for display in statusline
fn format_workspace_info(workspace: &WorkspaceInfo, prefs: &PersonalityPreferences) -> String {
    let mut workspace_parts = Vec::new();

    // Add folder icon if using icons
    if prefs.use_icons {
        workspace_parts.push(ICON_FOLDER.to_string());
    }

    // Prefer project name from project_dir, fallback to current_dir
    if let Some(project_dir) = &workspace.project_dir {
        if let Some(project_name) = std::path::Path::new(project_dir).file_name() {
            workspace_parts.push(project_name.to_string_lossy().to_string());
        }
    } else if let Some(current_dir) = &workspace.current_dir
        && let Some(dir_name) = std::path::Path::new(current_dir).file_name()
    {
        workspace_parts.push(dir_name.to_string_lossy().to_string());
    }

    if workspace_parts.is_empty() {
        return String::new();
    }

    let workspace_text = workspace_parts.join(" ");

    if prefs.use_colors {
        prefs.theme.apply_directory(&workspace_text)
    } else {
        workspace_text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::preferences::DisplayConfig;
    use crate::icons::*;
    use crate::state::SessionState;
    use crate::types::Activity;

    fn create_test_state() -> SessionState {
        SessionState {
            session_id: "test".to_string(),
            activity: Activity::Editing,
            current_job: Some("test.js".to_string()),
            personality: "ʕ•ᴥ•ʔ Code Wizard".to_string(),
            previous_personality: None,
            consecutive_actions: 1,
            error_count: 0,
            recent_activities: Vec::new(),
            mood: crate::state::MoodState::default(),
        }
    }

    fn create_test_preferences() -> PersonalityPreferences {
        PersonalityPreferences::default()
    }

    #[test]
    fn test_build_statusline_basic() {
        let state = create_test_state();
        let prefs = create_test_preferences();
        let statusline = build_statusline(&state, "Opus", &prefs, None);

        // Should contain personality (bold formatting is applied but we can't easily test ANSI codes)
        assert!(statusline.contains("ʕ•ᴥ•ʔ Code Wizard"));

        // Should contain activity and job
        assert!(statusline.contains("editing"));
        assert!(statusline.contains("test.js"));

        // Should contain model info
        assert!(statusline.contains("Opus"));

        // Should not contain error indicators for 0 errors
        assert!(!statusline.contains(ICON_ERROR));
        assert!(!statusline.contains(ICON_WARNING));
    }

    #[test]
    fn test_build_statusline_with_errors() {
        let mut state = create_test_state();
        let prefs = create_test_preferences();
        state.error_count = 1;
        let statusline = build_statusline(&state, "Sonnet", &prefs, None);

        // Should contain warning for 1 error
        assert!(statusline.contains(ICON_WARNING));
        assert!(!statusline.contains(ICON_ERROR));

        // Test with many errors
        state.error_count = 5;
        let statusline = build_statusline(&state, "Sonnet", &prefs, None);
        assert!(statusline.contains(ICON_ERROR));
    }

    #[test]
    fn test_build_statusline_no_job() {
        let mut state = create_test_state();
        let prefs = create_test_preferences();
        state.current_job = None;
        let statusline = build_statusline(&state, "Haiku", &prefs, None);

        // Should contain activity but no job
        assert!(statusline.contains("editing"));
        assert!(!statusline.contains("test.js"));
    }

    #[test]
    fn test_build_statusline_empty_job() {
        let mut state = create_test_state();
        let prefs = create_test_preferences();
        state.current_job = Some(String::new());
        let statusline = build_statusline(&state, "Haiku", &prefs, None);

        // Should treat empty job same as no job
        assert!(statusline.contains("editing"));
    }

    #[test]
    fn test_get_activity_icon() {
        // Test all known activity types
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
    }

    #[test]
    fn test_get_model_icon() {
        // Test model-specific icons
        assert_eq!(get_model_icon("Opus"), ICON_OPUS);
        assert_eq!(get_model_icon("Sonnet"), ICON_SONNET);
        assert_eq!(get_model_icon("Haiku"), ICON_HAIKU);
        assert_eq!(get_model_icon("Unknown"), ICON_CLAUDE_DEFAULT);
    }

    #[test]
    fn test_theme_model_colors() {
        use crate::theme::Theme;

        let theme = Theme::default(); // Dark theme

        // Test model color application (should return colored strings, not color names)
        let opus_output = theme.apply_model_color("[ Opus]", "Opus");
        assert!(opus_output.contains("Opus"));

        let sonnet_output = theme.apply_model_color("[ Sonnet]", "Sonnet");
        assert!(sonnet_output.contains("Sonnet"));

        let haiku_output = theme.apply_model_color("[ Haiku]", "Haiku");
        assert!(haiku_output.contains("Haiku"));

        // Test case insensitive matching
        let opus_lower = theme.apply_model_color("[ opus]", "opus");
        assert!(opus_lower.contains("opus"));

        // Test unknown model (should still work)
        let unknown = theme.apply_model_color("[ GPT-4]", "GPT-4");
        assert!(unknown.contains("GPT-4"));
    }

    #[test]
    fn test_claude_input_parsing() {
        let json_str = r#"{
            "session_id": "test_123",
            "model": {
                "display_name": "Opus"
            },
            "workspace": {
                "current_dir": "/path/to/project",
                "project_dir": "/path/to/project"
            }
        }"#;

        let claude_input: ClaudeInput = serde_json::from_str(json_str).unwrap();
        assert_eq!(claude_input.session_id, Some("test_123".to_string()));
        assert_eq!(
            claude_input.model.unwrap().display_name,
            Some("Opus".to_string())
        );
        assert!(claude_input.workspace.is_some());
    }

    #[test]
    fn test_claude_input_parsing_minimal() {
        let json_str = r"{}";
        let claude_input: ClaudeInput = serde_json::from_str(json_str).unwrap();
        assert_eq!(claude_input.session_id, None);
        assert!(claude_input.model.is_none());
        assert!(claude_input.workspace.is_none());
    }

    #[test]
    fn test_statusline_with_different_activities() {
        let mut state = create_test_state();
        let prefs = create_test_preferences();

        // Test different activities produce different icons
        let activities = [
            (Activity::Editing, ICON_EDITING),
            (Activity::Coding, ICON_CODE),
            (Activity::Configuring, ICON_GEAR),
            (Activity::Navigating, ICON_FOLDER),
            (Activity::Reading, ICON_READING),
            (Activity::Testing, ICON_TESTING),
            (Activity::Building, ICON_BUILDING),
            (Activity::Searching, ICON_SEARCHING),
        ];

        for (activity, expected_icon) in activities {
            state.activity = activity.clone();
            let statusline = build_statusline(&state, "Claude", &prefs, None);

            // The icon should be in the statusline (though we can't easily test positioning)
            assert!(statusline.contains(expected_icon));
            assert!(statusline.contains(&activity.to_string()));
        }
    }

    #[test]
    fn test_statusline_formatting_structure() {
        let state = create_test_state();
        let prefs = create_test_preferences();
        let statusline = build_statusline(&state, "TestModel", &prefs, None);

        // Should contain separators
        assert!(statusline.contains("•"));

        // Should contain brackets for model
        assert!(statusline.contains('['));
        assert!(statusline.contains(']'));

        // Should be structured as: personality • activity job • [model]
        // We can't test exact formatting due to ANSI codes, but can check basic structure
        let parts: Vec<&str> = statusline.split("•").collect();
        assert!(parts.len() >= 2); // At least personality and activity parts
    }

    #[test]
    fn test_long_job_names() {
        let mut state = create_test_state();
        let prefs = create_test_preferences();
        state.current_job =
            Some("very_long_filename_that_might_cause_display_issues.js".to_string());

        let statusline = build_statusline(&state, "Claude", &prefs, None);

        // Should handle long job names gracefully
        assert!(statusline.contains("very_long_filename_that_might_cause_display_issues.js"));
        assert!(statusline.len() > 50); // Should be a substantial statusline
    }

    #[test]
    fn test_different_personalities() {
        let mut state = create_test_state();
        let prefs = create_test_preferences();
        let personalities = [
            "ʕ•ᴥ•ʔ Code Wizard",
            "(╯°□°)╯︵ ┻━┻ Table Flipper",
            "┗(▀̿Ĺ̯▀̿ ̿)┓ Git Manager",
            "φ(．．) Documentation Writer",
        ];

        for personality in personalities {
            state.personality = personality.to_string();
            let statusline = build_statusline(&state, "Claude", &prefs, None);
            assert!(statusline.contains(personality));
        }
    }

    #[test]
    fn test_error_states() {
        let mut state = create_test_state();
        let prefs = create_test_preferences();

        // No errors
        state.error_count = 0;
        let statusline = build_statusline(&state, "Claude", &prefs, None);
        assert!(!statusline.contains(ICON_ERROR));
        assert!(!statusline.contains(ICON_WARNING));

        // Low errors (warning)
        state.error_count = 1;
        let statusline = build_statusline(&state, "Claude", &prefs, None);
        assert!(!statusline.contains(ICON_ERROR));
        assert!(statusline.contains(ICON_WARNING));

        state.error_count = 2;
        let statusline = build_statusline(&state, "Claude", &prefs, None);
        assert!(!statusline.contains(ICON_ERROR));
        assert!(statusline.contains(ICON_WARNING));

        // High errors (error icon)
        state.error_count = 3;
        let statusline = build_statusline(&state, "Claude", &prefs, None);
        assert!(statusline.contains(ICON_ERROR));
        assert!(!statusline.contains(ICON_WARNING));

        state.error_count = 10;
        let statusline = build_statusline(&state, "Claude", &prefs, None);
        assert!(statusline.contains(ICON_ERROR));
        assert!(!statusline.contains(ICON_WARNING));
    }

    #[test]
    fn test_display_configuration_options() {
        use crate::types::Activity;
        let state = SessionState {
            activity: Activity::Editing,
            current_job: Some("test.js".to_string()),
            error_count: 2,
            ..Default::default()
        };

        // Test with all options enabled (default)
        let prefs_all = PersonalityPreferences::default();
        let statusline_all = build_statusline(&state, "Opus", &prefs_all, None);

        // Should contain personality, activity, model, and separators
        assert!(statusline_all.contains("Booting Up"));
        assert!(statusline_all.contains("editing"));
        assert!(statusline_all.contains("test.js"));
        assert!(statusline_all.contains("Opus"));
        assert!(statusline_all.contains("•")); // Separator

        // Test with minimal configuration
        let prefs_minimal = PersonalityPreferences {
            show_personality: false,
            show_activity: false,
            show_model: false,
            show_error_indicators: false, // Also disable error indicators
            use_icons: false,
            use_colors: false,
            display: DisplayConfig {
                show_separators: false,
                ..Default::default()
            },
            ..Default::default()
        };

        let statusline_minimal = build_statusline(&state, "Opus", &prefs_minimal, None);
        // Should be empty since we disabled everything important
        assert!(statusline_minimal.is_empty());
    }

    #[test]
    fn test_compact_mode() {
        use crate::types::Activity;
        let state = SessionState {
            activity: Activity::Editing,
            current_job: Some("test.js".to_string()),
            ..Default::default()
        };

        // Normal mode
        let prefs_normal = PersonalityPreferences {
            use_colors: false, // Disable colors for easier testing
            ..Default::default()
        };
        let statusline_normal = build_statusline(&state, "Opus", &prefs_normal, None);

        // Compact mode
        let prefs_compact = PersonalityPreferences {
            use_colors: false, // Disable colors for easier testing
            display: DisplayConfig {
                compact_mode: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let statusline_compact = build_statusline(&state, "Opus", &prefs_compact, None);

        // Compact mode should be shorter due to less spacing
        assert!(statusline_compact.len() < statusline_normal.len());
    }

    #[test]
    fn test_debug_info_display() {
        use crate::types::Activity;
        let state = SessionState {
            activity: Activity::Testing,
            error_count: 3,
            consecutive_actions: 7,
            session_id: "test123".to_string(),
            ..Default::default()
        };

        // Without debug info
        let prefs_normal = PersonalityPreferences::default();
        let statusline_normal = build_statusline(&state, "Sonnet", &prefs_normal, None);
        assert!(!statusline_normal.contains("E:3"));
        assert!(!statusline_normal.contains("C:7"));
        assert!(!statusline_normal.contains("S:test123"));

        // With debug info
        let prefs_debug = PersonalityPreferences {
            display: DisplayConfig {
                show_debug_info: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let statusline_debug = build_statusline(&state, "Sonnet", &prefs_debug, None);
        assert!(statusline_debug.contains("E:3"));
        assert!(statusline_debug.contains("C:7"));
        assert!(statusline_debug.contains("S:test123"));
    }

    #[test]
    fn test_separators_configuration() {
        use crate::types::Activity;
        let state = SessionState {
            activity: Activity::Coding,
            current_job: Some("app.rs".to_string()),
            ..Default::default()
        };

        // With separators (default)
        let prefs_with_sep = PersonalityPreferences {
            use_colors: false,
            ..Default::default()
        };
        let statusline_with_sep = build_statusline(&state, "Haiku", &prefs_with_sep, None);
        assert!(statusline_with_sep.contains("•"));

        // Without separators
        let prefs_no_sep = PersonalityPreferences {
            use_colors: false,
            display: DisplayConfig {
                show_separators: false,
                ..Default::default()
            },
            ..Default::default()
        };
        let statusline_no_sep = build_statusline(&state, "Haiku", &prefs_no_sep, None);
        assert!(!statusline_no_sep.contains("•"));
        // But should still have content
        assert!(statusline_no_sep.contains("Booting Up"));
        assert!(statusline_no_sep.contains("coding"));
        assert!(statusline_no_sep.contains("app.rs"));
        assert!(statusline_no_sep.contains("Haiku"));
    }
}
