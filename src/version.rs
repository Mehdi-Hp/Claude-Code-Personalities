use anyhow::{Context, Result, anyhow};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const GITHUB_REPO: &str = "Mehdi-Hp/claude-code-personalities";
const VERSION_CACHE_DURATION: Duration = Duration::from_secs(60 * 60); // 1 hour

/// Current version from Cargo.toml (set at compile time)
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub published_at: Option<String>,
    pub assets: Vec<GitHubAsset>,
    pub prerelease: bool,
    pub draft: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VersionCache {
    latest_version: String,
    release_info: GitHubRelease,
    cached_at: u64,
}

impl VersionCache {
    fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now - self.cached_at > VERSION_CACHE_DURATION.as_secs()
    }
}

pub struct VersionManager {
    cache_path: std::path::PathBuf,
}

impl VersionManager {
    /// Create a new version manager with default cache location.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The cache directory cannot be determined from system directories
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .or_else(|| dirs::home_dir().map(|h| h.join(".cache")))
            .ok_or_else(|| anyhow!("Could not determine cache directory"))?;

        let cache_path = cache_dir
            .join("claude-code-personalities")
            .join("version_cache.json");

        Ok(Self { cache_path })
    }

    /// Get current version as a semver Version
    pub fn current_version() -> Result<Version> {
        Version::parse(CURRENT_VERSION)
            .with_context(|| format!("Failed to parse current version: {CURRENT_VERSION}"))
    }

    /// Check if an update is available by comparing current version with latest release.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Current version cannot be parsed as valid semver
    /// - Latest release information cannot be fetched from GitHub
    /// - GitHub API returns invalid response data
    /// - Network connectivity issues prevent API access
    pub async fn check_for_update(&self) -> Result<Option<GitHubRelease>> {
        let latest_release = self.get_latest_release().await?;
        let latest_version = self.parse_version_from_tag(&latest_release.tag_name)?;
        let current_version = Self::current_version()?;

        if latest_version > current_version {
            Ok(Some(latest_release))
        } else {
            Ok(None)
        }
    }

    /// Check if an update is available, always fetching fresh data from GitHub (bypasses cache).
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Current version cannot be parsed as valid semver
    /// - Latest release information cannot be fetched from GitHub
    /// - GitHub API returns invalid response data
    /// - Network connectivity issues prevent API access
    pub async fn check_for_update_force(&self) -> Result<Option<GitHubRelease>> {
        // Fetch fresh data directly from GitHub, bypassing cache
        let latest_release = self.fetch_latest_release().await?;
        let latest_version = self.parse_version_from_tag(&latest_release.tag_name)?;
        let current_version = Self::current_version()?;

        // Cache the fresh result for future non-force requests
        if let Err(e) = self.cache_version_info(&latest_release).await {
            // Don't fail the entire operation if caching fails
            eprintln!("Warning: Failed to cache version info: {e}");
        }

        if latest_version > current_version {
            Ok(Some(latest_release))
        } else {
            Ok(None)
        }
    }

    /// Get the latest release information from GitHub
    pub async fn get_latest_release(&self) -> Result<GitHubRelease> {
        // Try to load from cache first
        if let Ok(cached) = self.load_cached_version().await
            && !cached.is_expired()
        {
            return Ok(cached.release_info);
        }

        // Fetch from GitHub API
        let release = self.fetch_latest_release().await?;

        // Cache the result
        self.cache_version_info(&release).await?;

        Ok(release)
    }

    /// Download an asset from a GitHub release to the specified path.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The asset download URL is unreachable
    /// - HTTP request fails or returns non-success status
    /// - Downloaded content size doesn't match expected size
    /// - Target directory cannot be created
    /// - Target file cannot be created or written to
    /// - File write or flush operations fail
    pub async fn download_asset(
        &self,
        asset: &GitHubAsset,
        target_path: &std::path::Path,
    ) -> Result<()> {
        use tokio::fs::File;
        use tokio::io::AsyncWriteExt;

        let response = reqwest::get(&asset.browser_download_url)
            .await
            .with_context(|| format!("Failed to download asset: {}", asset.name))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to download asset: HTTP {}",
                response.status()
            ));
        }

        let bytes = response
            .bytes()
            .await
            .with_context(|| format!("Failed to read asset content: {}", asset.name))?;

        // Verify size matches expected
        if bytes.len() != asset.size as usize {
            return Err(anyhow!(
                "Downloaded asset size mismatch: expected {}, got {}",
                asset.size,
                bytes.len()
            ));
        }

        // Create parent directory if needed
        if let Some(parent) = target_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        let mut file = File::create(target_path)
            .await
            .with_context(|| format!("Failed to create file: {}", target_path.display()))?;

        file.write_all(&bytes)
            .await
            .with_context(|| format!("Failed to write file: {}", target_path.display()))?;

        file.flush()
            .await
            .with_context(|| format!("Failed to flush file: {}", target_path.display()))?;

        Ok(())
    }

    /// Find the appropriate asset for the current platform
    #[must_use]
    pub fn find_platform_asset<'a>(
        &self,
        release: &'a GitHubRelease,
        platform_target: &str,
    ) -> Option<&'a GitHubAsset> {
        let expected_name = format!("claude-code-personalities-{platform_target}");
        release
            .assets
            .iter()
            .find(|asset| asset.name == expected_name)
    }

    /// Parse version from GitHub tag (removes 'v' prefix)
    fn parse_version_from_tag(&self, tag: &str) -> Result<Version> {
        let version_str = tag.strip_prefix('v').unwrap_or(tag);
        Version::parse(version_str)
            .with_context(|| format!("Failed to parse version from tag: {tag}"))
    }

    /// Fetch latest release from GitHub API
    async fn fetch_latest_release(&self) -> Result<GitHubRelease> {
        let url = format!("https://api.github.com/repos/{GITHUB_REPO}/releases/latest");

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header(
                "User-Agent",
                format!("claude-code-personalities/{CURRENT_VERSION}"),
            )
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .with_context(|| format!("Failed to fetch latest release from GitHub: {url}"))?;

        if !response.status().is_success() {
            return Err(anyhow!("GitHub API error: HTTP {}", response.status()));
        }

        let release: GitHubRelease = response
            .json()
            .await
            .with_context(|| "Failed to parse GitHub API response")?;

        if release.draft {
            return Err(anyhow!("Latest release is a draft"));
        }

        Ok(release)
    }

    /// Load cached version information
    async fn load_cached_version(&self) -> Result<VersionCache> {
        let content = tokio::fs::read_to_string(&self.cache_path)
            .await
            .with_context(|| {
                format!(
                    "Failed to read version cache: {}",
                    self.cache_path.display()
                )
            })?;

        serde_json::from_str(&content).with_context(|| "Failed to parse version cache JSON")
    }

    /// Cache version information
    async fn cache_version_info(&self, release: &GitHubRelease) -> Result<()> {
        let cache = VersionCache {
            latest_version: release.tag_name.clone(),
            release_info: release.clone(),
            cached_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        // Create cache directory if needed
        if let Some(parent) = self.cache_path.parent() {
            tokio::fs::create_dir_all(parent).await.with_context(|| {
                format!("Failed to create cache directory: {}", parent.display())
            })?;
        }

        let content = serde_json::to_string_pretty(&cache)
            .with_context(|| "Failed to serialize version cache")?;

        tokio::fs::write(&self.cache_path, content)
            .await
            .with_context(|| {
                format!(
                    "Failed to write version cache: {}",
                    self.cache_path.display()
                )
            })?;

        Ok(())
    }
}

