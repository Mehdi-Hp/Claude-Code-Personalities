use anyhow::{Context, Result, anyhow};
use colored::Colorize;
use inquire::Confirm;
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::cli::settings::{ClaudeSettings, get_claude_dir};
use crate::statusline::icons::{ICON_CHECK, ICON_INFO, ICON_WARNING};

pub struct InstallationOptions {
    pub interactive: bool,
    pub force_reinstall: bool,
    pub backup_existing: bool,
}

impl Default for InstallationOptions {
    fn default() -> Self {
        Self {
            interactive: true,
            force_reinstall: false,
            backup_existing: true,
        }
    }
}

/// Install Claude Code Personalities with the given options.
///
/// This function sets up the complete personalities system including:
/// - Binary installation to ~/.claude/
/// - Configuration of Claude Code settings.json
/// - Setup of statusline and hook system
///
/// # Errors
///
/// This function will return an error if:
/// - The Claude directory cannot be created or accessed
/// - The current binary cannot be copied to the target location
/// - Claude settings cannot be loaded or saved
/// - User cancels installation when in interactive mode
/// - File permissions cannot be set correctly
/// - Configuration validation fails
pub async fn install_personalities(options: InstallationOptions) -> Result<()> {
    println!(
        "{}",
        "Installing Claude Code Personalities...".bold().blue()
    );
    println!();

    // Step 1: Check if Claude directory exists
    let claude_dir = get_claude_dir()?;
    if claude_dir.exists() {
        print_success(&format!("Found Claude directory: {}", claude_dir.display()));
    } else {
        if options.interactive {
            let create_dir =
                Confirm::new("Claude directory not found. Create ~/.claude/ directory?")
                    .with_default(true)
                    .prompt()
                    .with_context(|| "Failed to get user confirmation for directory creation")?;

            if !create_dir {
                return Err(anyhow!(
                    "Installation cancelled: Claude directory is required"
                ));
            }
        }

        print_info("Creating Claude directory...");
        fs::create_dir_all(&claude_dir).await.with_context(|| {
            format!(
                "Failed to create Claude directory: {}",
                claude_dir.display()
            )
        })?;
        print_success(&format!("Created directory: {}", claude_dir.display()));
    }

    // Step 2: Check for existing installations
    print_info("Checking for existing installations...");
    let existing_binary = find_existing_binary().await?;

    if let Some(ref existing_path) = existing_binary {
        print_info(&format!(
            "Found existing installation: {}",
            existing_path.display()
        ));

        // If it's in ~/.claude (legacy location), warn the user
        if existing_path.starts_with(&claude_dir) {
            print_warning("Binary found in legacy location (~/.claude/)");
            print_info("Consider using 'migrate' command to move to ~/.local/bin for PATH access");
        }
    } else {
        print_info("No existing installation found");
    }

    // Step 3: Get current binary location
    let current_binary = get_current_binary_path()?;
    print_info(&format!("Current binary: {}", current_binary.display()));

    // Step 4: Determine target binary location - always use ~/.local/bin for new installations
    let target_binary = if let Some(existing_path) = &existing_binary {
        // Use existing location if found (unless we're forcing reinstall)
        existing_path.clone()
    } else {
        // Install to ~/.local/bin for PATH accessibility
        if let Some(home_dir) = dirs::home_dir() {
            let local_bin_dir = home_dir.join(".local/bin");
            // Ensure ~/.local/bin directory exists
            fs::create_dir_all(&local_bin_dir).await.with_context(|| {
                format!(
                    "Failed to create ~/.local/bin directory: {}",
                    local_bin_dir.display()
                )
            })?;
            local_bin_dir.join("claude-code-personalities")
        } else {
            return Err(anyhow!("Could not determine home directory"));
        }
    };

    // Step 5: Check if already installed
    if target_binary.exists() && !options.force_reinstall {
        if options.interactive {
            let reinstall =
                Confirm::new("Claude Code Personalities is already installed. Reinstall?")
                    .with_default(false)
                    .prompt()
                    .with_context(|| "Failed to get user confirmation for reinstallation")?;

            if !reinstall {
                print_info("Installation skipped.");
                return Ok(());
            }
        } else if !options.force_reinstall {
            print_warning("Already installed. Use --force to reinstall.");
            return Ok(());
        }
    }

    // Step 6: Copy binary to target location
    print_info("Installing binary...");
    copy_binary(&current_binary, &target_binary)
        .await
        .with_context(|| "Failed to install binary to target directory")?;
    print_success(&format!("Binary installed to: {}", target_binary.display()));

    // Step 7: Load Claude settings
    print_info("Loading Claude settings...");
    let mut settings = ClaudeSettings::load()
        .await
        .with_context(|| "Failed to load Claude settings")?;

    // Step 8: Create backup if requested and settings exist
    if options.backup_existing && settings.settings_path.exists() {
        print_info("Creating settings backup...");
        let backup_path = settings
            .create_backup()
            .await
            .with_context(|| "Failed to create settings backup")?;
        print_success(&format!("Backup created: {}", backup_path.display()));
    }

    // Step 9: Check current configuration
    let config_summary = settings.get_configuration_summary();
    if config_summary.has_personality_statusline {
        print_warning("Personalities are already configured in settings.json");
        if options.interactive {
            let reconfigure = Confirm::new("Reconfigure anyway?")
                .with_default(false)
                .prompt()
                .with_context(|| "Failed to get user confirmation for reconfiguration")?;

            if !reconfigure {
                print_info("Settings configuration skipped.");
                return Ok(());
            }
        }
    }

    // Step 10: Configure statusline
    print_info("Configuring statusline...");
    settings
        .configure_statusline(&target_binary)
        .with_context(|| "Failed to configure statusline in settings")?;
    print_success("Statusline configured");

    // Step 11: Configure hooks
    print_info("Configuring hooks...");
    settings
        .configure_hooks(&target_binary)
        .with_context(|| "Failed to configure hooks in settings")?;
    print_success("Hooks configured");

    // Step 12: Save settings
    print_info("Saving settings...");
    settings
        .save()
        .await
        .with_context(|| "Failed to save updated Claude settings")?;
    print_success(&format!(
        "Settings saved to: {}",
        settings.settings_path.display()
    ));

    // Step 13: Verify installation
    print_info("Verifying installation...");
    let final_summary = settings.get_configuration_summary();
    if final_summary.is_fully_configured() {
        print_success("Installation verified successfully");
    } else {
        print_warning("Installation completed but verification failed");
        return Err(anyhow!(
            "Installation verification failed - some components may not be configured correctly"
        ));
    }

    // Step 14: Show success message
    println!();
    print_installation_success(&target_binary, &settings.settings_path);

    Ok(())
}

