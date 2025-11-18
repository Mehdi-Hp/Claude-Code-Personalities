use anyhow::{Context, Result, anyhow};
use cliclack::{confirm, intro, outro};
use colored::Colorize;
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::cli::settings::{ClaudeSettings, get_claude_dir};
use crate::icons::{ICON_CHECK, ICON_INFO, ICON_WARNING};

pub struct InitOptions {
    pub non_interactive: bool,
    pub backup: bool,
}

impl Default for InitOptions {
    fn default() -> Self {
        Self {
            non_interactive: false,
            backup: true,
        }
    }
}

/// Initialize Claude Code settings for personalities.
///
/// This function configures Claude Code settings.json to enable:
/// - Dynamic statusline with personality faces
/// - Activity tracking hooks
/// - Session state management
///
/// # Errors
///
/// This function will return an error if:
/// - The Claude directory cannot be created or accessed
/// - Claude settings cannot be loaded or saved
/// - User cancels configuration when in interactive mode
/// - File permissions prevent settings modification
pub async fn init_claude_code(options: InitOptions) -> Result<()> {
    intro("Initializing Claude Code Personalities")?;

    // Step 1: Check if Claude directory exists
    let claude_dir = get_claude_dir()?;
    if claude_dir.exists() {
        print_success(&format!("Found Claude directory: {}", claude_dir.display()));
    } else {
        if !options.non_interactive {
            let create_dir = confirm("Claude directory not found. Create ~/.claude/ directory?")
                .initial_value(true)
                .interact()
                .with_context(|| "Failed to get user confirmation for directory creation")?;

            if !create_dir {
                return Err(anyhow!(
                    "Initialization cancelled: Claude directory is required"
                ));
            }
        }

        fs::create_dir_all(&claude_dir).await.with_context(|| {
            format!(
                "Failed to create Claude directory: {}",
                claude_dir.display()
            )
        })?;
        print_success(&format!("Created directory: {}", claude_dir.display()));
    }

    // Step 2: Find the installed binary location
    let binary_path = find_existing_binary().await?
        .ok_or_else(|| anyhow!(
            "No claude-code-personalities binary found in PATH or ~/.local/bin. \
            Please install the binary first using the install.sh script or download from GitHub releases."
        ))?;

    print_success(&format!("Found binary: {}", binary_path.display()));

    // Step 3: Load Claude settings
    let mut settings = ClaudeSettings::load()
        .await
        .with_context(|| "Failed to load Claude settings")?;

    // Step 4: Check current Claude Code configuration
    let config_summary = settings.get_configuration_summary();
    let settings_configured = config_summary.has_personality_statusline;

    if settings_configured {
        // Claude Code is already configured
        if !options.non_interactive {
            let reconfigure =
                confirm("Claude Code Personalities is already configured. Reconfigure?")
                    .initial_value(false)
                    .interact()
                    .with_context(|| "Failed to get user confirmation for reconfiguration")?;

            if !reconfigure {
                print_info("Configuration skipped.");
                return Ok(());
            }
        } else {
            print_info("Already configured. Proceeding with reconfiguration...");
        }
    }

    // Step 5: Create backup if requested and settings exist
    if options.backup && settings.settings_path.exists() {
        let backup_path = settings
            .create_backup()
            .await
            .with_context(|| "Failed to create settings backup")?;
        print_success(&format!("Backup created: {}", backup_path.display()));
    }

    // Step 6: Configure statusline
    settings
        .configure_statusline(&binary_path)
        .with_context(|| "Failed to configure statusline in settings")?;
    print_success("Statusline configured");

    // Step 7: Configure hooks
    settings
        .configure_hooks(&binary_path)
        .with_context(|| "Failed to configure hooks in settings")?;
    print_success("Hooks configured");

    // Step 8: Save settings
    settings
        .save()
        .await
        .with_context(|| "Failed to save updated Claude settings")?;
    print_success(&format!(
        "Settings saved to: {}",
        settings.settings_path.display()
    ));

    // Step 9: Verify configuration
    let final_summary = settings.get_configuration_summary();
    if final_summary.is_fully_configured() {
        print_success("Configuration verified successfully");
    } else {
        print_warning("Configuration completed but verification failed");
        return Err(anyhow!(
            "Configuration verification failed - some components may not be configured correctly"
        ));
    }

    // Step 10: Show success message
    println!();
    print_init_success(&binary_path, &settings.settings_path)?;

    Ok(())
}

