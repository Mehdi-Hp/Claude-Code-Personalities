use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

use crate::config::PersonalityPreferences;
use crate::icons::{ICON_CHECK, ICON_ERROR, ICON_INFO, ICON_WARNING};
use crate::version::CURRENT_VERSION;

// Sub-modules
pub mod config;
pub mod install;
pub mod settings;
pub mod uninstall;
pub mod update;

/// Install Claude Code Personalities with default options.
///
/// # Errors
///
/// This function will return an error if the installation process fails.
/// See [`install::install_personalities`] for detailed error conditions.
pub async fn install() -> Result<()> {
    let options = install::InstallationOptions::default();
    install::install_personalities(options).await
}

/// Update Claude Code Personalities to the latest version with default options.
///
/// # Errors
///
/// This function will return an error if the update process fails.
/// See [`update::update_personalities`] for detailed error conditions.
pub async fn update() -> Result<()> {
    let options = update::UpdateOptions::default();
    update::update_personalities(options).await
}

/// Uninstall Claude Code Personalities with default options.
///
/// # Errors
///
/// This function will return an error if the uninstallation process fails.
/// See [`uninstall::uninstall_personalities`] for detailed error conditions.
pub async fn uninstall() -> Result<()> {
    let options = uninstall::UninstallOptions::default();
    uninstall::uninstall_personalities(options).await
}

/// Display the current installation and configuration status.
///
/// # Errors
///
/// This function will return an error if:
/// - The Claude directory path cannot be determined
/// - Settings files cannot be read or parsed
/// - Update checking fails due to network or API errors
pub async fn status() -> Result<()> {
    use crate::state::SessionState;
    use crate::statusline::{ClaudeInput, build_statusline};
    use anyhow::Context;

    println!("{}", "Claude Code Personalities Status".bold().blue());
    println!();

    let claude_dir =
        get_claude_dir().with_context(|| "Failed to determine Claude directory path")?;
    let settings_file = claude_dir.join("settings.json");

    // Check if Claude directory exists
    if claude_dir.exists() {
        println!(
            "{} Claude directory found: {}",
            ICON_CHECK.green(),
            claude_dir.display()
        );
    } else {
        println!(
            "{} Claude directory not found: {}",
            ICON_ERROR.red(),
            claude_dir.display()
        );
    }

    // Check if settings.json exists
    if settings_file.exists() {
        println!("{} Settings file found", ICON_CHECK.green());

        // Use our settings module to check configuration
        let settings = settings::ClaudeSettings::load()
            .await
            .with_context(|| "Failed to load Claude settings")?;
        let summary = settings.get_configuration_summary();

        if summary.is_fully_configured() {
            println!("{} Personalities fully configured", ICON_CHECK.green());
        } else if summary.has_personality_statusline {
            println!(
                "{} Personalities partially configured",
                ICON_WARNING.yellow()
            );
        } else {
            println!(
                "{} Personalities not configured in settings",
                ICON_WARNING.yellow()
            );
        }
    } else {
        println!("{} Settings file not found", ICON_ERROR.red());
    }

    // Test statusline
    println!("\n{} Testing statusline output:", ICON_INFO.cyan());
    let test_input = r#"{"model":{"display_name":"Opus"},"workspace":{"current_dir":"/test"},"session_id":"test"}"#;

    // Simulate statusline output
    let claude_input: ClaudeInput = serde_json::from_str(test_input)
        .with_context(|| "Failed to parse test statusline input")?;
    let session_id = claude_input
        .session_id
        .unwrap_or_else(|| "test".to_string());
    let model_name = claude_input
        .model
        .and_then(|m| m.display_name)
        .unwrap_or_else(|| "Claude".to_string());

    let state = SessionState::load(&session_id)
        .await
        .with_context(|| format!("Failed to load test session state for session {session_id}"))?;
    let prefs = PersonalityPreferences::load_or_default()
        .await
        .with_context(|| "Failed to load preferences for status test")?;
    let statusline = build_statusline(&state, &model_name, &prefs, None);
    println!("  Output: {statusline}");

    Ok(())
}

/// Check for available updates and display version information.
///
/// # Errors
///
/// This function will return an error if:
/// - Version manager initialization fails
/// - GitHub API requests fail or return invalid responses
/// - Network connectivity issues prevent update checking
/// - Version parsing or comparison fails
pub async fn check_update_with_force(force: bool) -> Result<()> {
    use crate::version::{VersionManager, format_version_comparison};
    use anyhow::Context;

    println!("{}", "Checking for updates...".bold().blue());
    println!();

    let version_manager =
        VersionManager::new().with_context(|| "Failed to initialize version manager")?;

    if force {
        print_info("Force refresh enabled - bypassing cache...");
    } else {
        print_info("Checking latest version...");
    }

    let update_info = if force {
        version_manager
            .check_for_update_force()
            .await
            .with_context(|| "Failed to check for updates (forced refresh)")?
    } else {
        version_manager
            .check_for_update()
            .await
            .with_context(|| "Failed to check for updates")?
    };

    if let Some(release) = update_info {
        let latest_version = release
            .tag_name
            .strip_prefix('v')
            .unwrap_or(&release.tag_name);
        let comparison = format_version_comparison(CURRENT_VERSION, latest_version);

        println!();
        println!("{} {}", "📦 Update Available:".bold().green(), comparison);
        if let Some(name) = &release.name {
            println!("{} {}", "📋 Release:".bold(), name);
        }
        println!();
        println!(
            "Run {} to update",
            "claude-code-personalities update".cyan()
        );
    } else {
        println!();
        println!("{} You are running the latest version!", ICON_CHECK.green());
        println!("Current version: v{CURRENT_VERSION}");

        if !force {
            println!();
            print_info("Tip: Use --force to bypass cache and check GitHub directly");
        }
    }

    Ok(())
}

/// Configure personality display preferences through interactive prompts.
///
/// This is the legacy function that's now replaced by the new config subcommands.
/// It calls the display configuration from the config module for backward compatibility.
///
/// # Errors
///
/// This function will return an error if:
/// - Current preferences cannot be loaded from disk
/// - User interaction prompts fail to display or receive input
/// - Preferences cannot be saved after configuration
/// - File system operations fail during save
pub async fn configure() -> Result<()> {
    config::handle_config_command(Some("display")).await
}

/// Display help information and available commands.
///
/// # Errors
///
/// This function will return an error if:
/// - Output to stdout fails due to pipe closure or other I/O errors
pub fn help() -> Result<()> {
    println!("Claude Code Personalities v{CURRENT_VERSION}");
    println!("Dynamic text-face personalities for Claude Code's statusline");
    println!();
    println!("Usage: claude-code-personalities [COMMAND]");
    println!();
    println!("Commands:");
    println!("  install       Install Claude Code Personalities");
    println!("  update        Update to the latest version");
    println!("  uninstall     Remove Claude Code Personalities");
    println!("  status        Check installation status");
    println!("  check-update  Check for available updates");
    println!("  config        Configure display options");
    println!("  help          Show this help message");
    println!();
    println!("Modes (called by Claude Code):");
    println!("  --statusline  Run in statusline mode");
    println!("  --hook TYPE   Run in hook mode (activity, prompt-submit, session-end)");
    println!();
    println!("This is the Rust rewrite - much faster than the bash version!");

    Ok(())
}

fn get_claude_dir() -> Result<PathBuf> {
    settings::get_claude_dir()
}

/// Helper functions for status output
fn print_info(message: &str) {
    println!("  {} {}", ICON_INFO.cyan(), message);
}
