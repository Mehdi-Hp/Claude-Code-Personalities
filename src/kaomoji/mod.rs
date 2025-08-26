//! Centralized kaomoji (text-face emoticon) management module
//!
//! This module provides a single source of truth for all kaomoji personalities used
//! throughout the application, organized by category and purpose.

use std::fmt;

use crate::state::PersonalityModifier;

// Re-export all kaomoji categories
pub mod default;
pub mod file;
pub mod mood;
pub mod tool;

// Re-export commonly used kaomojis
pub use default::*;
pub use file::*;
pub use mood::*;
pub use tool::*;

/// A kaomoji personality consisting of a text-face and description
#[derive(Debug, Clone, PartialEq)]
pub struct Kaomoji {
    pub face: &'static str,
    pub description: &'static str,
}

impl Kaomoji {
    /// Create a new kaomoji
    pub const fn new(face: &'static str, description: &'static str) -> Self {
        Self { face, description }
    }

    /// Get the full personality string (face + description)
    pub fn personality(&self) -> String {
        format!("{} {}", self.face, self.description)
    }
}

impl fmt::Display for Kaomoji {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.face, self.description)
    }
}

/// Get mood-based kaomoji based on personality modifier and frustration level
pub fn get_mood_kaomoji(modifier: &PersonalityModifier, frustration_level: u8) -> &'static Kaomoji {
    match modifier {
        PersonalityModifier::Frustrated => {
            if frustration_level >= 10 {
                &FRUSTRATED_HIGH
            } else {
                &FRUSTRATED_MID
            }
        }
        PersonalityModifier::InTheZone => &HYPERFOCUSED,
        PersonalityModifier::Normal => &CODE_WIZARD,
    }
}

/// Get tool-specific kaomoji based on tool name and optional command
pub fn get_tool_kaomoji(tool_name: &str, command: Option<&str>) -> Option<&'static Kaomoji> {
    match tool_name {
        "Bash" => get_bash_kaomoji(command?),
        "Grep" => Some(&BUG_HUNTER),
        _ => None,
    }
}

/// Get bash command-specific kaomoji
pub fn get_bash_kaomoji(command: &str) -> Option<&'static Kaomoji> {
    // Git operations (highest priority)
    if is_git_command(command) {
        return Some(&GIT_MANAGER);
    }

    // Testing
    if is_test_command(command) {
        return Some(&TEST_TASKMASTER);
    }

    // Deployment/Infrastructure
    if is_deploy_command(command) {
        return Some(&DEPLOYMENT_GUARD);
    }

    // Database operations
    if is_database_command(command) {
        return Some(&DATABASE_EXPERT);
    }

    // Build/Compilation
    if is_build_command(command) {
        return Some(&COMPILATION_WARRIOR);
    }

    // Package management
    if is_package_command(command) {
        return Some(&DEPENDENCY_WRANGLER);
    }

    // File operations
    if is_file_command(command) {
        return Some(&FILE_EXPLORER);
    }

    // Process management
    if is_process_command(command) {
        return Some(&TASK_ASSASSIN);
    }

    // Network operations
    if is_network_command(command) {
        return Some(&NETWORK_SENTINEL);
    }

    // System monitoring
    if is_system_command(command) {
        return Some(&SYSTEM_DETECTIVE);
    }

    // System administration
    if is_admin_command(command) {
        return Some(&SYSTEM_ADMIN);
    }

    // Permissions
    if is_permission_command(command) {
        return Some(&PERMISSION_POLICE);
    }

    // Text processing
    if is_text_command(command) {
        return Some(&STRING_SURGEON);
    }

    // Editor commands
    if is_editor_command(command) {
        return Some(&EDITOR_USER);
    }

    // Archive operations
    if is_archive_command(command) {
        return Some(&COMPRESSION_CHEF);
    }

    // Environment/Shell
    if is_env_command(command) {
        return Some(&ENVIRONMENT_ENCHANTER);
    }

    // Version control (non-git)
    if is_vcs_command(command) {
        return Some(&CODE_HISTORIAN);
    }

    // Docker/Container operations
    if is_container_command(command) {
        return Some(&CONTAINER_CAPTAIN);
    }

    None
}

/// Get file-type specific kaomoji based on file path
pub fn get_file_kaomoji(file_path: &str) -> Option<&'static Kaomoji> {
    // Auth/Security files (higher priority)
    if is_auth_file(file_path) {
        return Some(&SECURITY_ANALYST);
    }

    // Performance files (higher priority)
    if is_performance_file(file_path) {
        return Some(&PERFORMANCE_TUNER);
    }

    // JavaScript/TypeScript files (high priority - before quality checks)
    if is_js_file(file_path) {
        return Some(&JS_MASTER);
    }

    // React/Vue/Svelte components (high priority - before quality checks)
    if is_ui_component_file(file_path) {
        return Some(&UI_DEVELOPER);
    }

    // Quality/Testing files
    if is_quality_file(file_path) {
        return Some(&QUALITY_AUDITOR);
    }

    // Documentation files
    if is_docs_file(file_path) {
        return Some(&DOCUMENTATION_WRITER);
    }

    // CSS/Styling files
    if is_style_file(file_path) {
        return Some(&STYLE_ARTIST);
    }

    // Template files
    if is_template_file(file_path) {
        return Some(&MARKUP_WIZARD);
    }

    // Config files
    if is_config_file(file_path) {
        return Some(&CONFIG_HELPER);
    }

    None
}

/// Get pattern-based kaomoji for consecutive actions
pub fn get_pattern_kaomoji(consecutive_actions: u32) -> Option<&'static Kaomoji> {
    if consecutive_actions > 20 {
        Some(&CODE_BERSERKER)
    } else if consecutive_actions > 10 {
        Some(&HYPERFOCUSED)
    } else {
        None
    }
}

