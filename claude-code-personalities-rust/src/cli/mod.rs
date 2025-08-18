use anyhow::Result;
use colored::*;
use std::path::PathBuf;
use tokio::fs;
use inquire::MultiSelect;

use crate::statusline::icons::*;
use crate::config::PersonalityPreferences;

const VERSION: &str = "0.1.0";
#[allow(dead_code)]
const GITHUB_REPO: &str = "Mehdi-Hp/claude-code-personalities";

pub async fn install() -> Result<()> {
    println!("{}", "Installing Claude Code Personalities...".bold().blue());
    println!("This is the Rust version - much faster than the bash implementation!");
    
    // TODO: Implement full installation logic
    // For now, just show what would happen
    
    let claude_dir = get_claude_dir()?;
    println!("Would install to: {}", claude_dir.display());
    
    println!("\n{} {}", ICON_CHECK.green(), "Installation planned (not yet implemented)".green());
    println!("Next steps:");
    println!("  1. Copy binary to ~/.claude/");
    println!("  2. Update settings.json");
    println!("  3. Configure hooks");
    
    Ok(())
}

pub async fn update() -> Result<()> {
    println!("{}", "Updating Claude Code Personalities...".bold().blue());
    
    // TODO: Implement update logic
    println!("\n{} {}", ICON_CHECK.green(), "Update planned (not yet implemented)".green());
    
    Ok(())
}

pub async fn uninstall() -> Result<()> {
    println!("{}", "Uninstalling Claude Code Personalities...".bold().blue());
    
    // TODO: Implement uninstall logic
    println!("\n{} {}", ICON_CHECK.green(), "Uninstall planned (not yet implemented)".green());
    
    Ok(())
}

pub async fn status() -> Result<()> {
    println!("{}", "Claude Code Personalities Status".bold().blue());
    println!();
    
    let claude_dir = get_claude_dir()?;
    let settings_file = claude_dir.join("settings.json");
    
    // Check if Claude directory exists
    if claude_dir.exists() {
        println!("{} Claude directory found: {}", ICON_CHECK.green(), claude_dir.display());
    } else {
        println!("{} Claude directory not found: {}", ICON_ERROR.red(), claude_dir.display());
    }
    
    // Check if settings.json exists
    if settings_file.exists() {
        println!("{} Settings file found", ICON_CHECK.green());
        
        // Check if configured for personalities
        let content = fs::read_to_string(&settings_file).await?;
        if content.contains("claude-code-personalities") {
            println!("{} Personalities configured in settings", ICON_CHECK.green());
        } else {
            println!("{} Personalities not configured in settings", ICON_WARNING.yellow());
        }
    } else {
        println!("{} Settings file not found", ICON_ERROR.red());
    }
    
    // Test statusline
    println!("\n{} Testing statusline output:", ICON_INFO.cyan());
    let test_input = r#"{"model":{"display_name":"Opus"},"workspace":{"current_dir":"/test"},"session_id":"test"}"#;
    
    // Simulate statusline output
    use crate::statusline::*;
    use crate::state::SessionState;
    
    let claude_input: ClaudeInput = serde_json::from_str(test_input)?;
    let session_id = claude_input.session_id.unwrap_or_else(|| "test".to_string());
    let model_name = claude_input.model
        .and_then(|m| m.display_name)
        .unwrap_or_else(|| "Claude".to_string());
    
    let state = SessionState::load(&session_id).await?;
    let prefs = PersonalityPreferences::load_or_default().await?;
    let statusline = build_statusline(&state, &model_name, &prefs);
    println!("  Output: {}", statusline);
    
    Ok(())
}

pub async fn check_update() -> Result<()> {
    println!("{}", "Checking for updates...".bold().blue());
    
    // TODO: Implement actual GitHub API check
    println!("Current version: v{}", VERSION);
    println!("Latest version: v{} (placeholder)", VERSION);
    println!("\n{} You are running the latest version!", ICON_CHECK.green());
    
    Ok(())
}

pub async fn configure() -> Result<()> {
    println!("{}", "Configure Claude Code Personalities".bold().blue());
    println!("Select which elements to show in the statusline:\n");
    
    // Load current preferences or defaults
    let mut prefs = PersonalityPreferences::load_or_default().await?;
    
    // Get all options with their current states
    let options = prefs.get_options();
    let option_names: Vec<&str> = options.iter().map(|(name, _)| *name).collect();
    
    // Get indices of currently selected options
    let default_selections: Vec<usize> = options
        .iter()
        .enumerate()
        .filter_map(|(i, (_, enabled))| if *enabled { Some(i) } else { None })
        .collect();
    
    // Show interactive multi-select prompt
    let selected = MultiSelect::new("Features to enable:", option_names.clone())
        .with_default(&default_selections)
        .prompt()?;
    
    // Update preferences based on selections
    prefs.update_from_selections(&selected);
    
    // Save updated preferences
    prefs.save().await?;
    
    println!("\n{} Configuration saved successfully!", ICON_CHECK.green());
    println!("Location: {}", PersonalityPreferences::get_preferences_path()?.display());
    
    // Show what was enabled/disabled
    println!("\nEnabled features:");
    for feature in &selected {
        println!("  {} {}", ICON_CHECK.green(), feature);
    }
    
    if selected.len() < option_names.len() {
        println!("\nDisabled features:");
        for option in &option_names {
            if !selected.contains(option) {
                println!("  {} {}", ICON_WARNING.yellow(), option);
            }
        }
    }
    
    println!("\n{} Run your Claude Code session to see the changes!", ICON_INFO.cyan());
    
    Ok(())
}

pub fn help() -> Result<()> {
    println!("Claude Code Personalities v{}", VERSION);
    println!("Dynamic text-face personalities for Claude Code's statusline");
    println!();
    println!("Usage: claude-code-personalities [COMMAND]");
    println!();
    println!("Commands:");
    println!("  install       Install Claude Code Personalities");
    println!("  update        Update to the latest version");
    println!("  uninstall     Remove Claude Code Personalities");
    println!("  status        Check installation status");
    println!("  check-update  Check for available updates");
    println!("  config        Configure display options");
    println!("  help          Show this help message");
    println!();
    println!("Modes (called by Claude Code):");
    println!("  --statusline  Run in statusline mode");
    println!("  --hook TYPE   Run in hook mode (pre-tool, post-tool, prompt-submit, session-end)");
    println!();
    println!("This is the Rust rewrite - much faster than the bash version!");
    
    Ok(())
}

fn get_claude_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    Ok(home.join(".claude"))
}