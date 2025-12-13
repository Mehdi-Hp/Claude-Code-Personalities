pub mod personality;

use anyhow::Result;
use colored::Colorize;
use serde::Deserialize;
use std::io::{self, Read};

use crate::config::{PersonalityPreferences, StatuslineSection};
use crate::icons::{ICON_FOLDER, ICON_GIT_BRANCH, get_activity_icon, get_model_icon};
use crate::state::SessionState;
use crate::version::VersionManager;

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

    // Use a consistent fallback when session_id is missing
    let session_id = claude_input.session_id.unwrap_or_else(|| {
        // Use a predictable session ID for the current Claude session
        std::env::var("CLAUDE_SESSION_ID").unwrap_or_else(|_| "claude_current".to_string())
    });
    let model_name = claude_input
        .model
        .and_then(|m| m.display_name)
        .unwrap_or_else(|| "Claude".to_string());

    // Load session state and preferences
    let mut state = SessionState::load(&session_id)
        .await
        .with_context(|| format!("Failed to load session state for session '{session_id}'"))?;
    let prefs = PersonalityPreferences::load_or_default()
        .await
        .with_context(|| "Failed to load personality preferences")?;

    // Get current directory from workspace for git operations
    let current_dir = claude_input
        .workspace
        .as_ref()
        .and_then(|w| w.current_dir.as_deref());

    // Refresh git branch if enabled (with caching to avoid performance overhead)
    // This runs git commands in the correct project directory
    if prefs.show_git && prefs.show_git_branch {
        if let Some(dir) = current_dir {
            state.refresh_git_branch(dir).await;
        }
    }

    // Refresh git status if enabled (with caching to avoid performance overhead)
    if prefs.show_git && prefs.show_git_status {
        if let Some(dir) = current_dir {
            state.refresh_git_status_in_dir(dir).await;
        } else {
            state.refresh_git_status().await;
        }
    }

    // Check for updates if enabled (uses cache to avoid API rate limits)
    let update_available = if prefs.show_update_available {
        if let Ok(version_manager) = VersionManager::new() {
            version_manager
                .check_for_update()
                .await
                .ok()
                .flatten()
                .map(|release| {
                    release
                        .tag_name
                        .strip_prefix('v')
                        .unwrap_or(&release.tag_name)
                        .to_string()
                })
        } else {
            None
        }
    } else {
        None
    };

    // Use static renderer
    let statusline = build_statusline(
        &state,
        &model_name,
        &prefs,
        claude_input.workspace.as_ref(),
        update_available.as_deref(),
    );

    print!("{statusline}");

    Ok(())
}

#[must_use]
pub fn build_statusline(
    state: &SessionState,
    model_name: &str,
    prefs: &PersonalityPreferences,
    workspace: Option<&WorkspaceInfo>,
    update_available: Option<&str>,
) -> String {
    let mut parts = Vec::new();

    // Iterate over section order from preferences
    for section in &prefs.section_order {
        let section_text = match section {
            StatuslineSection::Personality => render_personality_section(state, prefs),
            StatuslineSection::Directory => render_directory_section(workspace, prefs),
            StatuslineSection::Git => render_git_section(state, prefs),
            StatuslineSection::Activity => render_activity_section(state, prefs),
            StatuslineSection::Model => render_model_section(model_name, prefs),
            StatuslineSection::UpdateAvailable => render_update_section(update_available, prefs),
            StatuslineSection::DebugInfo => render_debug_section(state, prefs),
        };

        add_section_to_parts(&mut parts, section_text, prefs);
    }

    parts.join("")
}