/// Copy the current binary to the target location
async fn copy_binary(source: &PathBuf, target: &PathBuf) -> Result<()> {
    // Ensure target directory exists
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .await
            .with_context(|| format!("Failed to create target directory: {}", parent.display()))?;
    }

    // Copy the binary
    fs::copy(source, target).await.with_context(|| {
        format!(
            "Failed to copy binary from {} to {}",
            source.display(),
            target.display()
        )
    })?;

    // Make sure it's executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(target)
            .await
            .with_context(|| format!("Failed to get metadata for {}", target.display()))?
            .permissions();
        permissions.set_mode(0o755); // rwxr-xr-x
        fs::set_permissions(target, permissions)
            .await
            .with_context(|| {
                format!(
                    "Failed to set executable permissions on {}",
                    target.display()
                )
            })?;
    }

    Ok(())
}

/// Get the path to the currently running binary
fn get_current_binary_path() -> Result<PathBuf> {
    std::env::current_exe().with_context(|| "Failed to determine current binary path")
}

/// Print installation success message
fn print_installation_success(binary_path: &Path, settings_path: &Path) {
    println!(
        "{}",
        "╔══════════════════════════════════════════════════════════╗"
            .bold()
            .green()
    );
    println!(
        "{}",
        "║                                                          ║"
            .bold()
            .green()
    );
    println!(
        "{} {} {}",
        "║".bold().green(),
        "🎉 Claude Code Personalities Installed Successfully! 🎉"
            .bold()
            .white(),
        "║".bold().green()
    );
    println!(
        "{}",
        "║                                                          ║"
            .bold()
            .green()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════╝"
            .bold()
            .green()
    );
    println!();

    println!("{} {}", "📍 Installed to:".bold(), binary_path.display());
    println!("{} {}", "⚙️  Settings:   ".bold(), settings_path.display());
    println!();

    println!("{}", "Next Steps:".bold().cyan());
    println!("  1. Restart any running Claude Code sessions");
    println!("  2. Your statusline will now show dynamic personalities!");
    println!(
        "  3. Customize settings with: {}",
        "claude-code-personalities config".cyan()
    );
    println!();

    println!("{}", "What's Configured:".bold().yellow());
    println!(
        "  {} Dynamic statusline with personality faces",
        ICON_CHECK.green()
    );
    println!("  {} Activity tracking hooks", ICON_CHECK.green());
    println!("  {} Error state management", ICON_CHECK.green());
    println!("  {} Session cleanup", ICON_CHECK.green());
    println!();

    println!("{}", "Available Commands:".bold().magenta());
    println!(
        "  {} Check installation status",
        "claude-code-personalities status".cyan()
    );
    println!(
        "  {} Customize appearance",
        "claude-code-personalities config".cyan()
    );
    println!(
        "  {} Check for updates",
        "claude-code-personalities check-update".cyan()
    );
    println!(
        "  {} View all commands",
        "claude-code-personalities help".cyan()
    );
    println!();

    println!(
        "{} Start a new Claude Code session to see your personalities in action!",
        "🚀".yellow()
    );
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
    {
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path_str.is_empty() {
                let path_binary = PathBuf::from(path_str);
                if path_binary.exists() {
                    return Ok(Some(path_binary));
                }
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
    println!("  {} {}", ICON_INFO.cyan(), message);
}

fn print_success(message: &str) {
    println!("  {} {}", ICON_CHECK.green(), message);
}

fn print_warning(message: &str) {
    println!("  {} {}", ICON_WARNING.yellow(), message);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    #[tokio::test]
    async fn test_copy_binary() {
        let temp_dir = TempDir::new().unwrap();

        // Create a source "binary" file
        let mut source_file = NamedTempFile::new().unwrap();
        source_file
            .write_all(b"#!/bin/bash\necho 'test binary'")
            .unwrap();
        source_file.flush().unwrap();

        let source_path = source_file.path().to_path_buf();
        let target_path = temp_dir.path().join("target_binary");

        // Test copying
        copy_binary(&source_path, &target_path).await.unwrap();

        // Verify file was copied
        assert!(target_path.exists());
        let copied_content = fs::read(&target_path).await.unwrap();
        let original_content = fs::read(&source_path).await.unwrap();
        assert_eq!(copied_content, original_content);

        // Verify permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&target_path).await.unwrap();
            let permissions = metadata.permissions();
            assert_eq!(permissions.mode() & 0o777, 0o755);
        }
    }

    #[test]
    fn test_get_current_binary_path() {
        let path = get_current_binary_path().unwrap();
        assert!(path.is_absolute());
        // The path should end with the test binary name
        assert!(path.to_string_lossy().contains("claude"));
    }

    #[test]
    fn test_installation_options_default() {
        let options = InstallationOptions::default();
        assert!(options.interactive);
        assert!(!options.force_reinstall);
        assert!(options.backup_existing);
    }

    // Integration test for installation flow (requires manual verification)
    #[tokio::test]
    #[ignore] // Ignored by default since it modifies system state
    async fn test_installation_flow() {
        let _options = InstallationOptions {
            interactive: false,
            force_reinstall: true,
            backup_existing: true,
        };

        // This test would modify the actual Claude settings
        // In a real scenario, we'd use a temporary directory
        // install_personalities(options).await.unwrap();
    }
}
