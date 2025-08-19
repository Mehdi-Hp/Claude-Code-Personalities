pub mod personality;
pub mod icons;
pub mod animated;

use anyhow::Result;
use serde::Deserialize;
use std::io::{self, Read};
use colored::*;

use crate::state::SessionState;
use crate::types::Activity;
use crate::config::PersonalityPreferences;
use icons::*;
use animated::AnimatedStatuslineRenderer;

#[derive(Debug, Deserialize)]
pub struct ClaudeInput {
    pub session_id: Option<String>,
    pub model: Option<ModelInfo>,
    #[allow(dead_code)]
    pub workspace: Option<WorkspaceInfo>,
}

#[derive(Debug, Deserialize)]
pub struct ModelInfo {
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WorkspaceInfo {
    #[allow(dead_code)]
    pub current_dir: Option<String>,
    #[allow(dead_code)]
    pub project_dir: Option<String>,
}

pub async fn run_statusline() -> Result<()> {
    use anyhow::Context;
    
    // Read JSON from stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)
        .with_context(|| "Failed to read input from stdin")?;
    
    if input.trim().is_empty() {
        return Err(anyhow::anyhow!("No input received from Claude Code"))
            .with_context(|| "Claude Code should pass JSON input via stdin. Check that statusline is configured correctly.");
    }
    
    let claude_input: ClaudeInput = serde_json::from_str(&input)
        .with_context(|| {
            let preview = if input.len() > 100 {
                format!("{}...", &input[..100])
            } else {
                input.clone()
            };
            format!("Failed to parse JSON input from Claude Code. Received: {}", preview)
        })?;
    
    let session_id = claude_input.session_id.unwrap_or_else(|| "unknown".to_string());
    let model_name = claude_input.model
        .and_then(|m| m.display_name)
        .unwrap_or_else(|| "Claude".to_string());
    
    // Load session state and preferences
    let mut state = SessionState::load(&session_id).await
        .with_context(|| format!("Failed to load session state for session '{}'", session_id))?;
    let prefs = PersonalityPreferences::load_or_default().await
        .with_context(|| "Failed to load personality preferences")?;
    
    // Check if animations are needed and enabled
    let needs_animation = should_use_animation(&state, &prefs);
    
    let statusline = if needs_animation {
        // Use animated renderer
        let mut animator = AnimatedStatuslineRenderer::new(prefs.clone());
        animator.render_animated_statusline(&mut state, &model_name, &prefs).await
            .unwrap_or_else(|_| {
                // Fallback to static if animation fails
                build_statusline(&state, &model_name, &prefs)
            })
    } else {
        // Use static renderer
        build_statusline(&state, &model_name, &prefs)
    };
    
    println!("{}", statusline);
    
    // Save state if it was modified
    if state.animation_state.should_transition {
        state.save().await.ok(); // Don't fail if save fails
    }
    
    Ok(())
}

pub fn build_statusline(state: &SessionState, model_name: &str, prefs: &PersonalityPreferences) -> String {
    let mut parts = Vec::new();
    
    // Personality (bold)
    if prefs.show_personality {
        let personality_text = if prefs.use_colors {
            state.personality.bold().to_string()
        } else {
            state.personality.clone()
        };
        parts.push(personality_text);
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
        activity_parts.push(state.activity.to_string());
        
        // Current job/file
        if prefs.show_current_job {
            if let Some(job) = &state.current_job {
                if !job.is_empty() {
                    let job_text = if prefs.use_colors {
                        job.yellow().to_string()
                    } else {
                        job.clone()
                    };
                    activity_parts.push(job_text);
                }
            }
        }
        
        let activity_text = activity_parts.join(" ");
        let separator = if prefs.use_colors {
            "•".truecolor(128, 128, 128).to_string()
        } else {
            "•".to_string()
        };
        
        if !parts.is_empty() {
            parts.push(format!(" {} {}", separator, activity_text));
        } else {
            parts.push(activity_text);
        }
    }
    
    // Error indicators
    if prefs.show_error_indicators {
        if state.error_count >= 3 {
            let error_icon = if prefs.use_colors {
                ICON_ERROR.red().to_string()
            } else {
                ICON_ERROR.to_string()
            };
            parts.push(format!(" {}", error_icon));
        } else if state.error_count > 0 {
            let warning_icon = if prefs.use_colors {
                ICON_WARNING.yellow().to_string()
            } else {
                ICON_WARNING.to_string()
            };
            parts.push(format!(" {}", warning_icon));
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
            format!("[{}]", model_name)
        } else {
            format!("[{} {}]", model_icon, model_name)
        };
        
        let colored_model = if prefs.use_colors {
            let model_color = get_model_color(model_name);
            model_text.color(model_color).to_string()
        } else {
            model_text
        };
        
        let separator = if prefs.use_colors {
            "•".truecolor(128, 128, 128).to_string()
        } else {
            "•".to_string()
        };
        
        if !parts.is_empty() {
            parts.push(format!(" {} {}", separator, colored_model));
        } else {
            parts.push(colored_model);
        }
    }
    
    parts.join("")
}

fn get_activity_icon(activity: &Activity) -> &'static str {
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
    }
}

