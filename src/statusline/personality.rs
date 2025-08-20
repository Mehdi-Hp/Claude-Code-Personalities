use regex::Regex;
use once_cell::sync::Lazy;

use crate::state::SessionState;

// Compiled regexes for better performance and error handling
static BUILD_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"build|compile|make").expect("Invalid build regex"));
static TEST_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"test|spec").expect("Invalid test regex"));
static DEPLOY_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\bdeploy\b|docker-compose|rollout|release").expect("Invalid deploy regex"));
static DATABASE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"database|sql|mongo|postgres|mysql|redis|sqlite").expect("Invalid database regex"));
static PACKAGE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(npm|yarn|pip|cargo|gem|composer)\s+(install|add)\b").expect("Invalid package regex"));
static NETWORK_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"curl|wget|ping|ssh|scp").expect("Invalid network regex"));
static FILE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(ls |cd |mkdir |rm |mv |cp |find |touch |tree |pwd |cat |less |more |head |tail )").expect("Invalid file regex"));
static PROCESS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(ps |kill |killall |top|htop|jobs|fg |bg |nohup |pkill )").expect("Invalid process regex"));
static SYSTEM_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(df |du |free|uname|whoami|which |hostname|uptime|lscpu)").expect("Invalid system regex"));
static ADMIN_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(sudo |su |chmod |chown |systemctl |service |mount |umount )").expect("Invalid admin regex"));
static PERMISSION_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(chmod |chown |chgrp |umask |setfacl |getfacl )").expect("Invalid permission regex"));
static TEXT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(grep |sed |awk |sort |uniq |cut |tr |wc |diff )").expect("Invalid text regex"));
static EDITOR_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(vim |nvim |nano |emacs |code |subl )").expect("Invalid editor regex"));
static ARCHIVE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(tar |zip |unzip |gzip |gunzip |7z |rar )").expect("Invalid archive regex"));
static ENV_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(export |source |echo |env|set |alias |history)").expect("Invalid env regex"));
static VCS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(svn |hg |cvs |diff |patch )").expect("Invalid vcs regex"));
static CONTAINER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(docker |podman |kubectl |helm |k9s )").expect("Invalid container regex"));

#[must_use] 
pub fn determine_personality(
    state: &SessionState,
    tool_name: &str,
    file_path: Option<&str>,
    command: Option<&str>,
) -> String {
    // Check for frustration states first (highest priority)
    if let Some(frustration_personality) = get_frustration_personality(state.error_count) {
        return frustration_personality;
    }
    
    // Check tool-specific personalities (only high-priority ones like Bash git commands, Grep)
    if let Some(tool_personality) = get_tool_personality(tool_name, command) {
        return tool_personality;
    }
    
    // Check file-type specific personalities
    if let Some(file_personality) = get_file_personality(file_path) {
        return file_personality;
    }
    
    // Check consecutive action patterns
    if let Some(pattern_personality) = get_pattern_personality(state) {
        return pattern_personality;
    }
    
    // Default tool personalities (lowest priority)
    get_default_tool_personality(tool_name, state)
}

fn get_frustration_personality(error_count: u32) -> Option<String> {
    match error_count {
        5.. => Some("(╯°□°)╯︵ ┻━┻ Table Flipper".to_string()),
        3..=4 => Some("(ノಠ益ಠ)ノ Error Warrior".to_string()),
        _ => None,
    }
}

fn get_tool_personality(tool_name: &str, command: Option<&str>) -> Option<String> {
    match tool_name {
        "Bash" => get_bash_personality(command?),
        "Grep" => Some(get_grep_personality()),
        _ => None,
    }
}

fn get_default_tool_personality(tool_name: &str, state: &SessionState) -> String {
    match tool_name {
        "Edit" => "(⌐■_■) Code Wizard".to_string(),
        "Write" => "(• ε •) Gentle Refactorer".to_string(),
        "Delete" => "(ง'̀-'́)ง Code Janitor".to_string(),
        "Review" => "¯\\_(ツ)_/¯ Casual Code Reviewer".to_string(),
        "Read" => {
            if state.consecutive_actions > 5 {
                "⋋| ◉ ͟ʖ ◉ |⋌ Search Maestro".to_string()
            } else {
                "╭༼ ººل͟ºº ༽╮ Research King".to_string()
            }
        }
        _ => "ʕ•ᴥ•ʔ Code Wizard".to_string(),
    }
}

