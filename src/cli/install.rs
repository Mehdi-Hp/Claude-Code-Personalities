use anyhow::{Context, Result, anyhow};
use cliclack::{confirm, intro, outro};
use colored::Colorize;
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::cli::settings::{ClaudeSettings, get_claude_dir};
use crate::icons::{ICON_CHECK, ICON_INFO, ICON_WARNING};

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
    intro("Claude Code Personalities - Configuration")?;

    // Step 1: Check if Claude directory exists
    let claude_dir = get_claude_dir()?;
    if claude_dir.exists() {
        print_success(&format!("Found Claude directory: {}", claude_dir.display()));
    } else {
        if options.interactive {
            let create_dir = confirm("Claude directory not found. Create ~/.claude/ directory?")
                .initial_value(true)
                .interact()
                .with_context(|| "Failed to get user confirmation for directory creation")?;

            if !create_dir {
                return Err(anyhow!(
                    "Installation cancelled: Claude directory is required"
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

    // Step 2: Check for existing installations
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
        print_success("Ready for new installation");
    }

    // Step 3: Get current binary location
    let current_binary = get_current_binary_path()?;

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

    // Step 5: Load Claude settings first to check configuration
    let mut settings = ClaudeSettings::load()
        .await
        .with_context(|| "Failed to load Claude settings")?;

    // Step 6: Check current Claude Code configuration
    let config_summary = settings.get_configuration_summary();
    let settings_configured = config_summary.has_personality_statusline;

    if settings_configured && !options.force_reinstall {
        // Claude Code is already configured
        if options.interactive {
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
            print_warning("Already configured. Use --force to reconfigure.");
            return Ok(());
        }
    }

    // Step 7: Copy binary to target location
    copy_binary(&current_binary, &target_binary)
        .await
        .with_context(|| "Failed to install binary to target directory")?;
    print_success(&format!("Binary installed to: {}", target_binary.display()));

    // Step 8: Create backup if requested and settings exist
    if options.backup_existing && settings.settings_path.exists() {
        let backup_path = settings
            .create_backup()
            .await
            .with_context(|| "Failed to create settings backup")?;
        print_success(&format!("Backup created: {}", backup_path.display()));
    }

    // Step 9: Configure Claude Code settings (if needed)
    if !settings_configured || options.force_reinstall {
        // Step 10: Configure statusline
        settings
            .configure_statusline(&target_binary)
            .with_context(|| "Failed to configure statusline in settings")?;
        print_success("Statusline configured");

        // Step 11: Configure hooks
        settings
            .configure_hooks(&target_binary)
            .with_context(|| "Failed to configure hooks in settings")?;
        print_success("Hooks configured");

        // Step 12: Save settings
        settings
            .save()
            .await
            .with_context(|| "Failed to save updated Claude settings")?;
        print_success(&format!(
            "Settings saved to: {}",
            settings.settings_path.display()
        ));
    } else {
        print_success("Claude Code already configured");
    }

    // Step 13: Verify installation
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
    print_installation_success(&target_binary, &settings.settings_path)?;

    Ok(())
}

/// Copy the current binary to the target location
async fn copy_binary(source: &PathBuf, target: &PathBuf) -> Result<()> {
    // Validate source binary exists and has reasonable size
    let source_metadata = fs::metadata(source).await.with_context(|| {
        format!(
            "Failed to get metadata for source binary: {}",
            source.display()
        )
    })?;

    let source_size = source_metadata.len();
    if source_size == 0 {
        return Err(anyhow!(
            "Source binary is empty (0 bytes): {}. This indicates a corrupted installation. \
            Please download and install a fresh copy from GitHub releases.",
            source.display()
        ));
    }

    if source_size < 1024 {
        // Less than 1KB is suspicious for a Rust binary
        return Err(anyhow!(
            "Source binary is suspiciously small ({} bytes): {}. This may indicate corruption.",
            source_size,
            source.display()
        ));
    }

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

    // Verify the copy was successful
    let target_metadata = fs::metadata(target)
        .await
        .with_context(|| format!("Failed to verify copied binary: {}", target.display()))?;

    let target_size = target_metadata.len();
    if target_size != source_size {
        return Err(anyhow!(
            "Binary copy verification failed: source size {} bytes != target size {} bytes",
            source_size,
            target_size
        ));
    }

    // Make sure it's executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = target_metadata.permissions();
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
fn print_installation_success(binary_path: &Path, settings_path: &Path) -> Result<()> {
    outro(format!(
        "{} Claude Code Personalities Installed Successfully!",
        ICON_CHECK.green()
    ))?;

    println!();

    println!(
        "{} {}",
        "Installed to:".bold().color(colored::Color::TrueColor {
            r: 255,
            g: 165,
            b: 0
        }),
        binary_path.display()
    );
    println!(
        "{} {}",
        "Settings:   ".bold().color(colored::Color::TrueColor {
            r: 255,
            g: 165,
            b: 0
        }),
        settings_path.display()
    );
    println!();

    println!(
        "{}",
        "Next Steps:".bold().color(colored::Color::TrueColor {
            r: 255,
            g: 165,
            b: 0
        })
    );
    println!("  1. Restart any running Claude Code sessions");
    println!("  2. Your statusline will now show dynamic personalities!");
    println!(
        "  3. Customize settings with: {}",
        "claude-code-personalities config".white()
    );
    println!();

    println!(
        "{}",
        "What's Configured:"
            .bold()
            .color(colored::Color::TrueColor {
                r: 255,
                g: 165,
                b: 0
            })
    );
    println!(
        "  {} Dynamic statusline with personality faces",
        ICON_CHECK.green()
    );
    println!("  {} Activity tracking hooks", ICON_CHECK.green());
    println!("  {} Error state management", ICON_CHECK.green());
    println!("  {} Session cleanup", ICON_CHECK.green());
    println!();

    println!(
        "{}",
        "Available Commands:"
            .bold()
            .color(colored::Color::TrueColor {
                r: 255,
                g: 165,
                b: 0
            })
    );
    println!(
        "  {} Check installation status",
        "claude-code-personalities status".white()
    );
    println!(
        "  {} Customize appearance",
        "claude-code-personalities config".white()
    );
    println!(
        "  {} - Check for updates",
        "claude-code-personalities check-update".white()
    );
    println!(
        "  {} - View all commands",
        "claude-code-personalities help".white()
    );
    println!();

    println!(
        "{} Start a new Claude Code session to see your personalities in action!",
        "\u{f135}".yellow()
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
    println!("  {} {}", ICON_INFO.white(), message);
}

fn print_success(message: &str) {
    println!("  {} {}", ICON_CHECK.white(), message);
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

        // Create a source "binary" file (make it large enough to pass validation)
        let mut source_file = NamedTempFile::new().unwrap();
        let large_content = format!(
            "#!/bin/bash\necho 'test binary'\n# Padding to make file large enough:\n{}",
            "# ".repeat(500) // This creates ~1500 bytes total, well above our 1024 byte threshold
        );
        source_file.write_all(large_content.as_bytes()).unwrap();
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
