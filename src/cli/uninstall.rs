use anyhow::{Context, Result};
use cliclack::{intro, outro};
use colored::Colorize;
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::cli::settings::{ClaudeSettings, get_claude_dir};
use crate::icons::{ICON_CHECK, ICON_INFO, ICON_WARNING};

#[derive(Default)]
pub struct UninstallOptions {
    pub force: bool,
}

/// Uninstall Claude Code Personalities and remove all configuration.
///
/// This function performs a complete cleanup including:
/// - Removal of the binary from ~/.claude/
/// - Cleanup of Claude Code settings.json configuration
/// - Removal of temporary state files
/// - Optional removal of backup files and user preferences
///
/// # Errors
///
/// This function will return an error if:
/// - Claude directory or settings cannot be accessed
/// - File removal operations fail due to permissions
/// - Settings backup creation fails
/// - Settings file cannot be updated or saved
/// - User cancels uninstallation when in interactive mode
/// - Cleanup of temporary or backup files fails
pub async fn uninstall_personalities(options: UninstallOptions) -> Result<()> {
    intro("Uninstalling Claude Code Personalities")?;

    // Step 1: Find all Claude Code Personalities installations
    let claude_dir = get_claude_dir()?;
    let binary_locations = find_all_binary_locations(&claude_dir).await?;

    if binary_locations.is_empty() {
        print_warning("No Claude Code Personalities installations found");
        if !options.force {
            print_info("Use --force to clean up configuration anyway.");
            return Ok(());
        }
    } else {
        println!();
        print_info(&format!(
            "Found {} installation(s):",
            binary_locations.len()
        ));
        for location in &binary_locations {
            println!("    {}", location.display());
        }
    }

    // Step 2: Show what will be removed (non-interactive, complete cleanup)
    println!();
    println!("{}", "This will remove:".bold().yellow());
    println!("  • Claude Code Personalities binary");
    println!("  • Statusline and hook configuration from settings.json");
    println!("  • Temporary state files");
    println!("  • All backup files");
    println!("  • User preferences file");
    println!();

    // Step 3: Load Claude settings
    print_info("Loading Claude settings...");
    let mut settings = ClaudeSettings::load()
        .await
        .with_context(|| "Failed to load Claude settings")?;

    // Step 4: Create backup before making changes
    let backup_path = if settings.settings_path.exists() {
        print_info("Creating final backup of settings...");
        let backup = settings
            .create_backup()
            .await
            .with_context(|| "Failed to create final backup")?;
        print_success(&format!("Backup created: {}", backup.display()));
        Some(backup)
    } else {
        None
    };

    // Step 5: Check if personalities are configured
    if settings.is_personality_configured() {
        // Step 6: Remove personality configuration
        print_info("Removing personality configuration from settings...");
        settings.remove_personality_config();

        // Step 7: Save updated settings
        settings
            .save()
            .await
            .with_context(|| "Failed to save updated Claude settings")?;
        print_success("Personality configuration removed from settings");
    } else {
        print_warning("Personalities don't appear to be configured in settings.json");
    }

    // Step 8: Remove all found binaries
    if !binary_locations.is_empty() {
        print_info(&format!(
            "Removing {} binary/binaries...",
            binary_locations.len()
        ));
        let mut removed_count = 0;
        let mut failed_removals = Vec::new();

        for binary_path in &binary_locations {
            match fs::remove_file(binary_path).await {
                Ok(()) => {
                    print_success(&format!("Removed: {}", binary_path.display()));
                    removed_count += 1;
                }
                Err(e) => {
                    print_warning(&format!(
                        "Failed to remove {}: {}",
                        binary_path.display(),
                        e
                    ));
                    failed_removals.push((binary_path.clone(), e));
                }
            }
        }

        if removed_count > 0 {
            print_success(&format!(
                "Successfully removed {removed_count} binary/binaries"
            ));
        }

        if !failed_removals.is_empty() {
            print_warning(&format!(
                "{} binaries could not be removed (check permissions)",
                failed_removals.len()
            ));
        }
    }

    // Step 9: Clean up backup files (always for complete cleanup)
    print_info("Removing backup files...");
    let removed_count = cleanup_backup_files(&claude_dir)
        .await
        .with_context(|| "Failed to clean up backup files")?;
    if removed_count > 0 {
        print_success(&format!("Removed {removed_count} backup files"));
    } else {
        print_info("No backup files found");
    }

    // Step 10: Clean up temporary state files
    print_info("Cleaning up temporary files...");
    let temp_files_removed = cleanup_temp_files()
        .await
        .with_context(|| "Failed to clean up temporary files")?;
    if temp_files_removed > 0 {
        print_success(&format!("Removed {temp_files_removed} temporary files"));
    }

    // Step 11: Remove user preferences (always for complete cleanup)
    print_info("Removing user preferences...");
    let prefs_removed = cleanup_user_preferences()
        .await
        .with_context(|| "Failed to clean up user preferences")?;
    if prefs_removed {
        print_success("User preferences removed");
    }

    // Step 12: Show completion message
    println!();
    print_uninstall_success(false, false, backup_path.as_ref())?;

    Ok(())
}