fn get_bash_personality(command: &str) -> Option<String> {
    // Git operations
    if command.starts_with("git ") {
        return Some("┗(▀̿Ĺ̯▀̿ ̿)┓ Git Manager".to_string());
    }
    
    // Test execution
    if is_test_command(command) {
        return Some("( ദ്ദി ˙ᗜ˙ ) Test Taskmaster".to_string());
    }
    
    // Deployment/Infrastructure
    if is_deploy_command(command) {
        return Some("( ͡ _ ͡°)ﾉ⚲ Deployment Guard".to_string());
    }
    
    // Database operations
    if is_database_command(command) {
        return Some("⚆_⚆ Database Expert".to_string());
    }
    
    // Build/compilation
    if is_build_command(command) {
        return Some("ᕦ(ò_óˇ)ᕤ Compilation Warrior".to_string());
    }
    
    // Package management
    if is_package_command(command) {
        return Some("^⎚-⎚^ Dependency Wrangler".to_string());
    }
    
    // File operations
    if is_file_command(command) {
        return Some("ᓚ₍ ^. .^₎ File Explorer".to_string());
    }
    
    // Process management
    if is_process_command(command) {
        return Some("(╬ ಠ益ಠ) Task Assassin".to_string());
    }
    
    // Network operations
    if is_network_command(command) {
        return Some("(╭ರ_ಠ) Network Sentinel".to_string());
    }
    
    // System monitoring
    if is_system_command(command) {
        return Some("(◉_◉) System Detective".to_string());
    }
    
    // System administration
    if is_admin_command(command) {
        return Some("( ͡ಠ ʖ̯ ͡ಠ) System Admin".to_string());
    }
    
    // Permissions
    if is_permission_command(command) {
        return Some("(╯‵□′)╯ Permission Police".to_string());
    }
    
    // Text processing
    if is_text_command(command) {
        return Some("(˘▾˘~) String Surgeon".to_string());
    }
    
    // Editors
    if is_editor_command(command) {
        return Some("( . .)φ Editor User".to_string());
    }
    
    // Archive/Compression
    if is_archive_command(command) {
        return Some("(っ˘ڡ˘ς) Compression Chef".to_string());
    }
    
    // Environment/Shell
    if is_env_command(command) {
        return Some("(∗´ര ᎑ ര`∗) Environment Enchanter".to_string());
    }
    
    // Version control (non-git)
    if is_vcs_command(command) {
        return Some("(╯︵╰,) Code Historian".to_string());
    }
    
    // Container/Orchestration
    if is_container_command(command) {
        return Some("(づ｡◕‿‿◕｡)づ Container Captain".to_string());
    }
    
    None
}

fn get_grep_personality() -> String {
    "(つ◉益◉)つ Bug Hunter".to_string()
}

fn get_file_personality(file_path: Option<&str>) -> Option<String> {
    let file = file_path?;
    
    // Auth/Security files (higher priority)
    if is_auth_file(file) {
        return Some("ಠ_ಠ Security Analyst".to_string());
    }
    
    // Performance files (higher priority)
    if is_performance_file(file) {
        return Some("★⌒ヽ( ͡° ε ͡°) Performance Tuner".to_string());
    }
    
    // Quality/Lint files (higher priority)
    if is_quality_file(file) {
        return Some("৻( •̀ ᗜ •́ ৻) Quality Auditor".to_string());
    }
    
    // Documentation
    if is_doc_file(file) {
        return Some("φ(．．) Documentation Writer".to_string());
    }
    
    // React/Vue/Svelte components
    if is_ui_component_file(file) {
        return Some("(✿◠ᴗ◠) UI Developer".to_string());
    }
    
    // CSS and styling files
    if is_style_file(file) {
        return Some("♥‿♥ Style Artist".to_string());
    }
    
    // HTML and templates
    if is_template_file(file) {
        return Some("<(￣︶￣)> Markup Wizard".to_string());
    }
    
    // Config files
    if is_config_file(file) {
        return Some("(๑>؂•̀๑) Config Helper".to_string());
    }
        
    // JavaScript/TypeScript files (lower priority)
    if is_js_file(file) {
        return Some("(▀̿Ĺ̯▀̿ ̿) JS Master".to_string());
    }
    
    None
}

fn get_pattern_personality(state: &SessionState) -> Option<String> {
    // Long sessions (consecutive actions)
    if state.consecutive_actions > 20 {
        return Some("【╯°□°】╯︵ ┻━┻ Code Berserker".to_string());
    } else if state.consecutive_actions > 10 {
        return Some("┌༼◉ل͟◉༽┐ Hyperfocused Coder".to_string());
    }
    
    None
}