fn get_model_icon(_model_name: &str) -> &'static str {
    ICON_NORTH_STAR
}

fn get_model_color(model_name: &str) -> &'static str {
    if model_name.to_lowercase().contains("opus") {
        "magenta"
    } else if model_name.to_lowercase().contains("sonnet") {
        "cyan"
    } else if model_name.to_lowercase().contains("haiku") {
        "green"
    } else {
        "white"
    }
}

/// Determine if animation should be used based on state and preferences
fn should_use_animation(state: &SessionState, prefs: &PersonalityPreferences) -> bool {
    // Animations must be enabled in preferences
    if !prefs.enable_animations {
        return false;
    }
    
    // Use animation if:
    // 1. There's a pending personality transition
    if state.should_show_transition() && prefs.enable_transitions {
        return true;
    }
    
    // 2. Activity animations are enabled and we have an active activity
    if prefs.enable_activity_animations && state.activity != Activity::Idle {
        return true;
    }
    
    // 3. High error count should show error animations
    if state.error_count >= 3 {
        return true;
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::SessionState;

    fn create_test_state() -> SessionState {
        SessionState {
            session_id: "test".to_string(),
            activity: Activity::Editing,
            current_job: Some("test.js".to_string()),
            personality: "ʕ•ᴥ•ʔ Code Wizard".to_string(),
            consecutive_actions: 1,
            error_count: 0,
        }
    }

    fn create_test_preferences() -> PersonalityPreferences {
        PersonalityPreferences::default()
    }

    #[test]
    fn test_build_statusline_basic() {
        let state = create_test_state();
        let prefs = create_test_preferences();
        let statusline = build_statusline(&state, "Opus", &prefs);
        
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
        let statusline = build_statusline(&state, "Sonnet", &prefs);
        
        // Should contain warning for 1 error
        assert!(statusline.contains(ICON_WARNING));
        assert!(!statusline.contains(ICON_ERROR));
        
        // Test with many errors
        state.error_count = 5;
        let statusline = build_statusline(&state, "Sonnet", &prefs);
        assert!(statusline.contains(ICON_ERROR));
    }

    #[test]
    fn test_build_statusline_no_job() {
        let mut state = create_test_state();
        let prefs = create_test_preferences();
        state.current_job = None;
        let statusline = build_statusline(&state, "Haiku", &prefs);
        
        // Should contain activity but no job
        assert!(statusline.contains("editing"));
        assert!(!statusline.contains("test.js"));
    }

    #[test]
    fn test_build_statusline_empty_job() {
        let mut state = create_test_state();
        let prefs = create_test_preferences();
        state.current_job = Some("".to_string());
        let statusline = build_statusline(&state, "Haiku", &prefs);
        
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
        // All models should use the same icon currently
        assert_eq!(get_model_icon("Opus"), ICON_NORTH_STAR);
        assert_eq!(get_model_icon("Sonnet"), ICON_NORTH_STAR);
        assert_eq!(get_model_icon("Haiku"), ICON_NORTH_STAR);
        assert_eq!(get_model_icon("Unknown"), ICON_NORTH_STAR);
    }

    #[test]
    fn test_get_model_color() {
        // Test Claude model colors
        assert_eq!(get_model_color("Opus"), "magenta");
        assert_eq!(get_model_color("opus"), "magenta"); // Case insensitive
        assert_eq!(get_model_color("Claude Opus"), "magenta");
        
        assert_eq!(get_model_color("Sonnet"), "cyan");
        assert_eq!(get_model_color("sonnet"), "cyan");
        assert_eq!(get_model_color("Claude Sonnet"), "cyan");
        
        assert_eq!(get_model_color("Haiku"), "green");
        assert_eq!(get_model_color("haiku"), "green");
        assert_eq!(get_model_color("Claude Haiku"), "green");
        
        // Test unknown model
        assert_eq!(get_model_color("Unknown Model"), "white");
        assert_eq!(get_model_color("GPT-4"), "white");
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
        assert_eq!(claude_input.model.unwrap().display_name, Some("Opus".to_string()));
        assert!(claude_input.workspace.is_some());
    }

    #[test]
    fn test_claude_input_parsing_minimal() {
        let json_str = r#"{}"#;
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
            let statusline = build_statusline(&state, "Claude", &prefs);
            
            // The icon should be in the statusline (though we can't easily test positioning)
            assert!(statusline.contains(expected_icon));
            assert!(statusline.contains(&activity.to_string()));
        }
    }

    #[test]
    fn test_statusline_formatting_structure() {
        let state = create_test_state();
        let prefs = create_test_preferences();
        let statusline = build_statusline(&state, "TestModel", &prefs);
        
        // Should contain separators
        assert!(statusline.contains("•"));
        
        // Should contain brackets for model
        assert!(statusline.contains("["));
        assert!(statusline.contains("]"));
        
        // Should be structured as: personality • activity job • [model]
        // We can't test exact formatting due to ANSI codes, but can check basic structure
        let parts: Vec<&str> = statusline.split("•").collect();
        assert!(parts.len() >= 2); // At least personality and activity parts
    }

    #[test]
    fn test_long_job_names() {
        let mut state = create_test_state();
        let prefs = create_test_preferences();
        state.current_job = Some("very_long_filename_that_might_cause_display_issues.js".to_string());
        
        let statusline = build_statusline(&state, "Claude", &prefs);
        
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
            let statusline = build_statusline(&state, "Claude", &prefs);
            assert!(statusline.contains(personality));
        }
    }

    #[test]
    fn test_error_states() {
        let mut state = create_test_state();
        let prefs = create_test_preferences();
        
        // No errors
        state.error_count = 0;
        let statusline = build_statusline(&state, "Claude", &prefs);
        assert!(!statusline.contains(ICON_ERROR));
        assert!(!statusline.contains(ICON_WARNING));
        
        // Low errors (warning)
        state.error_count = 1;
        let statusline = build_statusline(&state, "Claude", &prefs);
        assert!(!statusline.contains(ICON_ERROR));
        assert!(statusline.contains(ICON_WARNING));
        
        state.error_count = 2;
        let statusline = build_statusline(&state, "Claude", &prefs);
        assert!(!statusline.contains(ICON_ERROR));
        assert!(statusline.contains(ICON_WARNING));
        
        // High errors (error icon)
        state.error_count = 3;
        let statusline = build_statusline(&state, "Claude", &prefs);
        assert!(statusline.contains(ICON_ERROR));
        assert!(!statusline.contains(ICON_WARNING));
        
        state.error_count = 10;
        let statusline = build_statusline(&state, "Claude", &prefs);
        assert!(statusline.contains(ICON_ERROR));
        assert!(!statusline.contains(ICON_WARNING));
    }
}