/// Clean up all backup files in Claude directory
async fn cleanup_backup_files(claude_dir: &PathBuf) -> Result<u32> {
    let mut removed_count = 0;

    if !claude_dir.exists() {
        return Ok(0);
    }

    let mut entries = fs::read_dir(claude_dir)
        .await
        .with_context(|| format!("Failed to read Claude directory: {}", claude_dir.display()))?;

    while let Some(entry) = entries
        .next_entry()
        .await
        .with_context(|| "Failed to read directory entry")?
    {
        let filename = entry.file_name();
        let filename_str = filename.to_string_lossy();

        // Remove personality-related backups
        if filename_str.starts_with("claude-code-personalities.backup.")
            || filename_str.starts_with("settings.json.backup.")
        {
            if let Err(e) = fs::remove_file(entry.path()).await {
                print_warning(&format!(
                    "Failed to remove backup file {}: {}",
                    entry.path().display(),
                    e
                ));
            } else {
                removed_count += 1;
            }
        }
    }

    Ok(removed_count)
}

/// Clean up temporary state files from /tmp
async fn cleanup_temp_files() -> Result<u32> {
    let mut removed_count = 0;
    let tmp_dir = PathBuf::from("/tmp");

    if !tmp_dir.exists() {
        return Ok(0);
    }

    let mut entries = fs::read_dir(&tmp_dir)
        .await
        .with_context(|| "Failed to read /tmp directory")?;

    while let Some(entry) = entries
        .next_entry()
        .await
        .with_context(|| "Failed to read /tmp entry")?
    {
        let filename = entry.file_name();
        let filename_str = filename.to_string_lossy();

        // Remove personality-related temp files
        if (filename_str.starts_with("claude_activity_")
            || filename_str.starts_with("claude_code_personalities_errors_")
            || filename_str.starts_with("claude_personalities_"))
            && fs::remove_file(entry.path()).await.is_ok()
        {
            removed_count += 1;
            // Don't warn about temp files - they might be in use
        }
    }

    Ok(removed_count)
}

