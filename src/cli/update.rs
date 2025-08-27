use anyhow::{Context, Result, anyhow};
use colored::Colorize;
use inquire::Confirm;
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::cli::settings::get_claude_dir;
use crate::icons::{ICON_CHECK, ICON_INFO, ICON_WARNING};
use crate::platform::Platform;
use crate::version::{
    CURRENT_VERSION, VersionManager, format_changelog, format_version_comparison,
};

pub struct UpdateOptions {
    pub force: bool,
    pub interactive: bool,
    pub include_prereleases: bool,
}

impl Default for UpdateOptions {
    fn default() -> Self {
        Self {
            force: false,
            interactive: true,
            include_prereleases: false,
        }
    }
}

/// Update Claude Code Personalities to the latest version.
///
/// This function performs a complete update process including:
/// - Checking for available updates from GitHub releases
/// - Downloading the appropriate binary for the current platform
/// - Replacing the existing binary with backup creation
/// - Verification of the updated installation
///
/// # Errors
///
/// This function will return an error if:
/// - Version manager cannot be initialized
/// - GitHub API requests fail or return invalid data
/// - Platform detection fails or platform is unsupported
/// - Binary download fails or file verification fails
/// - File system operations fail (backup, replace, permissions)
/// - User cancels update when in interactive mode
/// - Binary verification fails after installation
pub async fn update_personalities(options: UpdateOptions) -> Result<()> {
    println!(
        "{}",
        "Checking for Claude Code Personalities updates..."
            .bold()
            .blue()
    );
    println!();

    // Step 1: Check for updates and get release info
    let version_manager =
        VersionManager::new().with_context(|| "Failed to initialize version manager")?;

    let latest_release = check_and_get_release(&version_manager, &options).await?;

    // Step 2: Show update information and get user confirmation
    let should_proceed = show_update_info_and_confirm(&latest_release, &options).await?;
    if !should_proceed {
        return Ok(());
    }

    // Step 3: Perform the actual update
    perform_update(&version_manager, &latest_release).await?;

    print_success("Update completed successfully!");
    println!();

    Ok(())
}

async fn check_and_get_release(
    version_manager: &VersionManager,
    options: &UpdateOptions,
) -> Result<crate::version::GitHubRelease> {
    print_info("Checking latest version...");
    let update_info = version_manager
        .check_for_update()
        .await
        .with_context(|| "Failed to check for updates")?;

    if let Some(release) = update_info {
        if release.prerelease && !options.include_prereleases {
            print_info(
                "Latest release is a pre-release. Use --include-prereleases to update to it.",
            );
            return Err(anyhow!("Pre-release version available, but not included"));
        }
        Ok(release)
    } else {
        print_success("You are already running the latest version!");
        if options.force {
            print_info("Force update requested, continuing anyway...");
            version_manager
                .get_latest_release()
                .await
                .with_context(|| "Failed to get latest release for force update")
        } else {
            Err(anyhow!("Already running latest version"))
        }
    }
}

async fn show_update_info_and_confirm(
    latest_release: &crate::version::GitHubRelease,
    options: &UpdateOptions,
) -> Result<bool> {
    let latest_version = latest_release
        .tag_name
        .strip_prefix('v')
        .unwrap_or(&latest_release.tag_name);
    let comparison = format_version_comparison(CURRENT_VERSION, latest_version);
    println!();
    println!(
        "{} {}",
        format!("{}Update Available:", "\u{f135} ").bold().green(),
        comparison
    );

    if let Some(name) = &latest_release.name {
        println!("{} {}", format!("{}Release:", "\u{f044} ").bold(), name);
    }

    let changelog = format_changelog(latest_release);
    if changelog != "No changelog available" {
        println!();
        println!("{}", "📝 What's New:".bold().yellow());
        for line in changelog.lines().take(5) {
            println!("   {line}");
        }
        if changelog.lines().count() > 5 {
            println!("   ... (view full changelog on GitHub)");
        }
    }

    // Get user confirmation if interactive
    if options.interactive {
        println!();
        let should_update = Confirm::new("Do you want to update now?")
            .with_default(true)
            .prompt()
            .with_context(|| "Failed to get user confirmation for update")?;

        if !should_update {
            print_info("Update cancelled.");
            return Ok(false);
        }
    }

    Ok(true)
}