impl Default for VersionManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default VersionManager")
    }
}

/// Format a version comparison for display
#[must_use]
pub fn format_version_comparison(current: &str, latest: &str) -> String {
    format!("Current: v{current} → Available: v{latest}")
}

/// Get a user-friendly changelog from release body
pub fn format_changelog(release: &GitHubRelease) -> String {
    match &release.body {
        Some(body) if !body.trim().is_empty() => {
            // Clean up common GitHub release formatting
            body.lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
                .join("\n")
        }
        _ => "No changelog available".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_version_parsing() {
        let version = VersionManager::current_version().unwrap();
        assert!(!version.to_string().is_empty());
    }

    #[test]
    fn test_version_tag_parsing() {
        let vm = VersionManager::new().unwrap();

        // Test with 'v' prefix
        let version = vm.parse_version_from_tag("v1.2.3").unwrap();
        assert_eq!(version, Version::parse("1.2.3").unwrap());

        // Test without 'v' prefix
        let version = vm.parse_version_from_tag("1.2.3").unwrap();
        assert_eq!(version, Version::parse("1.2.3").unwrap());
    }

    #[test]
    fn test_platform_asset_matching() {
        let vm = VersionManager::new().unwrap();

        let release = GitHubRelease {
            tag_name: "v1.0.0".to_string(),
            name: Some("Release 1.0.0".to_string()),
            body: Some("Initial release".to_string()),
            published_at: None,
            assets: vec![
                GitHubAsset {
                    name: "claude-code-personalities-macos-x86_64".to_string(),
                    browser_download_url: "https://example.com/macos".to_string(),
                    size: 1024,
                },
                GitHubAsset {
                    name: "claude-code-personalities-linux-x86_64".to_string(),
                    browser_download_url: "https://example.com/linux".to_string(),
                    size: 1024,
                },
            ],
            prerelease: false,
            draft: false,
        };

        let asset = vm.find_platform_asset(&release, "macos-x86_64");
        assert!(asset.is_some());
        assert_eq!(
            asset.unwrap().name,
            "claude-code-personalities-macos-x86_64"
        );

        let asset = vm.find_platform_asset(&release, "unsupported-platform");
        assert!(asset.is_none());
    }

    #[test]
    fn test_version_cache_expiry() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Fresh cache
        let fresh_cache = VersionCache {
            latest_version: "1.0.0".to_string(),
            release_info: GitHubRelease {
                tag_name: "v1.0.0".to_string(),
                name: None,
                body: None,
                published_at: None,
                assets: vec![],
                prerelease: false,
                draft: false,
            },
            cached_at: now,
        };
        assert!(!fresh_cache.is_expired());

        // Expired cache
        let expired_cache = VersionCache {
            latest_version: "1.0.0".to_string(),
            release_info: GitHubRelease {
                tag_name: "v1.0.0".to_string(),
                name: None,
                body: None,
                published_at: None,
                assets: vec![],
                prerelease: false,
                draft: false,
            },
            cached_at: now - VERSION_CACHE_DURATION.as_secs() - 1,
        };
        assert!(expired_cache.is_expired());
    }

    #[test]
    fn test_format_version_comparison() {
        let comparison = format_version_comparison("1.0.0", "1.1.0");
        assert_eq!(comparison, "Current: v1.0.0 → Available: v1.1.0");
    }

    #[test]
    fn test_format_changelog() {
        let release = GitHubRelease {
            tag_name: "v1.0.0".to_string(),
            name: None,
            body: Some("## What's New\n\n- Feature A\n- Feature B\n- Bug fixes".to_string()),
            published_at: None,
            assets: vec![],
            prerelease: false,
            draft: false,
        };

        let changelog = format_changelog(&release);
        assert!(changelog.contains("Feature A"));
        assert!(changelog.contains("Feature B"));
    }

    #[tokio::test]
    async fn test_version_manager_creation() {
        let vm = VersionManager::new().unwrap();
        assert!(
            vm.cache_path
                .to_string_lossy()
                .contains("claude-code-personalities")
        );
    }
}