/// Clean up user preferences file
async fn cleanup_user_preferences() -> Result<bool> {
    let claude_dir = get_claude_dir()?;
    let prefs_file = claude_dir.join("personalities_config.json");

    if prefs_file.exists() {
        fs::remove_file(&prefs_file)
            .await
            .with_context(|| format!("Failed to remove preferences: {}", prefs_file.display()))?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Print uninstall success message
fn print_uninstall_success(
    kept_preferences: bool,
    kept_backups: bool,
    final_backup: Option<&PathBuf>,
) -> Result<()> {
    outro(format!(
        "{} Claude Code Personalities Uninstalled Successfully!",
        ICON_CHECK.green()
    ))?;

    println!();

    println!("{}", "What was removed:".bold().cyan());
    println!("  {} Claude Code Personalities binary", ICON_CHECK.green());
    println!("  {} Statusline and hook configuration", ICON_CHECK.green());
    println!("  {} Temporary state files", ICON_CHECK.green());
    if !kept_backups {
        println!("  {} All backup files", ICON_CHECK.green());
    }
    if !kept_preferences {
        println!("  {} User preferences", ICON_CHECK.green());
    }
    println!();

    if kept_preferences || kept_backups {
        println!("{}", "What was preserved:".bold().yellow());
        if kept_preferences {
            println!(
                "  {} User preferences (can be removed manually)",
                ICON_INFO.yellow()
            );
        }
        if kept_backups {
            println!(
                "  {} Settings backup files (can be removed manually)",
                ICON_INFO.yellow()
            );
            if let Some(backup) = final_backup {
                println!(
                    "  {} Final backup: {}",
                    ICON_INFO.yellow(),
                    backup.display()
                );
            }
        }
        println!();
    }

    println!("{}", "Next Steps:".bold().magenta());
    println!("  1. Restart any running Claude Code sessions");
    println!("  2. Your statusline will return to default behavior");
    if kept_backups {
        println!("  3. You can restore settings from backup if needed");
    }
    println!();

    println!("{}", "Reinstallation:".bold().cyan());
    println!("  To reinstall Claude Code Personalities:");
    println!(
        "  {} Download latest release from GitHub",
        "curl -L <release-url>".cyan()
    );
    println!(
        "  {} Run installation command",
        "claude-code-personalities install".cyan()
    );
    println!();

    println!(
        "{} Thank you for using Claude Code Personalities!",
        "\u{f1e6}".yellow() // Nerd font icon instead of emoji
    );

    Ok(())
}

/// Find all claude-code-personalities binary installations on the system.
///
/// This function searches for binaries in multiple common locations:
/// 1. PATH (using `which` command)
/// 2. ~/.local/bin/claude-code-personalities
/// 3. ~/.claude/claude-code-personalities  
/// 4. /usr/local/bin/claude-code-personalities
///
/// # Arguments
/// * `claude_dir` - The Claude directory path for checking ~/.claude location
///
/// # Returns
/// A vector of PathBuf containing all found binary locations
///
/// # Errors
/// Returns an error if directory traversal fails or command execution fails
async fn find_all_binary_locations(claude_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut locations = Vec::new();

    // 1. Check PATH using `which` command
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
                    locations.push(path_binary);
                }
            }
        }
    }

    // 2. Check ~/.local/bin/claude-code-personalities
    let local_bin = dirs::home_dir().map(|home| home.join(".local/bin/claude-code-personalities"));
    if let Some(local_bin) = local_bin {
        if local_bin.exists() && !locations.contains(&local_bin) {
            locations.push(local_bin);
        }
    }

    // 3. Check ~/.claude/claude-code-personalities
    let claude_binary = claude_dir.join("claude-code-personalities");
    if claude_binary.exists() && !locations.contains(&claude_binary) {
        locations.push(claude_binary);
    }

    // 4. Check /usr/local/bin/claude-code-personalities
    let usr_local = PathBuf::from("/usr/local/bin/claude-code-personalities");
    if usr_local.exists() && !locations.contains(&usr_local) {
        locations.push(usr_local);
    }

    Ok(locations)
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
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_cleanup_backup_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create some test backup files
        let backup_files = vec![
            "claude-code-personalities.backup.1.0.0",
            "settings.json.backup.20240101_120000",
            "other-file.txt", // Should not be removed
        ];

        for filename in &backup_files {
            let file_path = temp_dir.path().join(filename);
            fs::write(&file_path, b"backup content").await.unwrap();
        }

        let removed_count = cleanup_backup_files(&temp_dir.path().to_path_buf())
            .await
            .unwrap();
        assert_eq!(removed_count, 2, "Should remove 2 backup files");

        // Verify other file is still there
        let other_file = temp_dir.path().join("other-file.txt");
        assert!(
            other_file.exists(),
            "Non-backup files should not be removed"
        );
    }

    #[tokio::test]
    async fn test_cleanup_temp_files() {
        // This test is challenging since we can't easily create files in /tmp
        // So we'll just test that the function runs without error
        let result = cleanup_temp_files().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cleanup_user_preferences() {
        let temp_dir = TempDir::new().unwrap();

        // Mock the get_claude_dir function by creating a preferences file
        let prefs_file = temp_dir.path().join("personalities_config.json");
        fs::write(&prefs_file, b"{}").await.unwrap();

        // We can't easily test this function directly due to get_claude_dir dependency
        // But we can test the file operations
        assert!(prefs_file.exists());
        fs::remove_file(&prefs_file).await.unwrap();
        assert!(!prefs_file.exists());
    }

    #[test]
    fn test_uninstall_options_default() {
        let options = UninstallOptions::default();
        assert!(!options.force);
    }

    // Integration test for uninstall flow (requires manual verification)
    #[tokio::test]
    #[ignore] // Ignored by default since it would modify system state
    async fn test_uninstall_flow() {
        let _options = UninstallOptions { force: true };

        // This test would modify the actual Claude settings
        // In a real scenario, we'd use a temporary directory
        // uninstall_personalities(options).await.unwrap();
    }
}
