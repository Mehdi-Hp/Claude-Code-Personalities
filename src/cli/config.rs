//! Configuration management commands for Claude Code Personalities
//!
//! This module provides interactive configuration commands for customizing
//! display preferences, mood settings, and advanced options.

use anyhow::{Context, Result};
use colored::Colorize;
use inquire::{Confirm, MultiSelect, Select};

use crate::config::PersonalityPreferences;
use crate::icons::{ICON_CHECK, ICON_ERROR, ICON_INFO, ICON_WARNING};
use crate::theme::Theme;

/// Handle configuration subcommands
pub async fn handle_config_command(subcommand: Option<&str>) -> Result<()> {
    match subcommand {
        Some("display") => configure_display().await,
        Some("theme") => configure_theme().await,
        Some("reset") => reset_configuration().await,
        Some("show") => show_current_configuration().await,
        Some("export") => export_configuration().await,
        Some("import") => import_configuration().await,
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
    use inquire::Select;

    println!("{}", "Configure Claude Code Personalities".bold().blue());
    println!("Select a configuration category:\n");

    let options = vec![
        "Display Options - What appears in the statusline",
        "Theme - Change colors and visual style",
        "Show Current Config - View all current settings",
        "Reset to Defaults - Reset all settings",
        "Export Config - Save current config to file",
        "Import Config - Load config from file",
    ];

    let selection = Select::new("Configuration category:", options)
        .prompt()
        .with_context(
            || "Failed to get user selection. Interactive prompt was cancelled or failed.",
        )?;

    match selection {
        s if s.starts_with("Display Options") => configure_display().await,
        s if s.starts_with("Theme") => configure_theme().await,
        s if s.starts_with("Show Current Config") => show_current_configuration().await,
        s if s.starts_with("Reset to Defaults") => reset_configuration().await,
        s if s.starts_with("Export Config") => export_configuration().await,
        s if s.starts_with("Import Config") => import_configuration().await,
        _ => Ok(()),
    }
}

/// Configure display preferences (replaces the old configure function)
async fn configure_display() -> Result<()> {
    println!("{}", "Configure Display Options".bold().blue());
    println!("Select which elements to show in the statusline:\n");

    // Load current preferences or defaults
    let mut prefs = PersonalityPreferences::load_or_default()
        .await
        .with_context(|| "Failed to load current personality preferences")?;

    // Get all display options with their current states
    let options = prefs.get_display_options();
    let option_names: Vec<&str> = options.iter().map(|(name, _)| *name).collect();

    // Get indices of currently selected options
    let default_selections: Vec<usize> = options
        .iter()
        .enumerate()
        .filter_map(|(i, (_, enabled))| if *enabled { Some(i) } else { None })
        .collect();

    // Show interactive multi-select prompt
    let selected = MultiSelect::new("Features to enable:", option_names.clone())
        .with_default(&default_selections)
        .prompt()
        .with_context(|| "Failed to get user preferences selection. Interactive prompt was cancelled or failed.")?;

    // Update preferences based on selections
    prefs.update_from_selections(&selected);

    // Save updated preferences
    prefs
        .save()
        .await
        .with_context(|| "Failed to save updated personality preferences")?;

    println!(
        "\n{} Display configuration saved successfully!",
        ICON_CHECK.green()
    );

    let prefs_path = PersonalityPreferences::get_preferences_path()
        .with_context(|| "Failed to get preferences file path for display")?;
    println!("Location: {}", prefs_path.display());

    // Show what was enabled/disabled
    println!("\nEnabled features:");
    for feature in &selected {
        println!("  {} {}", ICON_CHECK.green(), feature);
    }

    if selected.len() < option_names.len() {
        println!("\nDisabled features:");
        for option in &option_names {
            if !selected.contains(option) {
                println!("  {} {}", ICON_WARNING.yellow(), option);
            }
        }
    }

    println!(
        "\n{} Run your Claude Code session to see the changes!",
        ICON_INFO.cyan()
    );

    Ok(())
}

/// Configure theme (color scheme)
async fn configure_theme() -> Result<()> {
    println!("{}", "Configure Theme".bold().blue());
    println!("Select a theme for Claude Code Personalities:\n");

    // Load current preferences
    let mut prefs = PersonalityPreferences::load_or_default()
        .await
        .with_context(|| "Failed to load current personality preferences")?;

    // Get all available themes with descriptions
    let themes = Theme::all();
    let theme_options: Vec<String> = themes
        .iter()
        .map(|theme| format!("{} - {}", theme.display_name(), theme.description()))
        .collect();

    // Find current theme index
    let current_theme_index = themes
        .iter()
        .position(|theme| *theme == prefs.theme)
        .unwrap_or(0);

    println!("Current theme: {}", prefs.theme.display_name().bold());
    println!();

    // Show theme selection
    let selection = Select::new("Choose a theme:", theme_options.clone())
        .with_starting_cursor(current_theme_index)
        .prompt()
        .with_context(
            || "Failed to get theme selection. Interactive prompt was cancelled or failed.",
        )?;

    // Find the selected theme
    let selected_theme_index = theme_options
        .iter()
        .position(|opt| opt == &selection)
        .unwrap_or(0);

    let selected_theme = &themes[selected_theme_index];

    // Preview the theme
    println!("\n{} Theme Preview:", ICON_INFO.cyan());
    preview_theme(selected_theme);

    // Confirm selection if it's different from current
    if *selected_theme != prefs.theme {
        let confirmed = Confirm::new("Apply this theme?")
            .with_default(true)
            .prompt()
            .with_context(|| "Failed to get theme confirmation")?;

        if confirmed {
            prefs.theme = selected_theme.clone();
            prefs
                .save()
                .await
                .with_context(|| "Failed to save theme configuration")?;

            println!(
                "\n{} Theme '{}' applied successfully!",
                ICON_CHECK.green(),
                selected_theme.display_name()
            );

            let prefs_path = PersonalityPreferences::get_preferences_path()
                .with_context(|| "Failed to get preferences file path for display")?;
            println!("Configuration saved to: {}", prefs_path.display());

            println!(
                "\n{} Run your Claude Code session to see the new theme!",
                ICON_INFO.cyan()
            );
        } else {
            println!("{} Theme change cancelled.", ICON_INFO.cyan());
        }
    } else {
        println!(
            "{} Theme '{}' is already selected.",
            ICON_INFO.cyan(),
            selected_theme.display_name()
        );
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
    println!("  Error: {}", theme.apply_error("⚠ Error"));
    println!("  Warning: {}", theme.apply_warning("⚡ Warning"));
    println!("  Success: {}", theme.apply_success("✓ Success"));
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

/// Show current configuration
async fn show_current_configuration() -> Result<()> {
    let prefs = PersonalityPreferences::load_or_default()
        .await
        .with_context(|| "Failed to load current personality preferences")?;

    println!("{}", "Current Configuration".bold().blue());
    println!();

    println!("{}", "Display Options:".bold());
    for (name, enabled) in prefs.get_display_options() {
        let status = if enabled {
            ICON_CHECK.green().to_string()
        } else {
            ICON_WARNING.yellow().to_string()
        };
        println!("  {} {}", status, name);
    }

    println!();
    println!("{}", "Theme:".bold());
    println!(
        "  {} Current: {}",
        ICON_CHECK.green(),
        prefs.theme.display_name()
    );
    println!(
        "  {} Description: {}",
        ICON_INFO.cyan(),
        prefs.theme.description()
    );

    let config_path = PersonalityPreferences::get_preferences_path()
        .with_context(|| "Failed to get preferences file path")?;

    println!(
        "\n{} Configuration file: {}",
        ICON_INFO.cyan(),
        config_path.display()
    );

    Ok(())
}

/// Reset all configuration to defaults
async fn reset_configuration() -> Result<()> {
    println!("{}", "Reset Configuration".bold().red());
    println!("This will reset ALL settings to their default values.\n");

    let confirmed = Confirm::new("Are you sure you want to reset all configuration?")
        .with_default(false)
        .prompt()
        .with_context(|| "Failed to get confirmation")?;

    if confirmed {
        let prefs = PersonalityPreferences::default();
        prefs
            .save()
            .await
            .with_context(|| "Failed to save reset configuration")?;

        println!(
            "{} All configuration has been reset to defaults!",
            ICON_CHECK.green()
        );
        println!("Run 'claude-code-personalities config show' to see current settings.");
    } else {
        println!("{} Configuration reset cancelled.", ICON_INFO.cyan());
    }

    Ok(())
}

/// Export current configuration to a file
async fn export_configuration() -> Result<()> {
    use std::path::Path;
    use tokio::fs;

    println!("{}", "Export Configuration".bold().blue());

    let prefs = PersonalityPreferences::load_or_default()
        .await
        .with_context(|| "Failed to load current personality preferences")?;

    // Default export path
    let default_path = "claude-code-personalities-config.json";
    let export_path = inquire::Text::new("Export to file:")
        .with_default(default_path)
        .prompt()
        .with_context(|| "Failed to get export file path")?;

    let export_path = Path::new(&export_path);

    // Check if file exists
    if export_path.exists() {
        let confirmed = Confirm::new(&format!(
            "File '{}' already exists. Overwrite?",
            export_path.display()
        ))
        .with_default(false)
        .prompt()
        .with_context(|| "Failed to get overwrite confirmation")?;

        if !confirmed {
            println!("{} Export cancelled.", ICON_INFO.cyan());
            return Ok(());
        }
    }

    // Export configuration
    let content = serde_json::to_string_pretty(&prefs)
        .with_context(|| "Failed to serialize configuration for export")?;

    fs::write(export_path, content)
        .await
        .with_context(|| format!("Failed to write configuration to {}", export_path.display()))?;

    println!(
        "{} Configuration exported to: {}",
        ICON_CHECK.green(),
        export_path.display()
    );

    Ok(())
}

/// Import configuration from a file
async fn import_configuration() -> Result<()> {
    use std::path::Path;
    use tokio::fs;

    println!("{}", "Import Configuration".bold().blue());

    let import_path = inquire::Text::new("Import from file:")
        .with_default("claude-code-personalities-config.json")
        .prompt()
        .with_context(|| "Failed to get import file path")?;

    let import_path = Path::new(&import_path);

    if !import_path.exists() {
        println!(
            "{} File '{}' does not exist.",
            ICON_ERROR.red(),
            import_path.display()
        );
        return Ok(());
    }

    // Load and validate configuration
    let content = fs::read_to_string(import_path).await.with_context(|| {
        format!(
            "Failed to read configuration from {}",
            import_path.display()
        )
    })?;

    let imported_prefs: PersonalityPreferences = serde_json::from_str(&content)
        .with_context(|| format!("Invalid configuration format in {}", import_path.display()))?;

    // Show what will be imported
    println!("\nConfiguration to import:");
    println!(
        "  Display options: {} enabled",
        imported_prefs
            .get_display_options()
            .iter()
            .filter(|(_, enabled)| *enabled)
            .count()
    );
    println!(
        "  Compact mode: {}",
        if imported_prefs.display.compact_mode {
            "enabled"
        } else {
            "disabled"
        }
    );
    println!(
        "  Debug info: {}",
        if imported_prefs.display.show_debug_info {
            "enabled"
        } else {
            "disabled"
        }
    );

    let confirmed = Confirm::new("\nImport this configuration?")
        .with_default(true)
        .prompt()
        .with_context(|| "Failed to get import confirmation")?;

    if confirmed {
        imported_prefs
            .save()
            .await
            .with_context(|| "Failed to save imported configuration")?;

        println!(
            "{} Configuration imported successfully from: {}",
            ICON_CHECK.green(),
            import_path.display()
        );
        println!("Run 'claude-code-personalities config show' to verify settings.");
    } else {
        println!("{} Import cancelled.", ICON_INFO.cyan());
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
    println!("  show       Show current configuration");
    println!("  reset      Reset all settings to defaults");
    println!("  export     Export configuration to a file");
    println!("  import     Import configuration from a file");
    println!();
    println!("If no subcommand is provided, an interactive menu will be shown.");
}