// Command classification functions
fn is_test_command(cmd: &str) -> bool {
    TEST_REGEX.is_match(cmd)
}

fn is_deploy_command(cmd: &str) -> bool {
    DEPLOY_REGEX.is_match(cmd)
}

fn is_database_command(cmd: &str) -> bool {
    DATABASE_REGEX.is_match(cmd)
}

fn is_build_command(cmd: &str) -> bool {
    BUILD_REGEX.is_match(cmd)
}

fn is_package_command(cmd: &str) -> bool {
    PACKAGE_REGEX.is_match(cmd)
}

fn is_file_command(cmd: &str) -> bool {
    FILE_REGEX.is_match(cmd)
}

fn is_process_command(cmd: &str) -> bool {
    PROCESS_REGEX.is_match(cmd)
}

fn is_network_command(cmd: &str) -> bool {
    NETWORK_REGEX.is_match(cmd)
}

fn is_system_command(cmd: &str) -> bool {
    SYSTEM_REGEX.is_match(cmd)
}

fn is_admin_command(cmd: &str) -> bool {
    ADMIN_REGEX.is_match(cmd)
}

fn is_permission_command(cmd: &str) -> bool {
    PERMISSION_REGEX.is_match(cmd)
}

fn is_text_command(cmd: &str) -> bool {
    TEXT_REGEX.is_match(cmd)
}

fn is_editor_command(cmd: &str) -> bool {
    EDITOR_REGEX.is_match(cmd)
}

fn is_archive_command(cmd: &str) -> bool {
    ARCHIVE_REGEX.is_match(cmd)
}

fn is_env_command(cmd: &str) -> bool {
    ENV_REGEX.is_match(cmd)
}

fn is_vcs_command(cmd: &str) -> bool {
    VCS_REGEX.is_match(cmd)
}

fn is_container_command(cmd: &str) -> bool {
    CONTAINER_REGEX.is_match(cmd)
}

// File classification functions  
fn is_doc_file(file: &str) -> bool {
    let file_lower = file.to_lowercase();
    file_lower.contains("readme") || file_lower.ends_with(".md")
}

fn is_ui_component_file(file: &str) -> bool {
    let file_lower = file.to_lowercase();
    file_lower.ends_with(".jsx") || file_lower.ends_with(".tsx") || file_lower.ends_with(".vue") || file_lower.ends_with(".svelte")
}

fn is_js_file(file: &str) -> bool {
    let file_lower = file.to_lowercase();
    file_lower.ends_with(".js") || file_lower.ends_with(".ts")
}

fn is_style_file(file: &str) -> bool {
    let file_lower = file.to_lowercase();
    file_lower.ends_with(".css") || file_lower.ends_with(".scss") || file_lower.ends_with(".sass") || 
    file_lower.ends_with(".less") || file_lower.ends_with(".styl") || file_lower.ends_with(".stylus") || 
    file_lower.ends_with(".postcss")
}

fn is_template_file(file: &str) -> bool {
    let file_lower = file.to_lowercase();
    file_lower.ends_with(".html") || file_lower.ends_with(".htm") || file_lower.ends_with(".ejs") || 
    file_lower.ends_with(".handlebars") || file_lower.ends_with(".hbs") || file_lower.ends_with(".pug") || 
    file_lower.ends_with(".jade") || file_lower.ends_with(".twig")
}

fn is_auth_file(file: &str) -> bool {
    let file_lower = file.to_lowercase();
    file_lower.contains("auth") || file_lower.contains("security")
}

fn is_config_file(file: &str) -> bool {
    let file_lower = file.to_lowercase();
    file_lower.contains("config") || file_lower.ends_with(".json") || 
    file_lower.ends_with(".yaml") || file_lower.ends_with(".yml")
}

fn is_performance_file(file: &str) -> bool {
    let file_lower = file.to_lowercase();
    file_lower.contains("optimize") || file_lower.contains("performance")
}

