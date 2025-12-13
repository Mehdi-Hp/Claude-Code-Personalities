//! Configuration management commands for Claude Code Personalities
//!
//! This module provides interactive configuration commands for customizing
//! display preferences, mood settings, and advanced options.

use anyhow::{Context, Result};
use cliclack::{confirm, intro, outro, select};
use colored::Colorize;

use crate::cli::interactive_config;
use crate::config::PersonalityPreferences;
use crate::icons::{ICON_CHECK, ICON_ERROR, ICON_INFO, ICON_WARNING};
use crate::state::SessionState;
use crate::statusline::{WorkspaceInfo, build_statusline};
use crate::theme::Theme;
use crate::types::Activity;

/// Handle configuration subcommands
pub async fn handle_config_command(subcommand: Option<&str>) -> Result<()> {
    match subcommand {
        Some("display") => configure_display().await,
        Some("theme") => {
            println!(
                "{} Theme configuration is temporarily disabled while the new Default theme is being finalized.",
                ICON_WARNING.yellow()
            );
            println!("The Default theme with context-aware colors is now active by default.");
            Ok(())
        }
        Some("reset") => reset_configuration().await,
        None => interactive_config_menu().await,
        Some(unknown) => {
            println!(
                "{} Unknown config subcommand: {}",
                ICON_ERROR.red(),
                unknown
            );
            print_config_help();
            Ok(())
        }
    }
}

/// Interactive configuration menu (default when just running 'config')
async fn interactive_config_menu() -> Result<()> {
    intro("Configure Claude Code Personalities")?;

    // Show current statusline preview
    let current_prefs = PersonalityPreferences::load_or_default()
        .await
        .with_context(|| "Failed to load current personality preferences")?;

    show_statusline_preview(&current_prefs);

    let selection = select("Select a configuration category")
        .item(
            "display",
            "Display Options",
            "What appears in the statusline",
        )
        // Theme temporarily hidden while Default theme is being finalized
        // .item("theme", "Theme", "Change colors and visual style")
        .item("reset", "Reset to Defaults", "Reset all settings")
        .interact()
        .with_context(
            || "Failed to get user selection. Interactive prompt was cancelled or failed.",
        )?;

    match selection.as_ref() {
        "display" => configure_display().await,
        "theme" => configure_theme().await, // Keep handler in case called directly
        "reset" => reset_configuration().await,
        _ => Ok(()),
    }
}

/// Configure display preferences with live-updating preview
async fn configure_display() -> Result<()> {
    intro("Configure Display Options")?;

    // Load current preferences or defaults
    let prefs = PersonalityPreferences::load_or_default()
        .await
        .with_context(|| "Failed to load current personality preferences")?;

    // Launch interactive TUI
    let updated_prefs = interactive_config::run_config_ui(prefs)
        .await
        .with_context(|| "Failed to run interactive configuration UI")?;

    // Save updated preferences
    updated_prefs
        .save()
        .await
        .with_context(|| "Failed to save updated personality preferences")?;

    let prefs_path = PersonalityPreferences::get_preferences_path()
        .with_context(|| "Failed to get preferences file path for display")?;

    outro(format!(
        "{} Display configuration saved to: {}",
        ICON_CHECK.green(),
        prefs_path.display()
    ))?;

    Ok(())
}

/// Configure theme (color scheme)
async fn configure_theme() -> Result<()> {
    intro("Configure Theme")?;

    // Load current preferences
    let mut prefs = PersonalityPreferences::load_or_default()
        .await
        .with_context(|| "Failed to load current personality preferences")?;

    // Get all available themes with descriptions
    let themes = Theme::all();

    println!("Current theme: {}\n", prefs.theme.display_name().bold());

    // Build select with theme items
    // Create IDs outside the loop to ensure they live long enough
    let theme_ids: Vec<String> = themes
        .iter()
        .enumerate()
        .map(|(index, _)| format!("theme_{}", index))
        .collect();

    let mut selector = select("Choose a theme");
    for (theme, id) in themes.iter().zip(&theme_ids) {
        selector = selector.item(id, theme.display_name(), theme.description());
    }

    // Show theme selection
    let selection = selector.interact().with_context(
        || "Failed to get theme selection. Interactive prompt was cancelled or failed.",
    )?;

    // Find the selected theme from the ID
    let selected_theme_index = selection
        .strip_prefix("theme_")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);

    let selected_theme = &themes[selected_theme_index];

    // Preview the theme
    println!("\n{} Theme Preview:", ICON_INFO.cyan());
    preview_theme(selected_theme);

    // Confirm selection if it's different from current
    if *selected_theme != prefs.theme {
        let confirmed = confirm("Apply this theme?")
            .initial_value(true)
            .interact()
            .with_context(|| "Failed to get theme confirmation")?;

        if confirmed {
            prefs.theme = selected_theme.clone();
            prefs
                .save()
                .await
                .with_context(|| "Failed to save theme configuration")?;

            let prefs_path = PersonalityPreferences::get_preferences_path()
                .with_context(|| "Failed to get preferences file path for display")?;

            outro(format!(
                "{} Theme '{}' applied! Configuration saved to: {}",
                ICON_CHECK.green(),
                selected_theme.display_name(),
                prefs_path.display()
            ))?;
        } else {
            outro(format!("{} Theme change cancelled.", ICON_INFO.cyan()))?;
        }
    } else {
        outro(format!(
            "{} Theme '{}' is already selected.",
            ICON_INFO.cyan(),
            selected_theme.display_name()
        ))?;
    }

    Ok(())
}