/// Print initialization success message
fn print_init_success(binary_path: &Path, settings_path: &Path) -> Result<()> {
    outro(format!(
        "{} Claude Code Personalities Initialized Successfully!",
        ICON_CHECK.green()
    ))?;

    println!();

    println!(
        "{} {}",
        "Binary:  ".bold().color(colored::Color::TrueColor {
            r: 171,
            g: 232,
            b: 239
        }),
        binary_path.display()
    );
    println!(
        "{} {}",
        "Settings:".bold().color(colored::Color::TrueColor {
            r: 171,
            g: 232,
            b: 239
        }),
        settings_path.display()
    );
    println!();

    println!(
        "{}",
        "Next Steps:".bold().color(colored::Color::TrueColor {
            r: 171,
            g: 232,
            b: 239
        })
    );
    println!("  1. Your statusline will now show dynamic personalities!");
    println!(
        "  2. Customize appearance with: {}",
        "claude-code-personalities config".white()
    );
    println!();

    println!(
        "{}",
        "Available Commands:"
            .bold()
            .color(colored::Color::TrueColor {
                r: 171,
                g: 232,
                b: 239
            })
    );
    println!(
        "  {:35} {}",
        "claude-code-personalities status".white(),
        "- Check installation status".bright_black()
    );
    println!(
        "  {:35} {}",
        "claude-code-personalities config".white(),
        "- Customize appearance".bright_black()
    );
    println!(
        "  {:35} {}",
        "claude-code-personalities check-update".white(),
        "- Check for updates".bright_black()
    );
    println!(
        "  {:35} {}",
        "claude-code-personalities help".white(),
        "- View all commands".bright_black()
    );

    Ok(())
}

/// Find existing claude-code-personalities binary installation.
///
/// This function searches for an existing binary installation in order of preference:
/// 1. PATH (using `which` command) - highest priority since it's accessible
/// 2. ~/.local/bin/claude-code-personalities - standard user binary location
/// 3. ~/.claude/claude-code-personalities - legacy installation location
///
/// # Returns
/// `Some(PathBuf)` if an existing binary is found, `None` if no binary exists.
///
/// # Errors
/// Returns an error if command execution fails or directory traversal fails
async fn find_existing_binary() -> Result<Option<PathBuf>> {
    // 1. Check PATH using `which` command (highest priority)
    if let Ok(output) = tokio::process::Command::new("which")
        .arg("claude-code-personalities")
        .output()
        .await
        && output.status.success()
    {
        let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path_str.is_empty() {
            let path_binary = PathBuf::from(path_str);
            if path_binary.exists() {
                return Ok(Some(path_binary));
            }
        }
    }

    // 2. Check ~/.local/bin/claude-code-personalities
    if let Some(home_dir) = dirs::home_dir() {
        let local_bin = home_dir.join(".local/bin/claude-code-personalities");
        if local_bin.exists() {
            return Ok(Some(local_bin));
        }
    }

    // 3. Check ~/.claude/claude-code-personalities (legacy location)
    let claude_dir = get_claude_dir()?;
    let claude_binary = claude_dir.join("claude-code-personalities");
    if claude_binary.exists() {
        return Ok(Some(claude_binary));
    }

    Ok(None)
}

/// Helper functions for status output
fn print_info(message: &str) {
    println!("  {} {}", ICON_INFO.bright_black(), message.bright_black());
}

fn print_success(message: &str) {
    println!("  {} {}", ICON_CHECK.bright_black(), message.bright_black());
}

fn print_warning(message: &str) {
    println!("  {} {}", ICON_WARNING.yellow(), message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_options_default() {
        let options = InitOptions::default();
        assert!(!options.non_interactive);
        assert!(options.backup);
    }

    // Integration test for initialization flow (requires manual verification)
    #[tokio::test]
    #[ignore] // Ignored by default since it modifies system state
    async fn test_init_flow() {
        let options = InitOptions {
            non_interactive: true,
            backup: true,
        };

        // This test would modify the actual Claude settings
        // In a real scenario, we'd use a temporary directory
        // init_claude_code(options).await.unwrap();
    }
}
