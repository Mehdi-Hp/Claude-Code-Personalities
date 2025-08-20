use anyhow::{anyhow, Context, Result};
use chrono::Local;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Clone)]
pub struct ClaudeSettings {
    pub settings_path: PathBuf,
    pub content: Value,
}

impl ClaudeSettings {
    /// Load Claude settings from the standard location (~/.claude/settings.json).
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The home directory cannot be determined
    /// - The settings file cannot be read due to permissions or I/O errors
    /// - The settings file contains invalid JSON
    pub async fn load() -> Result<Self> {
        let settings_path = get_claude_settings_path()?;
        Self::load_from_path(settings_path).await
    }
    
    /// Load Claude settings from a specific file path.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The specified file cannot be read due to permissions or I/O errors
    /// - The file contains invalid JSON that cannot be parsed
    /// - Path conversion or file system operations fail
    pub async fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let settings_path = path.as_ref().to_path_buf();
        
        let content = if settings_path.exists() {
            let raw_content = fs::read_to_string(&settings_path).await
                .with_context(|| format!("Failed to read Claude settings from {}", settings_path.display()))?;
            
            serde_json::from_str(&raw_content)
                .with_context(|| "Failed to parse Claude settings JSON")?
        } else {
            // Create minimal default settings
            serde_json::json!({})
        };
        
