use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersonalityPreferences {
    pub show_personality: bool,
    pub show_activity: bool,
    pub show_current_job: bool,
    pub show_current_dir: bool,
    pub show_model: bool,
    pub show_error_indicators: bool,
    pub use_icons: bool,
    pub use_colors: bool,
}

impl Default for PersonalityPreferences {
    fn default() -> Self {
        Self {
            show_personality: true,
            show_activity: true,
            show_current_job: true,
            show_current_dir: false, // Hidden by default per user request
            show_model: true,
            show_error_indicators: true,
            use_icons: true,
            use_colors: true,
        }
    }
}

impl PersonalityPreferences {
    /// Get the path to the preferences file.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The home directory cannot be determined
    /// - The HOME environment variable is not set
    pub fn get_preferences_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))
            .with_context(
                || "Unable to locate home directory. Ensure the HOME environment variable is set.",
            )?;
        Ok(home.join(".claude").join("personalities_config.json"))
    }

    /// Load preferences from file, or return default if file doesn't exist.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The preferences path cannot be determined
    /// - The preferences file exists but cannot be read
    /// - The preferences file contains invalid JSON
    pub async fn load_or_default() -> Result<Self> {
        let path = Self::get_preferences_path()?;

        if path.exists() {
            let content = fs::read_to_string(&path).await.with_context(|| {
                format!(
                    "Failed to read personality preferences from {}",
                    path.display()
                )
            })?;
            let prefs: PersonalityPreferences = serde_json::from_str(&content)
                .with_context(|| "Invalid JSON format in personality preferences file")?;
            Ok(prefs)
        } else {
            Ok(Self::default())
        }
    }

    /// Save preferences to file.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The preferences path cannot be determined
    /// - The .claude directory cannot be created
    /// - JSON serialization fails
    /// - The preferences file cannot be written to disk
    pub async fn save(&self) -> Result<()> {
        let path = Self::get_preferences_path()?;

        // Create .claude directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }

        let content = serde_json::to_string_pretty(self)
            .with_context(|| "Failed to serialize personality preferences to JSON")?;
        fs::write(&path, content).await.with_context(|| {
            format!(
                "Failed to write personality preferences to {}",
                path.display()
            )
        })?;
        Ok(())
    }

    /// Get a list of all preference options with their current values
    #[must_use]
    pub fn get_options(&self) -> Vec<(&'static str, bool)> {
        vec![
            ("Show Personality", self.show_personality),
            ("Show Activity", self.show_activity),
            ("Show Current Job/File", self.show_current_job),
            ("Show Current Directory", self.show_current_dir),
            ("Show Model Indicator", self.show_model),
            ("Show Error Indicators", self.show_error_indicators),
            ("Use Icons", self.use_icons),
            ("Use Colors", self.use_colors),
        ]
    }

    /// Update preferences from a list of selected option names
    pub fn update_from_selections(&mut self, selections: &[&str]) {
        // Reset all to false first
        self.show_personality = false;
        self.show_activity = false;
        self.show_current_job = false;
        self.show_current_dir = false;
        self.show_model = false;
        self.show_error_indicators = false;
        self.use_icons = false;
        self.use_colors = false;

        // Set selected ones to true
        for selection in selections {
            match *selection {
                "Show Personality" => self.show_personality = true,
                "Show Activity" => self.show_activity = true,
                "Show Current Job/File" => self.show_current_job = true,
                "Show Current Directory" => self.show_current_dir = true,
                "Show Model Indicator" => self.show_model = true,
                "Show Error Indicators" => self.show_error_indicators = true,
                "Use Icons" => self.use_icons = true,
                "Use Colors" => self.use_colors = true,
                _ => {} // Ignore unknown options
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_preferences() {
        let prefs = PersonalityPreferences::default();
        assert!(prefs.show_personality);
        assert!(prefs.show_activity);
        assert!(prefs.show_current_job);
        assert!(!prefs.show_current_dir); // Should be false by default
        assert!(prefs.show_model);
        assert!(prefs.show_error_indicators);
        assert!(prefs.use_icons);
        assert!(prefs.use_colors);
    }

    #[test]
    fn test_get_options() {
        let prefs = PersonalityPreferences::default();
        let options = prefs.get_options();

        assert_eq!(options.len(), 8);
        assert!(options.iter().any(|(name, _)| *name == "Show Personality"));
        assert!(options.iter().any(|(name, _)| *name == "Show Activity"));
        assert!(
            options
                .iter()
                .any(|(name, _)| *name == "Show Current Job/File")
        );
        assert!(
            options
                .iter()
                .any(|(name, _)| *name == "Show Current Directory")
        );
        assert!(
            options
                .iter()
                .any(|(name, _)| *name == "Show Model Indicator")
        );
        assert!(
            options
                .iter()
                .any(|(name, _)| *name == "Show Error Indicators")
        );
        assert!(options.iter().any(|(name, _)| *name == "Use Icons"));
        assert!(options.iter().any(|(name, _)| *name == "Use Colors"));
    }

    #[test]
    fn test_update_from_selections() {
        let mut prefs = PersonalityPreferences::default();

        // Select only a few options
        let selections = vec!["Show Personality", "Use Icons"];
        prefs.update_from_selections(&selections);

        assert!(prefs.show_personality);
        assert!(!prefs.show_activity);
        assert!(!prefs.show_current_job);
        assert!(!prefs.show_current_dir);
        assert!(!prefs.show_model);
        assert!(!prefs.show_error_indicators);
        assert!(prefs.use_icons);
        assert!(!prefs.use_colors);
    }

    #[tokio::test]
    async fn test_save_and_load() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().join("test_config.json");

        // Mock the preferences path function temporarily
        let mut prefs = PersonalityPreferences::default();
        prefs.show_current_dir = true; // Change a default value

        // Manually save to temp path for testing
        let content = serde_json::to_string_pretty(&prefs).unwrap();
        fs::write(&temp_path, content).await.unwrap();

        // Load and verify
        let loaded_content = fs::read_to_string(&temp_path).await.unwrap();
        let loaded_prefs: PersonalityPreferences = serde_json::from_str(&loaded_content).unwrap();

        assert!(loaded_prefs.show_current_dir);
        assert_eq!(loaded_prefs.show_personality, prefs.show_personality);
    }
}