fn is_quality_file(file: &str) -> bool {
    let file_lower = file.to_lowercase();
    file_lower.contains("lint") || file_lower.contains("eslint") || file_lower.contains("prettier")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create test state
    fn create_test_state(error_count: u32, consecutive_actions: u32) -> SessionState {
        SessionState {
            session_id: "test".to_string(),
            activity: crate::types::Activity::Idle,
            current_job: None,
            personality: "Test".to_string(),
            previous_personality: None,
            consecutive_actions,
            error_count,
            recent_activities: Vec::new(),
        }
    }

    #[test]
    fn test_frustration_states() {
        let state = create_test_state(5, 0);
        let personality = determine_personality(&state, "Edit", None, None);
        assert_eq!(personality, "(╯°□°)╯︵ ┻━┻ Table Flipper");

        let state = create_test_state(3, 0);
        let personality = determine_personality(&state, "Edit", None, None);
        assert_eq!(personality, "(ノಠ益ಠ)ノ Error Warrior");

        let state = create_test_state(2, 0);
        let personality = determine_personality(&state, "Edit", None, None);
        assert_eq!(personality, "(⌐■_■) Code Wizard");
    }

    #[test]
    fn test_git_operations() {
        let state = create_test_state(0, 0);
        
        let personality = determine_personality(&state, "Bash", None, Some("git status"));
        assert_eq!(personality, "┗(▀̿Ĺ̯▀̿ ̿)┓ Git Manager");

        let personality = determine_personality(&state, "Bash", None, Some("git commit -m 'test'"));
        assert_eq!(personality, "┗(▀̿Ĺ̯▀̿ ̿)┓ Git Manager");

        let personality = determine_personality(&state, "Bash", None, Some("ls -la"));
        assert_ne!(personality, "┗(▀̿Ĺ̯▀̿ ̿)┓ Git Manager");
    }

    #[test]
    fn test_file_type_personalities() {
        let state = create_test_state(0, 0);

        // JavaScript files
        let personality = determine_personality(&state, "Edit", Some("test.js"), None);
        assert_eq!(personality, "(▀̿Ĺ̯▀̿ ̿) JS Master");

        let personality = determine_personality(&state, "Edit", Some("component.tsx"), None);
        assert_eq!(personality, "(✿◠ᴗ◠) UI Developer");

        // Documentation
        let personality = determine_personality(&state, "Edit", Some("README.md"), None);
        assert_eq!(personality, "φ(．．) Documentation Writer");

        // CSS files
        let personality = determine_personality(&state, "Edit", Some("styles.css"), None);
        assert_eq!(personality, "♥‿♥ Style Artist");

        // Config files
        let personality = determine_personality(&state, "Edit", Some("config.json"), None);
        assert_eq!(personality, "(๑>؂•̀๑) Config Helper");

        // Auth files
        let personality = determine_personality(&state, "Edit", Some("auth.js"), None);
        assert_eq!(personality, "ಠ_ಠ Security Analyst");
    }

    #[test]
    fn test_command_classifications() {
        // Test commands
        assert!(is_test_command("npm test"));
        assert!(is_test_command("pytest main.py"));
        assert!(is_test_command("cargo test"));
        assert!(!is_test_command("npm install"));

        // Build commands
        assert!(is_build_command("npm run build"));
        assert!(is_build_command("cargo build"));
        assert!(is_build_command("make all"));
        assert!(!is_build_command("npm test"));

        // Package commands
        assert!(is_package_command("npm install lodash"));
        assert!(is_package_command("cargo add serde"));
        assert!(is_package_command("pip install requests"));
        assert!(!is_package_command("npm test"));

        // Deploy commands
        assert!(is_deploy_command("docker-compose up"));
        assert!(is_deploy_command("kubectl deploy"));
        assert!(!is_deploy_command("docker ps"));

        // Database commands
        assert!(is_database_command("psql database"));
        assert!(is_database_command("mongo collection"));
        assert!(!is_database_command("curl api"));
    }

    #[test]
    fn test_bash_command_personalities() {
        let state = create_test_state(0, 0);

        // Test execution
        let personality = determine_personality(&state, "Bash", None, Some("npm test"));
        assert_eq!(personality, "( ദ്ദി ˙ᗜ˙ ) Test Taskmaster");

        // Build/compilation
        let personality = determine_personality(&state, "Bash", None, Some("npm run build"));
        assert_eq!(personality, "ᕦ(ò_óˇ)ᕤ Compilation Warrior");

        // Package management
        let personality = determine_personality(&state, "Bash", None, Some("npm install express"));
        assert_eq!(personality, "^⎚-⎚^ Dependency Wrangler");

        // Database
        let personality = determine_personality(&state, "Bash", None, Some("psql mydb"));
        assert_eq!(personality, "⚆_⚆ Database Expert");

        // Deployment
        let personality = determine_personality(&state, "Bash", None, Some("docker-compose up"));
        assert_eq!(personality, "( ͡ _ ͡°)ﾉ⚲ Deployment Guard");

        // File operations
        let personality = determine_personality(&state, "Bash", None, Some("ls -la"));
        assert_eq!(personality, "ᓚ₍ ^. .^₎ File Explorer");

        // Network operations
        let personality = determine_personality(&state, "Bash", None, Some("curl api.example.com"));
        assert_eq!(personality, "(╭ರ_ಠ) Network Sentinel");
    }

    #[test]
    fn test_grep_debugging() {
        let state = create_test_state(0, 0);
        let personality = determine_personality(&state, "Grep", None, None);
        assert_eq!(personality, "(つ◉益◉)つ Bug Hunter");
    }

    #[test]
    fn test_consecutive_actions() {
        // Hyperfocused coder
        let state = create_test_state(0, 15);
        let personality = determine_personality(&state, "Edit", None, None);
        assert_eq!(personality, "┌༼◉ل͟◉༽┐ Hyperfocused Coder");

        // Code berserker
        let state = create_test_state(0, 25);
        let personality = determine_personality(&state, "Edit", None, None);
        assert_eq!(personality, "【╯°□°】╯︵ ┻━┻ Code Berserker");
    }

    #[test]
    fn test_tool_specific_personalities() {
        let state = create_test_state(0, 0);

        let personality = determine_personality(&state, "Edit", None, None);
        assert_eq!(personality, "(⌐■_■) Code Wizard");

        let personality = determine_personality(&state, "Write", None, None);
        assert_eq!(personality, "(• ε •) Gentle Refactorer");

        let personality = determine_personality(&state, "Delete", None, None);
        assert_eq!(personality, "(ง'̀-'́)ง Code Janitor");

        let personality = determine_personality(&state, "Review", None, None);
        assert_eq!(personality, "¯\\_(ツ)_/¯ Casual Code Reviewer");
    }

    #[test]
    fn test_read_tool_personalities() {
        let state = create_test_state(0, 3);
        let personality = determine_personality(&state, "Read", None, None);
        assert_eq!(personality, "╭༼ ººل͟ºº ༽╮ Research King");

        let state = create_test_state(0, 6);
        let personality = determine_personality(&state, "Read", None, None);
        assert_eq!(personality, "⋋| ◉ ͟ʖ ◉ |⋌ Search Maestro");
    }

    #[test]
    fn test_file_classification_functions() {
        // Doc files
        assert!(is_doc_file("README.md"));
        assert!(is_doc_file("readme.txt"));
        assert!(is_doc_file("DOCS.md"));
        assert!(!is_doc_file("main.js"));

        // UI component files
        assert!(is_ui_component_file("Component.jsx"));
        assert!(is_ui_component_file("App.tsx"));
        assert!(is_ui_component_file("Button.vue"));
        assert!(is_ui_component_file("Modal.svelte"));
        assert!(!is_ui_component_file("utils.js"));

        // JS files
        assert!(is_js_file("script.js"));
        assert!(is_js_file("types.ts"));
        assert!(!is_js_file("styles.css"));

        // Style files
        assert!(is_style_file("main.css"));
        assert!(is_style_file("variables.scss"));
        assert!(is_style_file("theme.less"));
        assert!(!is_style_file("index.html"));

        // Template files
        assert!(is_template_file("index.html"));
        assert!(is_template_file("template.ejs"));
        assert!(is_template_file("layout.pug"));
        assert!(!is_template_file("script.js"));

        // Auth files
        assert!(is_auth_file("auth.js"));
        assert!(is_auth_file("security.ts"));
        assert!(!is_auth_file("main.js"));

        // Config files
        assert!(is_config_file("package.json"));
        assert!(is_config_file("config.yaml"));
        assert!(is_config_file("settings.yml"));
        assert!(!is_config_file("main.js"));
    }

    #[test]
    fn test_priority_order() {
        // Error states should override everything
        let state = create_test_state(5, 0);
        let personality = determine_personality(&state, "Bash", None, Some("git status"));
        assert_eq!(personality, "(╯°□°)╯︵ ┻━┻ Table Flipper");

        // Git should override file types
        let state = create_test_state(0, 0);
        let personality = determine_personality(&state, "Bash", Some("test.js"), Some("git status"));
        assert_eq!(personality, "┗(▀̿Ĺ̯▀̿ ̿)┓ Git Manager");

        // File types should override tool defaults
        let state = create_test_state(0, 0);
        let personality = determine_personality(&state, "Edit", Some("README.md"), None);
        assert_eq!(personality, "φ(．．) Documentation Writer");
    }
}