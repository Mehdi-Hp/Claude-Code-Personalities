//! Configuration management commands for Claude Code Personalities
//!
//! This module provides interactive configuration commands for customizing
//! display preferences, mood settings, and advanced options.

use anyhow::{Context, Result};
use cliclack::{confirm, intro, multiselect, outro, select};
use colored::Colorize;

use crate::config::PersonalityPreferences;
use crate::icons::{ICON_CHECK, ICON_ERROR, ICON_INFO, ICON_WARNING};
use crate::theme::Theme;

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

/// Configure display preferences (replaces the old configure function)
async fn configure_display() -> Result<()> {
    intro("Configure Display Options")?;

    // Load current preferences or defaults
    let mut prefs = PersonalityPreferences::load_or_default()
        .await
        .with_context(|| "Failed to load current personality preferences")?;

    // Get all display options with their current states
    let options = prefs.get_display_options();

    // Build multiselect with items and initial selections
    let mut selector = multiselect("Select which elements to show in the statusline");

    // Add each option with its current state
    // Note: cliclack doesn't support initial selections in multiselect,
    // so we'll show current state in the hint
    for (name, enabled) in &options {
        let hint = if *enabled { "(currently enabled)" } else { "" };
        selector = selector.item(name, name, hint);
    }

    // Show interactive multi-select prompt
    let selected: Vec<&&str> = selector.interact().with_context(
        || "Failed to get user preferences selection. Interactive prompt was cancelled or failed.",
    )?;

    // Convert selected to owned strings first, then to refs
    let selected_strings: Vec<String> = selected.iter().map(|s| s.to_string()).collect();
    let selected_refs: Vec<&str> = selected_strings.iter().map(|s| s.as_str()).collect();
    prefs.update_from_selections(&selected_refs);

    // Save updated preferences
    prefs
        .save()
        .await
        .with_context(|| "Failed to save updated personality preferences")?;

    let prefs_path = PersonalityPreferences::get_preferences_path()
        .with_context(|| "Failed to get preferences file path for display")?;

    // Show what was enabled/disabled before outro
    println!("\nEnabled features:");
    for feature in &selected_strings {
        println!("  {} {}", ICON_CHECK.green(), feature);
    }

    // Check if any options were disabled
    let all_options: Vec<&str> = options.iter().map(|(name, _)| *name).collect();
    let disabled_options: Vec<&&str> = all_options
        .iter()
        .filter(|opt| !selected_refs.contains(opt))
        .collect();

    if !disabled_options.is_empty() {
        println!("\nDisabled features:");
        for option in disabled_options {
            println!("  {} {}", ICON_WARNING.yellow(), option);
        }
    }

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
        theme.apply_personality("ʕ•ᴥ•ʔ Code Wizard")
    );
    println!("  Activity: {}", theme.apply_activity("editing"));
    println!("  Directory: {}", theme.apply_directory("my-project"));
    println!("  File: {}", theme.apply_file("main.rs"));
    println!("  Error: {}", theme.apply_error("\u{f071} Error"));
    println!("  Warning: {}", theme.apply_warning("\u{f0e7} Warning"));
    println!("  Success: {}", theme.apply_success("\u{f00c} Success"));
    println!("  Info: {}", theme.apply_info("\u{f129} Info"));
    println!("  Model Colors:");
    println!("    Opus: {}", theme.apply_model_color("[ Opus]", "Opus"));
    println!(
        "    Sonnet: {}",
        theme.apply_model_color("[ Sonnet]", "Sonnet")
    );
    println!(
        "    Haiku: {}",
        theme.apply_model_color("[ Haiku]", "Haiku")
    );
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