async fn perform_update(
    version_manager: &VersionManager,
    latest_release: &crate::version::GitHubRelease,
) -> Result<()> {
    // Step 1: Detect platform and find appropriate asset
    print_info("Detecting platform...");
    let platform = Platform::detect().with_context(|| "Failed to detect current platform")?;

    if !platform.is_supported() {
        return Err(anyhow!("Unsupported platform: {}", platform.description()));
    }

    print_success(&format!("Platform: {}", platform.description()));

    let asset = version_manager
        .find_platform_asset(latest_release, &platform.target)
        .ok_or_else(|| {
            anyhow!(
                "No binary available for platform: {}",
                platform.description()
            )
        })?;

    // Step 2: Set up paths and verify current installation
    let paths = setup_update_paths()?;

    // Step 3: Download and verify the new binary
    download_and_verify_binary(version_manager, asset, &paths.temp_binary).await?;

    // Step 4: Backup and replace binary
    backup_and_replace_binary(&paths).await?;

    // Step 5: Verify installation and cleanup
    let latest_version = latest_release
        .tag_name
        .strip_prefix('v')
        .unwrap_or(&latest_release.tag_name);
    verify_installation_and_cleanup(&paths, latest_version).await?;

    Ok(())
}

struct UpdatePaths {
    current_binary: PathBuf,
    backup_binary: PathBuf,
    temp_binary: PathBuf,
    claude_dir: PathBuf,
}

fn setup_update_paths() -> Result<UpdatePaths> {
    let claude_dir = get_claude_dir().with_context(|| "Failed to determine Claude directory")?;
    let current_binary = claude_dir.join("claude-code-personalities");
    let backup_binary = claude_dir.join(format!(
        "claude-code-personalities.backup.{CURRENT_VERSION}"
    ));
    let temp_binary = claude_dir.join("claude-code-personalities.tmp");

    if !current_binary.exists() {
        return Err(anyhow!(
            "Claude Code Personalities is not installed in the expected location: {}",
            current_binary.display()
        ));
    }

    Ok(UpdatePaths {
        current_binary,
        backup_binary,
        temp_binary,
        claude_dir,
    })
}

async fn download_and_verify_binary(
    version_manager: &VersionManager,
    asset: &crate::version::GitHubAsset,
    temp_binary: &Path,
) -> Result<()> {
    let size_mb = asset.size / 1_048_576; // Convert to MB using integer division
    if size_mb > 0 {
        print_info(&format!("Downloading {} ({} MB)...", asset.name, size_mb));
    } else {
        let size_kilobytes = asset.size / 1024; // Show in KB for smaller files
        print_info(&format!(
            "Downloading {} ({} KB)...",
            asset.name, size_kilobytes
        ));
    }
    version_manager
        .download_asset(asset, temp_binary)
        .await
        .with_context(|| "Failed to download update")?;
    print_success("Download completed");

    print_info("Verifying download...");
    verify_binary(&temp_binary.to_path_buf())
        .await
        .with_context(|| "Downloaded binary verification failed")?;
    print_success("Binary verified");

    Ok(())
}

async fn backup_and_replace_binary(paths: &UpdatePaths) -> Result<()> {
    print_info("Creating backup of current version...");
    fs::copy(&paths.current_binary, &paths.backup_binary)
        .await
        .with_context(|| format!("Failed to create backup: {}", paths.backup_binary.display()))?;
    print_success(&format!(
        "Backup created: {}",
        paths.backup_binary.display()
    ));

    print_info("Installing updated binary...");
    fs::rename(&paths.temp_binary, &paths.current_binary)
        .await
        .with_context(|| "Failed to replace current binary with updated version")?;

    // Set executable permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&paths.current_binary)
            .await
            .with_context(|| "Failed to get metadata for updated binary")?
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&paths.current_binary, permissions)
            .await
            .with_context(|| "Failed to set executable permissions on updated binary")?;
    }

    print_success("Binary updated successfully");
    Ok(())
}