/// Format workspace information for display in statusline
fn format_workspace_info(workspace: &WorkspaceInfo, prefs: &PersonalityPreferences) -> String {
    let mut workspace_parts = Vec::new();

    // Add folder icon if using icons
    if prefs.show_directory_icon {
        workspace_parts.push(ICON_FOLDER.to_string());
    }

    // Only add directory name if label is enabled
    if prefs.show_directory_label {
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

/// Render the personality section
fn render_personality_section(
    state: &SessionState,
    prefs: &PersonalityPreferences,
) -> Option<String> {
    if !prefs.show_personality {
        return None;
    }
    let personality_text = if prefs.use_colors {
        prefs
            .theme
            .apply_personality_with_context(&state.personality, state)
    } else {
        state.personality.clone()
    };
    Some(personality_text)
}

/// Render the directory/workspace section
fn render_directory_section(
    workspace: Option<&WorkspaceInfo>,
    prefs: &PersonalityPreferences,
) -> Option<String> {
    if !prefs.show_current_dir {
        return None;
    }
    let workspace = workspace?;
    let workspace_text = format_workspace_info(workspace, prefs);
    if workspace_text.is_empty() {
        return None;
    }
    Some(workspace_text)
}

/// Render the git branch section
fn render_git_section(state: &SessionState, prefs: &PersonalityPreferences) -> Option<String> {
    if !prefs.show_git || !prefs.show_git_branch {
        return None;
    }
    let branch = state.git_branch.as_ref()?;
    if branch.is_empty() {
        return None;
    }

    // Build git text piece by piece: icon + label + status
    let mut git_parts = Vec::new();

    // Icon
    if prefs.show_git_icon {
        git_parts.push(ICON_GIT_BRANCH.to_string());
    }

    // Branch name
    let branch_display = if !branch.contains('/') {
        format!("{} branch", branch)
    } else {
        branch.clone()
    };
    git_parts.push(branch_display);

    // Combine icon and branch name
    let base_text = git_parts.join(" ");

    // Build final text with colors and status
    let branch_text = if prefs.use_colors {
        let base_colored = if !branch.contains('/') {
            let branch_part = if prefs.show_git_icon {
                format!("{} {}", ICON_GIT_BRANCH, branch)
            } else {
                branch.clone()
            };
            format!(
                "{}{}",
                prefs.theme.apply_file(&branch_part),
                " branch".dimmed()
            )
        } else {
            prefs.theme.apply_file(&base_text)
        };

        // Add git status indicator if enabled
        if prefs.show_git_status {
            if let Some(is_dirty) = state.git_dirty {
                let status_text = if is_dirty {
                    let count = state.git_dirty_count.unwrap_or(0);
                    if count > 0 {
                        prefs.theme.apply_warning(&format!(" ±{}", count))
                    } else {
                        prefs.theme.apply_warning(" ±")
                    }
                } else {
                    prefs.theme.apply_success(" ✓")
                };
                format!("{base_colored}{status_text}")
            } else {
                base_colored
            }
        } else {
            base_colored
        }
    } else {
        // No colors - simple concatenation
        if prefs.show_git_status {
            if let Some(is_dirty) = state.git_dirty {
                let status_text = if is_dirty {
                    let count = state.git_dirty_count.unwrap_or(0);
                    if count > 0 {
                        format!(" ±{}", count)
                    } else {
                        " ±".to_string()
                    }
                } else {
                    " ✓".to_string()
                };
                format!("{base_text}{status_text}")
            } else {
                base_text
            }
        } else {
            base_text
        }
    };

    if branch_text.is_empty() {
        return None;
    }
    Some(branch_text)
}

/// Render the activity section
fn render_activity_section(state: &SessionState, prefs: &PersonalityPreferences) -> Option<String> {
    if !prefs.show_activity {
        return None;
    }

    let activity_icon = if prefs.show_activity_icon {
        get_activity_icon(&state.activity)
    } else {
        ""
    };

    let mut activity_parts = Vec::new();
    if !activity_icon.is_empty() {
        let colored_icon = if prefs.use_colors {
            prefs.theme.apply_activity(activity_icon)
        } else {
            activity_icon.to_string()
        };
        activity_parts.push(colored_icon);
    }

    // Only show activity label text if enabled
    if prefs.show_activity_label {
        let activity_str = if prefs.use_colors {
            prefs.theme.apply_activity(&state.activity.to_string())
        } else {
            state.activity.to_string()
        };
        activity_parts.push(activity_str);
    }

    // Context (shows command name OR file depending on activity)
    if prefs.show_context {
        // Check for command name first (for bash operations)
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
        // Otherwise check for file (for edit/read operations)
        else if let Some(file) = &state.current_file {
            if !file.is_empty() {
                let file_text = if prefs.use_colors {
                    prefs.theme.apply_file(file)
                } else {
                    file.clone()
                };
                activity_parts.push(file_text);
            }
        }
    }

    let activity_text = activity_parts.join(" ");
    if activity_text.is_empty() {
        return None;
    }
    Some(activity_text)
}

/// Render the model section
fn render_model_section(model_name: &str, prefs: &PersonalityPreferences) -> Option<String> {
    if !prefs.show_model {
        return None;
    }

    let model_icon = if prefs.show_model_icon {
        get_model_icon(model_name)
    } else {
        ""
    };

    // Build model text piece by piece: icon + label
    let mut model_parts = Vec::new();
    if !model_icon.is_empty() {
        model_parts.push(model_icon.to_string());
    }
    if prefs.show_model_label {
        model_parts.push(model_name.to_string());
    }

    if model_parts.is_empty() {
        return None;
    }

    let model_text = model_parts.join(" ");
    let colored_model = if prefs.use_colors {
        prefs
            .theme
            .apply_model_color_with_context(&model_text, model_name)
    } else {
        model_text
    };

    Some(colored_model)
}

/// Render the update available section
fn render_update_section(
    update_available: Option<&str>,
    prefs: &PersonalityPreferences,
) -> Option<String> {
    let version = update_available?;
    let update_icon = "\u{f062}"; // nf-fa-arrow_up
    let update_text = if update_icon.is_empty() {
        format!("v{version}")
    } else {
        format!("{update_icon} v{version}")
    };

    let colored_update = if prefs.use_colors {
        prefs.theme.apply_success(&update_text)
    } else {
        update_text
    };

    Some(colored_update)
}

/// Render the debug info section
fn render_debug_section(state: &SessionState, prefs: &PersonalityPreferences) -> Option<String> {
    if !prefs.display.show_debug_info {
        return None;
    }

    let debug_info = format!(
        "[E:{} C:{} S:{}]",
        state.error_count, state.consecutive_actions, state.session_id
    );

    let debug_text = if prefs.use_colors {
        prefs.theme.apply_separator(&debug_info)
    } else {
        debug_info
    };

    Some(debug_text)
}

/// Helper to add a section with proper separator handling
fn add_section_to_parts(
    parts: &mut Vec<String>,
    section_text: Option<String>,
    prefs: &PersonalityPreferences,
) {
    let Some(text) = section_text else {
        return;
    };
    if text.is_empty() {
        return;
    }

    if parts.is_empty() {
        parts.push(text);
    } else {
        let spacing = " ";
        if prefs.display.show_separators {
            let separator = if prefs.use_colors {
                prefs.theme.apply_separator(&prefs.display.separator_char)
            } else {
                prefs.display.separator_char.clone()
            };
            parts.push(format!("{spacing}{separator}{spacing}{text}"));
        } else {
            parts.push(format!("{spacing}{text}"));
        }
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
            current_file: None,
            git_branch: None,
            git_dirty: None,
            git_dirty_count: None,
            git_status_checked_at: None,
            personality: "ლ(╹◡╹ლ) Cowder".to_string(),
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
        let statusline = build_statusline(&state, "Opus", &prefs, None, None);

        // Should contain personality (bold formatting is applied but we can't easily test ANSI codes)
        assert!(statusline.contains("ლ(╹◡╹ლ) Cowder"));

        // Should contain activity and job
        assert!(statusline.contains("Editing"));
        assert!(statusline.contains("test.js"));

        // Should contain model info
        assert!(statusline.contains("Opus"));

        // Error indicators have been completely removed
    }

    #[test]
    fn test_build_statusline_with_errors() {
        let mut state = create_test_state();
        let prefs = create_test_preferences();

        // Error indicators have been removed - statusline should not contain error icons
        state.error_count = 1;
        let statusline = build_statusline(&state, "Sonnet", &prefs, None, None);

        // Test with many errors - should still not show any error icons
        state.error_count = 5;
        let statusline = build_statusline(&state, "Sonnet", &prefs, None, None);

        // Just verify the statusline is not empty
        assert!(!statusline.trim().is_empty());
    }

    #[test]
    fn test_build_statusline_no_job() {
        let mut state = create_test_state();
        let prefs = create_test_preferences();
        state.current_job = None;
        let statusline = build_statusline(&state, "Haiku", &prefs, None, None);

        // Should contain activity but no job
        assert!(statusline.contains("Editing"));
        assert!(!statusline.contains("test.js"));
    }

    #[test]
    fn test_build_statusline_empty_job() {
        let mut state = create_test_state();
        let prefs = create_test_preferences();
        state.current_job = Some(String::new());
        let statusline = build_statusline(&state, "Haiku", &prefs, None, None);

        // Should treat empty job same as no job
        assert!(statusline.contains("Editing"));
    }

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
        let opus_output = theme.apply_model_color("Opus", "Opus");
        assert!(opus_output.contains("Opus"));

        let sonnet_output = theme.apply_model_color("Sonnet", "Sonnet");
        assert!(sonnet_output.contains("Sonnet"));

        let haiku_output = theme.apply_model_color("Haiku", "Haiku");
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

        // Only these activities should have icons now
        let activities_with_icons = [
            (Activity::Executing, ICON_EXECUTING),
            (Activity::Reading, ICON_READING),
            (Activity::Idle, ICON_IDLE),
        ];
        for (activity, expected_icon) in activities_with_icons {
            state.activity = activity.clone();
            let statusline = build_statusline(&state, "Claude", &prefs, None, None);
            // The icon should be in the statusline
            assert!(statusline.contains(expected_icon));
            assert!(statusline.contains(&activity.to_string()));
        }

        // Test activities without icons still show activity text
        let activities_without_icons = [
            Activity::Editing,
            Activity::Coding,
            Activity::Testing,
            Activity::Building,
        ];
        for activity in activities_without_icons {
            state.activity = activity.clone();
            let statusline = build_statusline(&state, "Claude", &prefs, None, None);
            // Should contain activity text but no icon
            assert!(statusline.contains(&activity.to_string()));
        }
    }

    #[test]
    fn test_statusline_formatting_structure() {
        let state = create_test_state();
        let prefs = create_test_preferences();
        let statusline = build_statusline(&state, "TestModel", &prefs, None, None);

        // Should contain separators
        assert!(statusline.contains("•"));

        // Should contain model name (no brackets now)
        assert!(statusline.contains("TestModel"));

        // Should be structured as: personality • activity job • model
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

        let statusline = build_statusline(&state, "Claude", &prefs, None, None);

        // Should handle long job names gracefully
        assert!(statusline.contains("very_long_filename_that_might_cause_display_issues.js"));
        assert!(statusline.len() > 50); // Should be a substantial statusline
    }

    #[test]
    fn test_different_personalities() {
        let mut state = create_test_state();
        let prefs = create_test_preferences();
        let personalities = [
            "ლ(╹◡╹ლ) Cowder",
            "(╯°□°)╯︵ ┻━┻ Table Flipper",
            "┗(▀̿Ĺ̯▀̿ ̿)┓ Git Manager",
            "φ(．．) Documentation Writer",
        ];

        for personality in personalities {
            state.personality = personality.to_string();
            let statusline = build_statusline(&state, "Claude", &prefs, None, None);
            assert!(statusline.contains(personality));
        }
    }

    #[test]
    fn test_error_states() {
        let mut state = create_test_state();
        let prefs = create_test_preferences();

        // Error indicators have been completely removed
        // Statusline should work normally regardless of error count
        state.error_count = 0;
        let statusline = build_statusline(&state, "Claude", &prefs, None, None);
        assert!(!statusline.trim().is_empty());

        state.error_count = 1;
        let statusline = build_statusline(&state, "Claude", &prefs, None, None);
        assert!(!statusline.trim().is_empty());

        state.error_count = 10;
        let statusline = build_statusline(&state, "Claude", &prefs, None, None);
        assert!(!statusline.trim().is_empty());
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
        let statusline_all = build_statusline(&state, "Opus", &prefs_all, None, None);

        // Should contain personality, activity, model, and separators
        assert!(statusline_all.contains("Chillin"));
        assert!(statusline_all.contains("Editing"));
        assert!(statusline_all.contains("test.js"));
        assert!(statusline_all.contains("Opus"));
        assert!(statusline_all.contains("•")); // Separator

        // Test with minimal configuration
        let prefs_minimal = PersonalityPreferences {
            show_personality: false,
            show_activity: false,
            show_model: false,
            show_activity_icon: false,
            show_git_icon: false,
            show_directory_icon: false,
            show_model_icon: false,
            use_colors: false,
            display: DisplayConfig {
                show_separators: false,
                ..Default::default()
            },
            ..Default::default()
        };

        let statusline_minimal = build_statusline(&state, "Opus", &prefs_minimal, None, None);
        // Should be empty since we disabled everything important
        assert!(statusline_minimal.is_empty());
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
        let statusline_normal = build_statusline(&state, "Sonnet", &prefs_normal, None, None);
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
        let statusline_debug = build_statusline(&state, "Sonnet", &prefs_debug, None, None);
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
        let statusline_with_sep = build_statusline(&state, "Haiku", &prefs_with_sep, None, None);
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
        let statusline_no_sep = build_statusline(&state, "Haiku", &prefs_no_sep, None, None);
        assert!(!statusline_no_sep.contains("•"));
        // But should still have content
        assert!(statusline_no_sep.contains("Chillin"));
        assert!(statusline_no_sep.contains("Coding"));
        assert!(statusline_no_sep.contains("app.rs"));
        assert!(statusline_no_sep.contains("Haiku"));
    }
}
