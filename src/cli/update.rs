use anyhow::{Context, Result, anyhow};
use cliclack::{confirm, intro, outro};
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::cli::settings::get_claude_dir;
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
    intro("Checking for updates")?;

    let version_manager =
        VersionManager::new().with_context(|| "Failed to initialize version manager")?;

    let Some(latest_release) = check_and_get_release(&version_manager, &options).await? else {
        outro("Already on the latest version")?;
        return Ok(());
    };

    let should_proceed = show_update_info_and_confirm(&latest_release, &options).await?;
    if !should_proceed {
        return Ok(());
    }

    perform_update(&version_manager, &latest_release).await?;

    Ok(())
}

async fn check_and_get_release(
    version_manager: &VersionManager,
    options: &UpdateOptions,
) -> Result<Option<crate::version::GitHubRelease>> {
    let update_info = version_manager
        .check_for_update()
        .await
        .with_context(|| "Failed to check for updates")?;

    if let Some(release) = update_info {
        if release.prerelease && !options.include_prereleases {
            print_status("Latest release is a pre-release. Use --include-prereleases to update.");
            return Ok(None);
        }
        Ok(Some(release))
    } else if options.force {
        print_status("Force update requested...");
        let release = version_manager
            .get_latest_release()
            .await
            .with_context(|| "Failed to get latest release for force update")?;
        Ok(Some(release))
    } else {
        Ok(None)
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
    print_status(&format!("{comparison} available"));

    let changelog = format_changelog(latest_release);
    if changelog != "No changelog available" {
        println!();
        print_status("What's new:");
        for line in changelog.lines().take(5) {
            println!("    {line}");
        }
        if changelog.lines().count() > 5 {
            println!("    ...");
        }
    }

    if options.interactive {
        println!();
        let should_update = confirm("Update now?")
            .initial_value(true)
            .interact()
            .with_context(|| "Failed to get user confirmation for update")?;

        if !should_update {
            print_status("Update cancelled.");
            return Ok(false);
        }
    }

    Ok(true)
}

async fn perform_update(
    version_manager: &VersionManager,
    latest_release: &crate::version::GitHubRelease,
) -> Result<()> {
    let platform = Platform::detect().with_context(|| "Failed to detect current platform")?;

    if !platform.is_supported() {
        return Err(anyhow!("Unsupported platform: {}", platform.description()));
    }

    let asset = version_manager
        .find_platform_asset(latest_release, &platform.target)
        .ok_or_else(|| {
            anyhow!(
                "No binary available for platform: {}",
                platform.description()
            )
        })?;

    let paths = setup_update_paths().await?;

    println!();
    download_and_verify_binary(version_manager, asset, &paths.temp_binary).await?;
    backup_and_replace_binary(&paths).await?;

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

async fn setup_update_paths() -> Result<UpdatePaths> {
    let claude_dir = get_claude_dir().with_context(|| "Failed to determine Claude directory")?;

    // Find the existing binary location
    let current_binary = find_existing_binary().await?
        .ok_or_else(|| anyhow!(
            "No claude-code-personalities binary found in PATH, ~/.local/bin, or ~/.claude/. \
            Please install the binary first using the install.sh script or download from GitHub releases."
        ))?;

    let backup_binary = claude_dir.join(format!(
        "claude-code-personalities.backup.{CURRENT_VERSION}"
    ));
    let temp_binary = claude_dir.join("claude-code-personalities.tmp");

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
    print_status("Downloading...");
    version_manager
        .download_asset(asset, temp_binary)
        .await
        .with_context(|| "Failed to download update")?;

    print_status("Verifying...");
    verify_binary_sha256(asset, temp_binary)
        .await
        .with_context(|| "SHA256 verification failed")?;

    Ok(())
}

async fn backup_and_replace_binary(paths: &UpdatePaths) -> Result<()> {
    print_status("Installing...");
    fs::copy(&paths.current_binary, &paths.backup_binary)
        .await
        .with_context(|| format!("Failed to create backup: {}", paths.backup_binary.display()))?;

    fs::rename(&paths.temp_binary, &paths.current_binary)
        .await
        .with_context(|| "Failed to replace current binary with updated version")?;

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

    Ok(())
}

async fn verify_installation_and_cleanup(paths: &UpdatePaths, latest_version: &str) -> Result<()> {
    let new_version = get_binary_version(&paths.current_binary)
        .await
        .with_context(|| "Failed to verify new binary version")?;

    cleanup_old_backups(&paths.claude_dir)
        .await
        .with_context(|| "Failed to clean up old backups")?;

    if new_version.trim_start_matches('v') == latest_version {
        println!();
        outro(format!("Updated to v{latest_version}"))?;
        print_update_success(CURRENT_VERSION);
    } else {
        println!();
        outro(format!(
            "Updated (expected v{latest_version}, got {new_version})"
        ))?;
        print_update_success(CURRENT_VERSION);
    }

    Ok(())
}

/// Verify binary integrity using SHA256 checksum
async fn verify_binary_sha256(
    asset: &crate::version::GitHubAsset,
    binary_path: &Path,
) -> Result<()> {
    use sha2::{Digest, Sha256};

    // Download the checksum file
    let checksum_url = format!("{}.sha256", asset.browser_download_url);
    let response = reqwest::get(&checksum_url)
        .await
        .with_context(|| format!("Failed to download checksum from {}", checksum_url))?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Failed to download checksum: HTTP {}",
            response.status()
        ));
    }

    let checksum_content = response
        .text()
        .await
        .with_context(|| "Failed to read checksum content")?;

    // Parse expected hash (format: "hash  filename" or just "hash")
    let expected_hash = checksum_content
        .split_whitespace()
        .next()
        .ok_or_else(|| anyhow!("Invalid checksum file format"))?
        .to_lowercase();

    // Compute actual hash of downloaded binary
    let binary_content = fs::read(binary_path)
        .await
        .with_context(|| format!("Failed to read binary: {}", binary_path.display()))?;

    let mut hasher = Sha256::new();
    hasher.update(&binary_content);
    let actual_hash = format!("{:x}", hasher.finalize());

    // Compare hashes
    if actual_hash != expected_hash {
        return Err(anyhow!(
            "Checksum mismatch!\nExpected: {}\nActual:   {}",
            expected_hash,
            actual_hash
        ));
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
        // Silently ignore cleanup failures - not critical
        let _ = fs::remove_file(path).await;
    }

    Ok(())
}

fn print_update_success(old_version: &str) {
    println!();
    println!("  Backup: ~/.claude/claude-code-personalities.backup.{old_version}");
}

fn print_status(message: &str) {
    println!("  {message}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

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