/// Get default tool kaomoji
pub fn get_default_tool_kaomoji(tool_name: &str, consecutive_actions: u32) -> &'static Kaomoji {
    match tool_name {
        "Edit" => &CODE_WIZARD_ALT,
        "Write" => &GENTLE_REFACTORER,
        "Delete" => &CODE_JANITOR,
        "Review" => &CASUAL_CODE_REVIEWER,
        "Read" => {
            if consecutive_actions > 5 {
                &SEARCH_MAESTRO
            } else {
                &RESEARCH_KING
            }
        }
        _ => &CODE_WIZARD,
    }
}

// Helper functions for command classification
fn is_git_command(command: &str) -> bool {
    command.contains("git ")
}

fn is_test_command(command: &str) -> bool {
    command.contains("test") || command.contains("spec")
}

fn is_deploy_command(command: &str) -> bool {
    command.contains("deploy")
        || command.contains("docker")
        || command.contains("kubectl")
        || command.contains("terraform")
        || command.contains("ansible")
}

fn is_database_command(command: &str) -> bool {
    command.contains("database")
        || command.contains("sql")
        || command.contains("mongo")
        || command.contains("postgres")
        || command.contains("mysql")
        || command.contains("redis")
        || command.contains("sqlite")
}

fn is_build_command(command: &str) -> bool {
    command.contains("build") || command.contains("compile") || command.contains("make")
}

fn is_package_command(command: &str) -> bool {
    command.contains("npm install")
        || command.contains("yarn add")
        || command.contains("pip install")
        || command.contains("cargo add")
}

fn is_file_command(command: &str) -> bool {
    command.starts_with("ls ")
        || command.starts_with("cd ")
        || command.starts_with("mkdir ")
        || command.starts_with("rm ")
        || command.starts_with("mv ")
        || command.starts_with("cp ")
        || command.starts_with("find ")
        || command.starts_with("touch ")
        || command.starts_with("tree ")
}

fn is_process_command(command: &str) -> bool {
    command.starts_with("ps ")
        || command.starts_with("kill ")
        || command.starts_with("killall ")
        || command.contains("top")
        || command.contains("htop")
}

fn is_network_command(command: &str) -> bool {
    command.contains("curl") || command.contains("wget") || command.contains("ping")
}

fn is_system_command(command: &str) -> bool {
    command.starts_with("df ") || command.contains("free") || command.contains("uname")
}

fn is_admin_command(command: &str) -> bool {
    command.starts_with("sudo ") || command.contains("systemctl") || command.contains("service")
}

fn is_permission_command(command: &str) -> bool {
    command.starts_with("chmod ") || command.starts_with("chown ")
}

fn is_text_command(command: &str) -> bool {
    command.starts_with("grep ")
        || command.starts_with("sed ")
        || command.starts_with("awk ")
        || command.starts_with("sort ")
}

fn is_editor_command(command: &str) -> bool {
    command.starts_with("vim ")
        || command.starts_with("nvim ")
        || command.starts_with("nano ")
        || command.starts_with("code ")
}

fn is_archive_command(command: &str) -> bool {
    command.starts_with("tar ") || command.starts_with("zip ") || command.starts_with("unzip ")
}

fn is_env_command(command: &str) -> bool {
    command.starts_with("export ")
        || command.starts_with("source ")
        || command.starts_with("echo ")
        || command.contains("env")
}

fn is_vcs_command(command: &str) -> bool {
    command.contains("svn ") || command.contains("hg ") || command.contains("bzr ")
}

fn is_container_command(command: &str) -> bool {
    command.contains("docker") && !command.contains("docker-compose")
}

// File classification helpers
fn is_auth_file(file: &str) -> bool {
    let file_lower = file.to_lowercase();
    file_lower.contains("auth")
        || file_lower.contains("security")
        || file_lower.contains("login")
        || file_lower.contains("passport")
        || file_lower.contains("jwt")
}

fn is_performance_file(file: &str) -> bool {
    let file_lower = file.to_lowercase();
    file_lower.contains("performance")
        || file_lower.contains("benchmark")
        || file_lower.contains("profil")
        || file_lower.contains("metric")
}

fn is_quality_file(file: &str) -> bool {
    let file_lower = file.to_lowercase();
    file_lower.contains("test")
        || file_lower.contains("spec")
        || file_lower.contains("lint")
        || file_lower.contains("quality")
}

fn is_docs_file(file: &str) -> bool {
    let file_lower = file.to_lowercase();
    file_lower.contains("readme")
        || file_lower.contains("docs/")
        || file_lower.contains("documentation")
        || file_lower.ends_with(".md")
}

fn is_ui_component_file(file: &str) -> bool {
    file.ends_with(".jsx")
        || file.ends_with(".tsx")
        || file.ends_with(".vue")
        || file.ends_with(".svelte")
}

fn is_style_file(file: &str) -> bool {
    file.ends_with(".css")
        || file.ends_with(".scss")
        || file.ends_with(".sass")
        || file.ends_with(".less")
}

fn is_template_file(file: &str) -> bool {
    file.ends_with(".html")
        || file.ends_with(".ejs")
        || file.ends_with(".pug")
        || file.ends_with(".hbs")
}

fn is_config_file(file: &str) -> bool {
    let file_lower = file.to_lowercase();
    file_lower.contains("config")
        || file_lower.ends_with(".json")
        || file_lower.ends_with(".yaml")
        || file_lower.ends_with(".yml")
        || file_lower.ends_with(".toml")
}

fn is_js_file(file: &str) -> bool {
    file.ends_with(".js") || file.ends_with(".ts") || file.ends_with(".mjs")
}