        Ok(ClaudeSettings { settings_path, content })
    }
    
    /// Save the current settings back to the settings file.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The parent directory cannot be created
    /// - JSON serialization fails
    /// - The file cannot be written due to permissions or I/O errors
    pub async fn save(&self) -> Result<()> {
        // Create .claude directory if it doesn't exist
        if let Some(parent) = self.settings_path.parent() {
            fs::create_dir_all(parent).await
                .with_context(|| format!("Failed to create Claude directory: {}", parent.display()))?;
        }
        
        let pretty_json = serde_json::to_string_pretty(&self.content)
            .with_context(|| "Failed to serialize Claude settings to JSON")?;
        
        fs::write(&self.settings_path, pretty_json).await
            .with_context(|| format!("Failed to write Claude settings to {}", self.settings_path.display()))?;
        
        Ok(())
    }
    
    /// Create a timestamped backup of the current settings file.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The settings file does not exist
    /// - The backup file cannot be created due to permissions or I/O errors
    /// - File copy operations fail
    pub async fn create_backup(&self) -> Result<PathBuf> {
        if !self.settings_path.exists() {
            // No file to backup
            return Err(anyhow!("Settings file does not exist, cannot create backup"));
        }
        
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let backup_path = self.settings_path.with_extension(format!("json.backup.{timestamp}"));
        
        fs::copy(&self.settings_path, &backup_path).await
            .with_context(|| format!(
                "Failed to create backup: {} -> {}",
                self.settings_path.display(),
                backup_path.display()
            ))?;
        
        Ok(backup_path)
    }
    
    /// Configure Claude Code to use personalities statusline.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The binary path cannot be converted to a valid string
    /// - The binary path contains invalid UTF-8 characters
    pub fn configure_statusline(&mut self, binary_path: &Path) -> Result<()> {
        let statusline_config = serde_json::json!({
            "type": "command",
            "command": binary_path.to_str().ok_or_else(|| anyhow!("Invalid binary path"))?,
            "args": ["--statusline"],
            "padding": 0
        });
        
        self.content["statusLine"] = statusline_config;
        Ok(())
    }
    
    /// Configure Claude Code hooks for personality tracking.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The binary path cannot be converted to a valid string
    /// - The binary path contains invalid UTF-8 characters
    /// - JSON serialization of hook configuration fails
    pub fn configure_hooks(&mut self, binary_path: &Path) -> Result<()> {
        let binary_str = binary_path.to_str().ok_or_else(|| anyhow!("Invalid binary path"))?;
        
        // Get existing hooks or create new object
        let hooks = self.content.get("hooks").cloned().unwrap_or_else(|| serde_json::json!({}));
        let mut hooks_obj = match hooks {
            Value::Object(obj) => obj,
            _ => Map::new(),
        };
        
        // Pre-tool and post-tool hooks for activity tracking
        let activity_hook = serde_json::json!({
            "type": "command",
            "command": binary_str,
            "args": ["--hook", "activity"]
        });
        
        // Configure PreToolUse hook
        let pre_tool_hook = serde_json::json!([{
            "matcher": "*",
            "hooks": [activity_hook.clone()]
        }]);
        hooks_obj.insert("PreToolUse".to_string(), pre_tool_hook);
        
        // Configure PostToolUse hook
        let post_tool_hook = serde_json::json!([{
            "matcher": "*",
            "hooks": [activity_hook]
        }]);
        hooks_obj.insert("PostToolUse".to_string(), post_tool_hook);
        
        // User prompt submit hook for error reset
        let prompt_submit_hook = serde_json::json!([{
            "hooks": [{
                "type": "command",
                "command": binary_str,
                "args": ["--hook", "prompt-submit"]
            }]
        }]);
        hooks_obj.insert("UserPromptSubmit".to_string(), prompt_submit_hook);
        
        // Session end hook for cleanup
        let session_end_hook = serde_json::json!([{
            "matcher": "",
            "hooks": [{
                "type": "command",
                "command": binary_str,
                "args": ["--hook", "session-end"]
            }]
        }]);
        hooks_obj.insert("Stop".to_string(), session_end_hook);
        
        self.content["hooks"] = Value::Object(hooks_obj);
        Ok(())
    }
    
    /// Remove personality configuration from Claude settings
    pub fn remove_personality_config(&mut self) {
        // Remove statusline if it's our command
        if let Some(statusline) = self.content.get("statusLine") {
            if let Some(command) = statusline.get("command") {
                if let Some(cmd_str) = command.as_str() {
                    if cmd_str.contains("claude-code-personalities") {
                        self.content.as_object_mut().unwrap().remove("statusLine");
                    }
                }
            }
        }
        
        // Remove our hooks
        if let Some(Value::Object(hooks)) = self.content.get_mut("hooks") {
            // Remove hooks that contain our binary
            for hook_type in ["PreToolUse", "PostToolUse", "UserPromptSubmit", "Stop"] {
                if let Some(Value::Array(hook_array)) = hooks.get_mut(hook_type) {
                    hook_array.retain(|hook_entry| {
                        !hook_contains_personality_command(hook_entry)
                    });
                    
                    // Remove the hook type entirely if no hooks remain
                    if hook_array.is_empty() {
                        hooks.remove(hook_type);
                    }
                }
            }
            
            // Remove hooks object if empty
            if hooks.is_empty() {
                self.content.as_object_mut().unwrap().remove("hooks");
            }
        }
    }
    
    /// Check if personalities are currently configured
    pub fn is_personality_configured(&self) -> bool {
        // Check statusline
        if let Some(statusline) = self.content.get("statusLine") {
            if let Some(command) = statusline.get("command") {
                if let Some(cmd_str) = command.as_str() {
                    if cmd_str.contains("claude-code-personalities") {
                        return true;
                    }
                }
            }
        }
        
        // Check hooks
        if let Some(Value::Object(hooks)) = self.content.get("hooks") {
            for (_hook_type, hook_value) in hooks {
                if hook_contains_personality_command(hook_value) {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Get a summary of current configuration
    pub fn get_configuration_summary(&self) -> ConfigurationSummary {
        ConfigurationSummary {
            has_statusline: self.content.get("statusLine").is_some(),
            has_personality_statusline: self.is_personality_configured(),
            hook_types: self.get_configured_hook_types(),
            settings_file_exists: self.settings_path.exists(),
        }
    }
    
    /// Get list of configured hook types
    fn get_configured_hook_types(&self) -> Vec<String> {
        if let Some(Value::Object(hooks)) = self.content.get("hooks") {
            hooks.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigurationSummary {
    #[allow(dead_code)]
    pub has_statusline: bool,
    pub has_personality_statusline: bool,
    pub hook_types: Vec<String>,
    #[allow(dead_code)]
    pub settings_file_exists: bool,
}

impl ConfigurationSummary {
    pub fn is_fully_configured(&self) -> bool {
        self.has_personality_statusline 
            && self.hook_types.contains(&"PreToolUse".to_string())
            && self.hook_types.contains(&"PostToolUse".to_string())
            && self.hook_types.contains(&"UserPromptSubmit".to_string())
            && self.hook_types.contains(&"Stop".to_string())
    }
}

/// Check if a hook value contains our personality command
fn hook_contains_personality_command(hook_value: &Value) -> bool {
    match hook_value {
        Value::Array(arr) => {
            arr.iter().any(hook_contains_personality_command)
        }
        Value::Object(obj) => {
            if let Some(Value::Array(hooks)) = obj.get("hooks") {
                return hooks.iter().any(|hook| {
                    if let Some(command) = hook.get("command") {
                        if let Some(cmd_str) = command.as_str() {
                            return cmd_str.contains("claude-code-personalities");
                        }
                    }
                    false
                });
            } else if let Some(command) = obj.get("command") {
                if let Some(cmd_str) = command.as_str() {
                    return cmd_str.contains("claude-code-personalities");
                }
            }
            false
        }
        _ => false,
    }
}

/// Get the path to Claude's settings.json file (~/.claude/settings.json).
///
/// # Errors
///
/// This function will return an error if:
/// - The user's home directory cannot be determined
pub fn get_claude_settings_path() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow!("Could not find home directory"))?;
    Ok(home.join(".claude").join("settings.json"))
}

/// Get the path to Claude's directory (~/.claude/).
///
/// # Errors
///
/// This function will return an error if:
/// - The user's home directory cannot be determined
pub fn get_claude_dir() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow!("Could not find home directory"))?;
    Ok(home.join(".claude"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{NamedTempFile, TempDir};
    use std::io::Write;

    #[tokio::test]
    async fn test_load_nonexistent_settings() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");
        
        let settings = ClaudeSettings::load_from_path(&settings_path).await.unwrap();
        assert_eq!(settings.content, serde_json::json!({}));
    }

    #[tokio::test]
    async fn test_load_existing_settings() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let settings_content = serde_json::json!({
            "statusLine": {
                "type": "command",
                "command": "echo test"
            }
        });
        
        temp_file.write_all(settings_content.to_string().as_bytes()).unwrap();
        temp_file.flush().unwrap();
        
        let settings = ClaudeSettings::load_from_path(temp_file.path()).await.unwrap();
        assert_eq!(settings.content["statusLine"]["command"], "echo test");
    }

    #[tokio::test]
    async fn test_save_settings() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");
        
        let settings = ClaudeSettings {
            settings_path: settings_path.clone(),
            content: serde_json::json!({"test": "value"}),
        };
        
        settings.save().await.unwrap();
        
        // Verify file was created and has correct content
        assert!(settings_path.exists());
        let content = fs::read_to_string(&settings_path).await.unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["test"], "value");
    }

    #[tokio::test]
    async fn test_configure_statusline() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");
        let binary_path = PathBuf::from("/path/to/binary");
        
        let mut settings = ClaudeSettings {
            settings_path,
            content: serde_json::json!({}),
        };
        
        settings.configure_statusline(&binary_path).unwrap();
        
        assert_eq!(settings.content["statusLine"]["type"], "command");
        assert_eq!(settings.content["statusLine"]["command"], "/path/to/binary");
        assert_eq!(settings.content["statusLine"]["padding"], 0);
    }

    #[tokio::test]
    async fn test_configure_hooks() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");
        let binary_path = PathBuf::from("/path/to/binary");
        
        let mut settings = ClaudeSettings {
            settings_path,
            content: serde_json::json!({}),
        };
        
        settings.configure_hooks(&binary_path).unwrap();
        
        // Check that all required hooks are configured
        assert!(settings.content["hooks"]["PreToolUse"].is_array());
        assert!(settings.content["hooks"]["PostToolUse"].is_array());
        assert!(settings.content["hooks"]["UserPromptSubmit"].is_array());
        assert!(settings.content["hooks"]["Stop"].is_array());
    }

    #[tokio::test]
    async fn test_is_personality_configured() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");
        let binary_path = PathBuf::from("/path/to/claude-code-personalities");
        
        let mut settings = ClaudeSettings {
            settings_path,
            content: serde_json::json!({}),
        };
        
        // Initially not configured
        assert!(!settings.is_personality_configured());
        
        // Configure statusline
        settings.configure_statusline(&binary_path).unwrap();
        assert!(settings.is_personality_configured());
        
        // Configure hooks too
        settings.configure_hooks(&binary_path).unwrap();
        assert!(settings.is_personality_configured());
    }

    #[tokio::test]
    async fn test_remove_personality_config() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");
        let binary_path = PathBuf::from("/path/to/claude-code-personalities");
        
        let mut settings = ClaudeSettings {
            settings_path,
            content: serde_json::json!({}),
        };
        
        // Configure personalities
        settings.configure_statusline(&binary_path).unwrap();
        settings.configure_hooks(&binary_path).unwrap();
        assert!(settings.is_personality_configured());
        
        // Remove configuration
        settings.remove_personality_config();
        assert!(!settings.is_personality_configured());
    }

    #[tokio::test]
    async fn test_configuration_summary() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");
        let binary_path = PathBuf::from("/path/to/claude-code-personalities");
        
        let mut settings = ClaudeSettings {
            settings_path,
            content: serde_json::json!({}),
        };
        
        let summary = settings.get_configuration_summary();
        assert!(!summary.is_fully_configured());
        
        // Configure everything
        settings.configure_statusline(&binary_path).unwrap();
        settings.configure_hooks(&binary_path).unwrap();
        
        let summary = settings.get_configuration_summary();
        assert!(summary.is_fully_configured());
        assert!(summary.has_personality_statusline);
        assert_eq!(summary.hook_types.len(), 4); // PreToolUse, PostToolUse, UserPromptSubmit, Stop
    }

    #[test]
    fn test_hook_contains_personality_command() {
        // Test with array hook structure
        let hook_array = serde_json::json!([{
            "matcher": "*",
            "hooks": [{
                "type": "command",
                "command": "/path/to/claude-code-personalities",
                "args": ["--hook", "activity"]
            }]
        }]);
        assert!(hook_contains_personality_command(&hook_array));
        
        // Test with object hook structure
        let hook_obj = serde_json::json!({
            "type": "command",
            "command": "/path/to/claude-code-personalities",
            "args": ["--hook", "activity"]
        });
        assert!(hook_contains_personality_command(&hook_obj));
        
        // Test with non-personality command
        let other_hook = serde_json::json!({
            "type": "command",
            "command": "/path/to/other-tool",
            "args": ["--some-flag"]
        });
        assert!(!hook_contains_personality_command(&other_hook));
    }
}