async fn verify_installation_and_cleanup(paths: &UpdatePaths, latest_version: &str) -> Result<()> {
    print_info("Verifying installation...");
    let new_version = get_binary_version(&paths.current_binary)
        .await
        .with_context(|| "Failed to verify new binary version")?;

    if new_version.trim_start_matches('v') == latest_version {
        print_success(&format!("Update verified: now running v{latest_version}"));
    } else {
        print_warning(&format!(
            "Version mismatch: expected v{latest_version}, got {new_version}"
        ));
    }

    cleanup_old_backups(&paths.claude_dir)
        .await
        .with_context(|| "Failed to clean up old backups")?;

    println!();
    print_update_success(CURRENT_VERSION, latest_version, &paths.current_binary);

    Ok(())
}

/// Verify that a binary is valid and executable
async fn verify_binary(binary_path: &PathBuf) -> Result<()> {
    // Check that file exists and has content
    let metadata = fs::metadata(binary_path)
        .await
        .with_context(|| format!("Failed to get metadata for {}", binary_path.display()))?;

    if metadata.len() == 0 {
        return Err(anyhow!("Downloaded binary is empty"));
    }

    if metadata.len() < 1024 {
        return Err(anyhow!("Downloaded binary is suspiciously small"));
    }

    // On Unix, check if it looks like an executable
    #[cfg(unix)]
    {
        let content = fs::read(binary_path)
            .await
            .with_context(|| "Failed to read binary for verification")?;

        // Check for ELF magic number (Linux) or Mach-O magic (macOS)
        if content.len() >= 4 {
            let magic = &content[0..4];
            let is_elf = magic == [0x7f, 0x45, 0x4c, 0x46]; // ELF
            let is_macho = magic == [0xfe, 0xed, 0xfa, 0xce] || magic == [0xfe, 0xed, 0xfa, 0xcf]; // Mach-O
            let is_fat_macho = magic == [0xca, 0xfe, 0xba, 0xbe]; // Fat Mach-O

            if !is_elf && !is_macho && !is_fat_macho {
                return Err(anyhow!(
                    "Downloaded file does not appear to be a valid executable"
                ));
            }
        }
    }

    Ok(())
}

/// Get version from a binary by executing it
async fn get_binary_version(binary_path: &PathBuf) -> Result<String> {
    let output = tokio::process::Command::new(binary_path)
        .arg("--version")
        .output()
        .await
        .with_context(|| format!("Failed to execute binary: {}", binary_path.display()))?;

    if !output.status.success() {
        return Err(anyhow!("Binary version check failed"));
    }

    let version_str = String::from_utf8(output.stdout)
        .with_context(|| "Binary version output is not valid UTF-8")?;

    // Extract version from output like "claude-code-personalities 1.2.3"
    version_str
        .split_whitespace()
        .last()
        .ok_or_else(|| anyhow!("Could not parse version from binary output"))
        .map(std::string::ToString::to_string)
}

/// Clean up old backup files, keeping only the most recent 3
async fn cleanup_old_backups(claude_dir: &PathBuf) -> Result<()> {
    let mut backup_files = Vec::new();

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

        if filename_str.starts_with("claude-code-personalities.backup.") {
            if let Ok(metadata) = entry.metadata().await {
                backup_files.push((
                    entry.path(),
                    metadata
                        .modified()
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                ));
            }
        }
    }

    // Sort by modification time (newest first)
    backup_files.sort_by(|a, b| b.1.cmp(&a.1));

    // Remove all but the newest 3 backups
    for (path, _) in backup_files.iter().skip(3) {
        if let Err(e) = fs::remove_file(path).await {
            print_warning(&format!(
                "Failed to remove old backup {}: {}",
                path.display(),
                e
            ));
        }
    }

    Ok(())
}

