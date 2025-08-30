use anyhow::{Context, Result, anyhow};
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
            let raw_content = fs::read_to_string(&settings_path).await.with_context(|| {
                format!(
                    "Failed to read Claude settings from {}",
                    settings_path.display()
                )
            })?;

            serde_json::from_str(&raw_content)
                .with_context(|| "Failed to parse Claude settings JSON")?
        } else {
            // Create minimal default settings
            serde_json::json!({})
        };

        Ok(ClaudeSettings {
            settings_path,
            content,
        })
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
            fs::create_dir_all(parent).await.with_context(|| {
                format!("Failed to create Claude directory: {}", parent.display())
            })?;
        }

        let pretty_json = serde_json::to_string_pretty(&self.content)
            .with_context(|| "Failed to serialize Claude settings to JSON")?;

        fs::write(&self.settings_path, pretty_json)
            .await
            .with_context(|| {
                format!(
                    "Failed to write Claude settings to {}",
                    self.settings_path.display()
                )
            })?;

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
            return Err(anyhow!(
                "Settings file does not exist, cannot create backup"
            ));
        }

        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let backup_path = self
            .settings_path
            .with_extension(format!("json.backup.{timestamp}"));

        fs::copy(&self.settings_path, &backup_path)
            .await
            .with_context(|| {
                format!(
                    "Failed to create backup: {} -> {}",
                    self.settings_path.display(),
                    backup_path.display()
                )
            })?;

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
            "command": format!("{} --statusline", binary_path.to_str().ok_or_else(|| anyhow!("Invalid binary path"))?),
            "padding": 0
        });

        self.content["statusLine"] = statusline_config;
        Ok(())
    }

    /// Configure Claude Code hooks for personality tracking.
    ///
    /// This method merges personality hooks with existing hooks, preserving
    /// any previously configured hooks while adding the necessary personality
    /// tracking hooks.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The binary path cannot be converted to a valid string
    /// - The binary path contains invalid UTF-8 characters
    /// - JSON serialization of hook configuration fails
    /// - Existing hooks structure is malformed and cannot be parsed
    pub fn configure_hooks(&mut self, binary_path: &Path) -> Result<()> {
        let binary_str = binary_path
            .to_str()
            .ok_or_else(|| anyhow!("Invalid binary path"))?;

        // Get existing hooks or create new object
        let hooks = self
            .content
            .get("hooks")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        let mut hooks_obj = match hooks {
            Value::Object(obj) => obj,
            _ => Map::new(),
        };

        // Pre-tool hook for activity tracking
        let pre_tool_hook = serde_json::json!({
            "type": "command",
            "command": format!("{} --hook pre-tool", binary_str)
        });

        // Post-tool hook for activity tracking
        let post_tool_hook = serde_json::json!({
            "type": "command",
            "command": format!("{} --hook post-tool", binary_str)
        });

        // Configure PreToolUse hook - merge with existing
        let pre_tool_personality_hook = serde_json::json!({
            "matcher": "*",
            "hooks": [pre_tool_hook]
        });
        Self::merge_hook_array(&mut hooks_obj, "PreToolUse", pre_tool_personality_hook)?;

        // Configure PostToolUse hook - merge with existing
        let post_tool_personality_hook = serde_json::json!({
            "matcher": "*",
            "hooks": [post_tool_hook]
        });
        Self::merge_hook_array(&mut hooks_obj, "PostToolUse", post_tool_personality_hook)?;

        // User prompt submit hook for error reset - merge with existing
        let prompt_submit_personality_hook = serde_json::json!({
            "hooks": [{
                "type": "command",
                "command": format!("{} --hook prompt-submit", binary_str)
            }]
        });
        Self::merge_hook_array(
            &mut hooks_obj,
            "UserPromptSubmit",
            prompt_submit_personality_hook,
        )?;

        // Session end hook for cleanup - merge with existing
        let session_end_personality_hook = serde_json::json!({
            "matcher": "",
            "hooks": [{
                "type": "command",
                "command": format!("{} --hook session-end", binary_str)
            }]
        });
        Self::merge_hook_array(&mut hooks_obj, "Stop", session_end_personality_hook)?;

        self.content["hooks"] = Value::Object(hooks_obj);
        Ok(())
    }

    /// Merge a personality hook into an existing hook array, preserving existing hooks.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The existing hook type exists but is not a valid array structure
    /// - JSON serialization fails during the merge process
    fn merge_hook_array(
        hooks_obj: &mut Map<String, Value>,
        hook_type: &str,
        personality_hook: Value,
    ) -> Result<()> {
        let existing_hooks = hooks_obj.get(hook_type);

        match existing_hooks {
            Some(Value::Array(existing_array)) => {
                // Clone existing hooks and add our hook
                let mut merged_hooks = existing_array.clone();

                // First remove any existing personality hooks to avoid duplicates
                merged_hooks.retain(|hook| !hook_contains_personality_command(hook));

                // Add our personality hook
                merged_hooks.push(personality_hook);
                hooks_obj.insert(hook_type.to_string(), Value::Array(merged_hooks));
            }
            Some(_) => {
                // Existing value is not an array - this is malformed, but we'll replace it
                return Err(anyhow!(
                    "Existing hooks configuration for {} is malformed (not an array)",
                    hook_type
                ));
            }
            None => {
                // No existing hooks for this type - create new array
                hooks_obj.insert(hook_type.to_string(), Value::Array(vec![personality_hook]));
            }
        }

        Ok(())
    }

    /// Remove personality configuration from Claude settings
    ///
    /// # Panics
    ///
    /// This function will panic if the JSON structure is malformed and cannot be converted
    /// to a JSON object. This should only happen if the content was corrupted externally.
    pub fn remove_personality_config(&mut self) {
        // Remove statusline if it's our command
        if let Some(statusline) = self.content.get("statusLine")
            && let Some(command) = statusline.get("command")
            && let Some(cmd_str) = command.as_str()
            && cmd_str.contains("claude-code-personalities")
        {
            self.content.as_object_mut().unwrap().remove("statusLine");
        }

        // Remove our hooks surgically - only remove personality commands, preserve others
        if let Some(Value::Object(hooks)) = self.content.get_mut("hooks") {
            let mut hook_types_to_remove = Vec::new();

            for hook_type in ["PreToolUse", "PostToolUse", "UserPromptSubmit", "Stop"] {
                if let Some(Value::Array(hook_array)) = hooks.get_mut(hook_type) {
                    // Instead of removing entire entries, filter out personality commands
                    let mut entries_to_remove = Vec::new();

                    for (entry_idx, hook_entry) in hook_array.iter_mut().enumerate() {
                        if let Some(Value::Array(entry_hooks)) = hook_entry.get_mut("hooks") {
                            // Remove only personality commands from this entry's hooks array
                            entry_hooks.retain(|hook| {
                                if let Some(command) = hook.get("command") {
                                    if let Some(cmd_str) = command.as_str() {
                                        return !cmd_str.contains("claude-code-personalities");
                                    }
                                }
                                true
                            });

                            // If the entry has no hooks left, mark it for removal
                            if entry_hooks.is_empty() {
                                entries_to_remove.push(entry_idx);
                            }
                        }
                    }

                    // Remove empty entries (in reverse order to maintain indices)
                    for &entry_idx in entries_to_remove.iter().rev() {
                        hook_array.remove(entry_idx);
                    }

                    // Remove the hook type entirely if no entries remain
                    if hook_array.is_empty() {
                        hook_types_to_remove.push(hook_type);
                    }
                }
            }

            // Remove empty hook types
            for hook_type in hook_types_to_remove {
                hooks.remove(hook_type);
            }

            // Remove hooks object if empty
            if hooks.is_empty() {
                self.content.as_object_mut().unwrap().remove("hooks");
            }
        }
    }

    /// Check if personalities are currently configured
    #[must_use]
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
    #[must_use]
    pub fn get_configuration_summary(&self) -> ConfigurationSummary {
        ConfigurationSummary {
            has_personality_statusline: self.is_personality_configured(),
            hook_types: self.get_configured_hook_types(),
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
    pub has_personality_statusline: bool,
    pub hook_types: Vec<String>,
}

impl ConfigurationSummary {
    #[must_use]
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
        Value::Array(arr) => arr.iter().any(hook_contains_personality_command),
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
    let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
    Ok(home.join(".claude").join("settings.json"))
}

/// Get the path to Claude's directory (~/.claude/).
///
/// # Errors
///
/// This function will return an error if:
/// - The user's home directory cannot be determined
pub fn get_claude_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
    Ok(home.join(".claude"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    #[tokio::test]
    async fn test_load_nonexistent_settings() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");

        let settings = ClaudeSettings::load_from_path(&settings_path)
            .await
            .unwrap();
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

        temp_file
            .write_all(settings_content.to_string().as_bytes())
            .unwrap();
        temp_file.flush().unwrap();

        let settings = ClaudeSettings::load_from_path(temp_file.path())
            .await
            .unwrap();
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
        assert_eq!(
            settings.content["statusLine"]["command"],
            "/path/to/binary --statusline"
        );
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
    async fn test_configure_hooks_preserves_existing() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");
        let binary_path = PathBuf::from("/path/to/claude-code-personalities");

        // Start with existing hooks configuration
        let existing_config = serde_json::json!({
            "hooks": {
                "PreToolUse": [{
                    "matcher": "*.py",
                    "hooks": [{
                        "type": "command",
                        "command": "pylint"
                    }]
                }],
                "PostToolUse": [{
                    "matcher": "build*",
                    "hooks": [{
                        "type": "command",
                        "command": "notify-send"
                    }]
                }],
                "UserPromptSubmit": [{
                    "hooks": [{
                        "type": "command",
                        "command": "custom-logger"
                    }]
                }]
            }
        });

        let mut settings = ClaudeSettings {
            settings_path,
            content: existing_config,
        };

        // Configure personality hooks
        settings.configure_hooks(&binary_path).unwrap();

        // Verify existing hooks are preserved
        let pre_tool_hooks = settings.content["hooks"]["PreToolUse"].as_array().unwrap();
        let post_tool_hooks = settings.content["hooks"]["PostToolUse"].as_array().unwrap();
        let prompt_submit_hooks = settings.content["hooks"]["UserPromptSubmit"]
            .as_array()
            .unwrap();
        let stop_hooks = settings.content["hooks"]["Stop"].as_array().unwrap();

        // Should have 2 hooks each for PreToolUse and PostToolUse (existing + personality)
        assert_eq!(pre_tool_hooks.len(), 2);
        assert_eq!(post_tool_hooks.len(), 2);
        assert_eq!(prompt_submit_hooks.len(), 2);
        assert_eq!(stop_hooks.len(), 1); // Only personality hook

        // Check that existing pylint hook is preserved
        let has_pylint = pre_tool_hooks.iter().any(|hook| {
            hook.get("matcher").and_then(|m| m.as_str()) == Some("*.py")
                && hook
                    .get("hooks")
                    .and_then(|h| h.as_array())
                    .is_some_and(|hooks| {
                        hooks
                            .iter()
                            .any(|h| h.get("command").and_then(|c| c.as_str()) == Some("pylint"))
                    })
        });
        assert!(has_pylint, "Existing pylint hook should be preserved");

        // Check that existing notify-send hook is preserved
        let has_notify = post_tool_hooks.iter().any(|hook| {
            hook.get("matcher").and_then(|m| m.as_str()) == Some("build*")
                && hook
                    .get("hooks")
                    .and_then(|h| h.as_array())
                    .is_some_and(|hooks| {
                        hooks.iter().any(|h| {
                            h.get("command").and_then(|c| c.as_str()) == Some("notify-send")
                        })
                    })
        });
        assert!(has_notify, "Existing notify-send hook should be preserved");

        // Check that personality hooks are added
        let has_personality_pre = pre_tool_hooks.iter().any(|hook| {
            hook.get("matcher").and_then(|m| m.as_str()) == Some("*")
                && hook
                    .get("hooks")
                    .and_then(|h| h.as_array())
                    .is_some_and(|hooks| {
                        hooks.iter().any(|h| {
                            h.get("command")
                                .and_then(|c| c.as_str())
                                .is_some_and(|cmd| {
                                    cmd.contains("claude-code-personalities")
                                        && (cmd.contains("--hook pre-tool")
                                            || cmd.contains("--hook post-tool"))
                                })
                        })
                    })
        });
        assert!(
            has_personality_pre,
            "Personality PreToolUse hook should be added"
        );
    }

    #[tokio::test]
    async fn test_configure_hooks_removes_duplicate_personality_hooks() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");
        let binary_path = PathBuf::from("/path/to/claude-code-personalities");

        // Start with existing personality hooks (duplicate scenario)
        let existing_config = serde_json::json!({
            "hooks": {
                "PreToolUse": [{
                    "matcher": "*",
                    "hooks": [{
                        "type": "command",
                        "command": "/path/to/claude-code-personalities --hook activity"
                    }]
                }, {
                    "matcher": "*.js",
                    "hooks": [{
                        "type": "command",
                        "command": "eslint"
                    }]
                }]
            }
        });

        let mut settings = ClaudeSettings {
            settings_path,
            content: existing_config,
        };

        // Configure personality hooks again (should remove duplicates)
        settings.configure_hooks(&binary_path).unwrap();

        let pre_tool_hooks = settings.content["hooks"]["PreToolUse"].as_array().unwrap();

        // Should have 2 hooks: eslint + new personality (old personality removed)
        assert_eq!(pre_tool_hooks.len(), 2);

        // Count personality hooks - should be exactly 1
        let personality_hook_count = pre_tool_hooks
            .iter()
            .filter(|hook| hook_contains_personality_command(hook))
            .count();
        assert_eq!(
            personality_hook_count, 1,
            "Should have exactly 1 personality hook after deduplication"
        );

        // Verify eslint hook is preserved
        let has_eslint = pre_tool_hooks.iter().any(|hook| {
            hook.get("matcher").and_then(|m| m.as_str()) == Some("*.js")
                && hook
                    .get("hooks")
                    .and_then(|h| h.as_array())
                    .is_some_and(|hooks| {
                        hooks
                            .iter()
                            .any(|h| h.get("command").and_then(|c| c.as_str()) == Some("eslint"))
                    })
        });
        assert!(has_eslint, "Existing eslint hook should be preserved");
    }

    #[tokio::test]
    async fn test_configure_hooks_handles_malformed_hooks() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");
        let binary_path = PathBuf::from("/path/to/binary");

        // Start with malformed hooks (not an array)
        let malformed_config = serde_json::json!({
            "hooks": {
                "PreToolUse": "not-an-array"
            }
        });

        let mut settings = ClaudeSettings {
            settings_path,
            content: malformed_config,
        };

        // Should return an error for malformed hooks
        let result = settings.configure_hooks(&binary_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("malformed"));
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
    async fn test_remove_personality_config_preserves_other_hooks() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");
        let binary_path = PathBuf::from("/path/to/claude-code-personalities");

        // Start with a complex existing hooks configuration that includes non-personality hooks
        let existing_config = serde_json::json!({
            "hooks": {
                "PreToolUse": [{
                    "matcher": "*.py",
                    "hooks": [{
                        "type": "command",
                        "command": "pylint"
                    }, {
                        "type": "command",
                        "command": "black --check"
                    }]
                }],
                "PostToolUse": [{
                    "matcher": "build*",
                    "hooks": [{
                        "type": "command",
                        "command": "notify-send Build complete"
                    }]
                }],
                "UserPromptSubmit": [{
                    "hooks": [{
                        "type": "command",
                        "command": "custom-logger"
                    }, {
                        "type": "command",
                        "command": "analytics-tracker"
                    }]
                }]
            }
        });

        let mut settings = ClaudeSettings {
            settings_path,
            content: existing_config,
        };

        // Add personality hooks to the existing configuration
        settings.configure_statusline(&binary_path).unwrap();
        settings.configure_hooks(&binary_path).unwrap();
        assert!(settings.is_personality_configured());

        // Verify we have the expected number of hooks after adding personalities
        let pre_tool_hooks = settings.content["hooks"]["PreToolUse"].as_array().unwrap();
        let post_tool_hooks = settings.content["hooks"]["PostToolUse"].as_array().unwrap();
        let prompt_submit_hooks = settings.content["hooks"]["UserPromptSubmit"]
            .as_array()
            .unwrap();

        assert_eq!(pre_tool_hooks.len(), 2); // Original + personality
        assert_eq!(post_tool_hooks.len(), 2); // Original + personality  
        assert_eq!(prompt_submit_hooks.len(), 2); // Original + personality

        // Remove personality configuration surgically
        settings.remove_personality_config();
        assert!(!settings.is_personality_configured());

        // Verify statusline is removed since it was ours (personalities overwrite existing statuslines)
        assert!(settings.content.get("statusLine").is_none());

        // Verify other hooks are preserved
        let hooks_obj = settings.content["hooks"].as_object().unwrap();
        assert!(!hooks_obj.is_empty(), "Hooks object should not be empty");

        // Check PreToolUse - should still have the original pylint/black hooks
        let pre_tool_hooks = hooks_obj["PreToolUse"].as_array().unwrap();
        assert_eq!(pre_tool_hooks.len(), 1);
        let first_entry_hooks = pre_tool_hooks[0]["hooks"].as_array().unwrap();
        assert_eq!(first_entry_hooks.len(), 2); // pylint + black

        let commands: Vec<_> = first_entry_hooks
            .iter()
            .map(|h| h["command"].as_str().unwrap())
            .collect();
        assert!(commands.contains(&"pylint"));
        assert!(commands.contains(&"black --check"));

        // Check PostToolUse - should still have notify-send
        let post_tool_hooks = hooks_obj["PostToolUse"].as_array().unwrap();
        assert_eq!(post_tool_hooks.len(), 1);
        let post_hook_cmd = post_tool_hooks[0]["hooks"][0]["command"].as_str().unwrap();
        assert_eq!(post_hook_cmd, "notify-send Build complete");

        // Check UserPromptSubmit - should still have custom-logger and analytics-tracker
        let prompt_submit_hooks = hooks_obj["UserPromptSubmit"].as_array().unwrap();
        assert_eq!(prompt_submit_hooks.len(), 1);
        let prompt_entry_hooks = prompt_submit_hooks[0]["hooks"].as_array().unwrap();
        assert_eq!(prompt_entry_hooks.len(), 2);

        let prompt_commands: Vec<_> = prompt_entry_hooks
            .iter()
            .map(|h| h["command"].as_str().unwrap())
            .collect();
        assert!(prompt_commands.contains(&"custom-logger"));
        assert!(prompt_commands.contains(&"analytics-tracker"));

        // Verify no personality-related commands remain
        let all_content = serde_json::to_string(&settings.content).unwrap();
        assert!(!all_content.contains("claude-code-personalities"));
    }

    #[tokio::test]
    async fn test_remove_personality_config_mixed_commands_in_entry() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("settings.json");
        let binary_path = PathBuf::from("/path/to/claude-code-personalities");

        // Create a scenario where personality and non-personality commands exist in the same hook entry
        let mixed_config = serde_json::json!({
            "hooks": {
                "PreToolUse": [{
                    "matcher": "*.rs",
                    "hooks": [{
                        "type": "command",
                        "command": "rustfmt --check"
                    }, {
                        "type": "command",
                        "command": "/path/to/claude-code-personalities --hook pre-tool"
                    }, {
                        "type": "command",
                        "command": "clippy --all-targets"
                    }]
                }]
            }
        });

        let mut settings = ClaudeSettings {
            settings_path,
            content: mixed_config,
        };

        // Verify we have mixed commands initially
        let pre_tool_hooks = settings.content["hooks"]["PreToolUse"][0]["hooks"]
            .as_array()
            .unwrap();
        assert_eq!(pre_tool_hooks.len(), 3);
        assert!(settings.is_personality_configured());

        // Remove personality configuration surgically
        settings.remove_personality_config();
        assert!(!settings.is_personality_configured());

        // Verify the hook entry still exists but only has non-personality commands
        let hooks_obj = settings.content["hooks"].as_object().unwrap();
        let pre_tool_hooks = hooks_obj["PreToolUse"].as_array().unwrap();
        assert_eq!(pre_tool_hooks.len(), 1);

        let remaining_hooks = pre_tool_hooks[0]["hooks"].as_array().unwrap();
        assert_eq!(remaining_hooks.len(), 2); // Only rustfmt and clippy should remain

        let commands: Vec<_> = remaining_hooks
            .iter()
            .map(|h| h["command"].as_str().unwrap())
            .collect();
        assert!(commands.contains(&"rustfmt --check"));
        assert!(commands.contains(&"clippy --all-targets"));

        // Verify personality command was removed
        let all_content = serde_json::to_string(&settings.content).unwrap();
        assert!(!all_content.contains("claude-code-personalities"));
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
                "command": "/path/to/claude-code-personalities --hook activity"
            }]
        }]);
        assert!(hook_contains_personality_command(&hook_array));

        // Test with object hook structure
        let hook_obj = serde_json::json!({
            "type": "command",
            "command": "/path/to/claude-code-personalities --hook activity"
        });
        assert!(hook_contains_personality_command(&hook_obj));

        // Test with non-personality command
        let other_hook = serde_json::json!({
            "type": "command",
            "command": "/path/to/other-tool --some-flag"
        });
        assert!(!hook_contains_personality_command(&other_hook));
    }
}
