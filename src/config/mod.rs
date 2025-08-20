pub mod preferences;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub use preferences::PersonalityPreferences;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeSettings {
    #[serde(rename = "statusLine")]
    pub status_line: Option<StatusLineConfig>,
    pub hooks: Option<HooksConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusLineConfig {
    #[serde(rename = "type")]
    pub type_: String,
    pub command: String,
    pub padding: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HooksConfig {
    #[serde(rename = "PreToolUse")]
    pub pre_tool_use: Option<Vec<HookConfig>>,
    #[serde(rename = "PostToolUse")]
    pub post_tool_use: Option<Vec<HookConfig>>,
    #[serde(rename = "UserPromptSubmit")]
    pub user_prompt_submit: Option<Vec<HookConfig>>,
    #[serde(rename = "Stop")]
    pub stop: Option<Vec<HookConfig>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HookConfig {
    pub matcher: Option<String>,
    pub hooks: Vec<HookCommand>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HookCommand {
    #[serde(rename = "type")]
    pub type_: String,
    pub command: String,
}

impl ClaudeSettings {
    /// Get the path to Claude Code's settings.json file.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The home directory cannot be determined
    #[allow(dead_code)]
    pub fn get_claude_settings_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home.join(".claude").join("settings.json"))
    }
    
    /// Load Claude Code settings from disk, or return None if file doesn't exist.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The settings path cannot be determined
    /// - The settings file exists but cannot be read
    /// - The settings file contains invalid JSON
    #[allow(dead_code)]
    pub async fn load() -> Result<Option<Self>> {
        let path = Self::get_claude_settings_path()?;
        
        if path.exists() {
            let content = tokio::fs::read_to_string(&path).await?;
            let settings: ClaudeSettings = serde_json::from_str(&content)?;
            Ok(Some(settings))
        } else {
            Ok(None)
        }
    }
    
    /// Save Claude Code settings to disk.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The settings path cannot be determined
    /// - JSON serialization fails
    /// - The settings file cannot be written to disk
    #[allow(dead_code)]
    pub async fn save(&self) -> Result<()> {
        let path = Self::get_claude_settings_path()?;
        let content = serde_json::to_string_pretty(self)?;
        tokio::fs::write(&path, content).await?;
        Ok(())
    }
    
    #[allow(dead_code)]
    pub fn configure_for_personalities(binary_path: &str) -> Self {
        Self {
            status_line: Some(StatusLineConfig {
                type_: "command".to_string(),
                command: format!("{binary_path} --statusline"),
                padding: Some(0),
            }),
            hooks: Some(HooksConfig {
                pre_tool_use: Some(vec![HookConfig {
                    matcher: Some("*".to_string()),
                    hooks: vec![HookCommand {
                        type_: "command".to_string(),
                        command: format!("{binary_path} --hook pre-tool"),
                    }],
                }]),
                post_tool_use: Some(vec![HookConfig {
                    matcher: Some("*".to_string()),
                    hooks: vec![HookCommand {
                        type_: "command".to_string(),
                        command: format!("{binary_path} --hook post-tool"),
                    }],
                }]),
                user_prompt_submit: Some(vec![HookConfig {
                    matcher: None,
                    hooks: vec![HookCommand {
                        type_: "command".to_string(),
                        command: format!("{binary_path} --hook prompt-submit"),
                    }],
                }]),
                stop: Some(vec![HookConfig {
                    matcher: Some("".to_string()),
                    hooks: vec![HookCommand {
                        type_: "command".to_string(),
                        command: format!("{binary_path} --hook session-end"),
                    }],
                }]),
            }),
        }
    }
}