/// Preview a theme by showing sample colored text
fn preview_theme(theme: &Theme) {
    println!(
        "  Personality: {}",
        theme.apply_personality("ლ(╹◡╹ლ) Cowder")
    );
    println!("  Activity: {}", theme.apply_activity("editing"));
    println!("  Directory: {}", theme.apply_directory("my-project"));
    println!("  File: {}", theme.apply_file("main.rs"));
    println!("  Error: {}", theme.apply_error("\u{f071} Error"));
    println!("  Warning: {}", theme.apply_warning("\u{f0e7} Warning"));
    println!("  Success: {}", theme.apply_success("\u{f00c} Success"));
    println!("  Info: {}", theme.apply_info("\u{f129} Info"));
    println!("  Model Colors:");
    println!("    Opus: {}", theme.apply_model_color("Opus", "Opus"));
    println!(
        "    Sonnet: {}",
        theme.apply_model_color("Sonnet", "Sonnet")
    );
    println!("    Haiku: {}", theme.apply_model_color("Haiku", "Haiku"));
}

/// Reset all configuration to defaults
async fn reset_configuration() -> Result<()> {
    intro("Reset Configuration")?;

    println!(
        "{} This will reset ALL settings to their default values.",
        ICON_WARNING.yellow()
    );
    println!();

    let confirmed = confirm("Are you sure you want to reset all configuration?")
        .initial_value(false)
        .interact()
        .with_context(|| "Failed to get confirmation")?;

    if confirmed {
        let prefs = PersonalityPreferences::default();
        prefs
            .save()
            .await
            .with_context(|| "Failed to save reset configuration")?;

        outro(format!(
            "{} All configuration has been reset to defaults!",
            ICON_CHECK.green()
        ))?;
    } else {
        outro(format!(
            "{} Configuration reset cancelled.",
            ICON_INFO.cyan()
        ))?;
    }

    Ok(())
}

/// Create a sample session state for preview purposes
fn create_preview_state() -> SessionState {
    SessionState {
        session_id: "preview".to_string(),
        activity: Activity::Coding,
        current_job: None,
        current_file: Some("main.rs".to_string()),
        git_branch: Some("main".to_string()),
        git_dirty: Some(true),    // Show dirty state in preview
        git_dirty_count: Some(3), // Show 3 dirty files in preview
        git_status_checked_at: None,
        personality: "ლ(╹◡╹ლ) Cowder".to_string(),
        previous_personality: None,
        consecutive_actions: 5,
        error_count: 1,
        recent_activities: vec![Activity::Editing, Activity::Reading],
        mood: crate::state::MoodState::default(),
    }
}

/// Create a sample workspace for preview purposes
fn create_preview_workspace() -> WorkspaceInfo {
    WorkspaceInfo {
        current_dir: Some("/home/user/projects/claude-code-personalities".to_string()),
        project_dir: Some("/home/user/projects/claude-code-personalities".to_string()),
    }
}

/// Display a preview of what the statusline will look like
fn show_statusline_preview(prefs: &PersonalityPreferences) {
    use colored::Colorize;

    println!("{}", "━".repeat(60).bright_black());

    let state = create_preview_state();
    let workspace = create_preview_workspace();
    let statusline = build_statusline(&state, "Sonnet", prefs, Some(&workspace), None);

    println!("  {}", statusline);
    println!("{}", "━".repeat(60).bright_black());
}

/// Print help for config subcommands
fn print_config_help() {
    println!("Usage: claude-code-personalities config [SUBCOMMAND]");
    println!();
    println!("Subcommands:");
    println!("  display    Configure what appears in the statusline");
    println!("  theme      Change color theme");
    println!("  reset      Reset all settings to defaults");
    println!();
    println!("If no subcommand is provided, an interactive menu will be shown.");
}