/// Print update success message
fn print_update_success(old_version: &str, new_version: &str, binary_path: &Path) {
    println!(
        "{}",
        "╔═══════════════════════════════════════════════════════════╗"
            .bold()
            .green()
    );
    println!(
        "{}",
        "║                                                           ║"
            .bold()
            .green()
    );
    println!(
        "{} {} {}",
        "║".bold().green(),
        "🎉 Claude Code Personalities Updated Successfully! 🎉"
            .bold()
            .white(),
        "║".bold().green()
    );
    println!(
        "{}",
        "║                                                           ║"
            .bold()
            .green()
    );
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════════╝"
            .bold()
            .green()
    );
    println!();

    let version_change = format!("v{old_version} → v{new_version}");
    println!(
        "{} {}",
        format!("{}Version:", "\u{f135} ").bold(),
        version_change.green()
    );
    println!(
        "{} {}",
        format!("{}Location:", "\u{f07b} ").bold(),
        binary_path.display()
    );
    println!();

    println!("{}", "What's Next:".bold().cyan());
    println!("  1. Restart any running Claude Code sessions to use the new version");
    println!("  2. Your personalities will continue working with new features!");
    println!(
        "  3. Check status with: {}",
        "claude-code-personalities status".cyan()
    );
    println!();

    println!("{}", "Need Help?".bold().magenta());
    println!(
        "  {} View configuration options",
        "claude-code-personalities config".cyan()
    );
    println!(
        "  {} Get help and usage info",
        "claude-code-personalities help".cyan()
    );
    println!(
        "  {} Report issues on GitHub",
        "https://github.com/Mehdi-Hp/claude-code-personalities/issues".cyan()
    );
    println!();

    println!(
        "{} Your Claude Code personalities are now up to date!",
        "\u{f135}".yellow()
    );
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
    async fn test_verify_binary_valid() {
        let mut temp_file = NamedTempFile::new().unwrap();

        // Write ELF magic number + some content
        temp_file.write_all(&[0x7f, 0x45, 0x4c, 0x46]).unwrap(); // ELF magic
        temp_file.write_all(&vec![0; 2000]).unwrap(); // Pad to reasonable size
        temp_file.flush().unwrap();

        let path = temp_file.path().to_path_buf();
        verify_binary(&path).await.unwrap();
    }

    #[tokio::test]
    async fn test_verify_binary_empty() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        let result = verify_binary(&path).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[tokio::test]
    async fn test_verify_binary_too_small() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"small").unwrap();
        temp_file.flush().unwrap();

        let path = temp_file.path().to_path_buf();
        let result = verify_binary(&path).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("small"));
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_verify_binary_invalid_format() {
        let mut temp_file = NamedTempFile::new().unwrap();

        // Write non-executable content
        temp_file
            .write_all(b"This is not an executable file")
            .unwrap();
        temp_file.write_all(&vec![0; 2000]).unwrap(); // Pad to reasonable size
        temp_file.flush().unwrap();

        let path = temp_file.path().to_path_buf();
        let result = verify_binary(&path).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("valid executable"));
    }

    #[tokio::test]
    async fn test_cleanup_old_backups() {
        let temp_dir = TempDir::new().unwrap();

        // Create several backup files
        for i in 0..5 {
            let backup_name = format!("claude-code-personalities.backup.1.{i}.0");
            let backup_path = temp_dir.path().join(backup_name);
            fs::write(&backup_path, b"backup content").await.unwrap();

            // Add small delay to ensure different modification times
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        // Also create a non-backup file that shouldn't be touched
        let other_file = temp_dir.path().join("other-file.txt");
        fs::write(&other_file, b"other content").await.unwrap();

        // Run cleanup
        cleanup_old_backups(&temp_dir.path().to_path_buf())
            .await
            .unwrap();

        // Count remaining backup files
        let mut entries = fs::read_dir(temp_dir.path()).await.unwrap();
        let mut backup_count = 0;
        let mut other_files = 0;

        while let Some(entry) = entries.next_entry().await.unwrap() {
            let filename = entry.file_name();
            let filename_str = filename.to_string_lossy();

            if filename_str.starts_with("claude-code-personalities.backup.") {
                backup_count += 1;
            } else {
                other_files += 1;
            }
        }

        assert_eq!(backup_count, 3, "Should keep only 3 most recent backups");
        assert_eq!(other_files, 1, "Should not touch non-backup files");
    }

    #[test]
    fn test_update_options_default() {
        let options = UpdateOptions::default();
        assert!(!options.force);
        assert!(options.interactive);
        assert!(!options.include_prereleases);
    }
}
