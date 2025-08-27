use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::{self, Read};
use std::path::Path;

use crate::state::SessionState;
use crate::statusline::personality::determine_personality;
use crate::types::Activity;

#[derive(Debug, Serialize, Deserialize)]
pub struct HookInput {
    pub session_id: Option<String>,
    pub tool_name: Option<String>,
    pub tool_input: Option<serde_json::Value>,
    pub tool_response: Option<ToolResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolResponse {
    pub error: Option<serde_json::Value>,
}

/// Run a hook handler based on the hook type.
///
/// # Errors
///
/// This function will return an error if:
/// - An unknown hook type is provided
/// - The specific hook handler fails to execute
/// - Any underlying hook processing fails
pub async fn run_hook(hook_type: &str) -> Result<()> {
    match hook_type {
        "pre-tool" | "post-tool" => handle_tool_hook().await,
        "prompt-submit" => handle_prompt_submit().await,
        "session-end" => handle_session_end().await,
        _ => {
            eprintln!("Unknown hook type: {hook_type}");
            std::process::exit(1);
        }
    }
}

/// Handle pre-tool and post-tool hooks by updating session state and personality.
///
/// # Errors
///
/// This function will return an error if:
/// - No input is received from Claude Code via stdin
/// - The input JSON is malformed or cannot be parsed
/// - Session state cannot be loaded or saved
/// - Error counting operations fail
async fn handle_tool_hook() -> Result<()> {
    use anyhow::Context;

    // Read JSON from stdin
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .with_context(|| "Failed to read hook input from stdin")?;

    if input.trim().is_empty() {
        return Err(anyhow::anyhow!("No input received for hook"))
            .with_context(|| "Hook should receive JSON input from Claude Code via stdin");
    }

    let hook_input: HookInput =
        serde_json::from_str(&input).with_context(|| "Failed to parse hook input JSON")?;

    // Use a consistent fallback when session_id is missing
    let session_id = hook_input.session_id.unwrap_or_else(|| {
        // Use a predictable session ID for the current Claude session
        std::env::var("CLAUDE_SESSION_ID").unwrap_or_else(|_| "claude_current".to_string())
    });
    let tool_name = hook_input.tool_name.unwrap_or_default();

    // Load current state
    let mut state = SessionState::load(&session_id).await.with_context(|| {
        format!("Failed to load session state for hook processing (session: {session_id})")
    })?;

    // Check for errors
    if let Some(response) = &hook_input.tool_response {
        if response.error.is_some() {
            state.increment_errors().await.with_context(|| {
                format!("Failed to increment error count for session {session_id}")
            })?;
        }
    }

    // Extract tool parameters
    let (file_path, command, pattern) = extract_tool_params(hook_input.tool_input.as_ref());

    // Determine activity and current job
    let (activity, current_job) = determine_activity(
        &tool_name,
        file_path.as_deref(),
        command.as_deref(),
        pattern.as_deref(),
    );

    // Determine personality
    let personality =
        determine_personality(&state, &tool_name, file_path.as_deref(), command.as_deref());

    // Update state - this will automatically detect personality changes and set transition flags
    state
        .update_activity(activity, current_job, personality)
        .await
        .with_context(|| format!("Failed to update activity state for session {session_id}"))?;

    Ok(())
}

/// Handle user prompt submit events by resetting the error count.
///
/// # Errors
///
/// This function will return an error if:
/// - No input is received from Claude Code via stdin
/// - The input JSON is malformed or cannot be parsed
/// - Session state cannot be loaded or saved
/// - Error reset operations fail
async fn handle_prompt_submit() -> Result<()> {
    use anyhow::Context;

    // Read JSON from stdin to get session_id
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .with_context(|| "Failed to read prompt submit hook input from stdin")?;

    let hook_input: HookInput = serde_json::from_str(&input)
        .with_context(|| "Failed to parse prompt submit hook input JSON")?;
    let session_id = hook_input
        .session_id
        .unwrap_or_else(|| "unknown".to_string());

    // Reset error count
    let mut state = SessionState::load(&session_id).await.with_context(|| {
        format!("Failed to load session state for error reset (session: {session_id})")
    })?;
    state
        .reset_errors()
        .await
        .with_context(|| format!("Failed to reset error count for session {session_id}"))?;

    Ok(())
}

/// Handle session end events by cleaning up temporary session files.
///
/// # Errors
///
/// This function will return an error if:
/// - No input is received from Claude Code via stdin
/// - The input JSON is malformed or cannot be parsed
/// - Session file cleanup operations fail
async fn handle_session_end() -> Result<()> {
    use anyhow::Context;

    // Read JSON from stdin to get session_id
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .with_context(|| "Failed to read session end hook input from stdin")?;

    let hook_input: HookInput = serde_json::from_str(&input)
        .with_context(|| "Failed to parse session end hook input JSON")?;
    let session_id = hook_input
        .session_id
        .unwrap_or_else(|| "unknown".to_string());

    // Cleanup session files
    SessionState::cleanup(&session_id)
        .await
        .with_context(|| format!("Failed to cleanup session files for session {session_id}"))?;

    Ok(())
}

/// Helper function for case-insensitive extension checking
fn has_extension(file: &str, extensions: &[&str]) -> bool {
    let path = Path::new(file);
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy();
        extensions
            .iter()
            .any(|&expected| ext_str.eq_ignore_ascii_case(expected))
    } else {
        false
    }
}

fn extract_tool_params(
    tool_input: Option<&serde_json::Value>,
) -> (Option<String>, Option<String>, Option<String>) {
    if let Some(input) = tool_input {
        let file_path = input
            .get("file_path")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let command = input
            .get("command")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let pattern = input
            .get("pattern")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        (file_path, command, pattern)
    } else {
        (None, None, None)
    }
}

fn determine_activity(
    tool_name: &str,
    file_path: Option<&str>,
    command: Option<&str>,
    pattern: Option<&str>,
) -> (Activity, Option<String>) {
    match tool_name {
        "Edit" | "MultiEdit" => {
            let job = file_path.map(|f| trim_filename(f, 20));

            // Detect refactoring: MultiEdit usually indicates mass changes
            if tool_name == "MultiEdit" {
                return (Activity::Refactoring, job);
            }

            // Check file type for specific activities
            if let Some(path) = file_path {
                if is_documentation_file(path) {
                    return (Activity::Documenting, job);
                } else if is_config_file(path) {
                    return (Activity::Configuring, job);
                } else if is_code_file(path) {
                    return (Activity::Coding, job);
                }
            }

            (Activity::Editing, job)
        }
        "Write" => {
            let job = file_path.map(|f| trim_filename(f, 20));

            // Check file type for specific activities
            if let Some(path) = file_path {
                if is_documentation_file(path) {
                    return (Activity::Documenting, job);
                } else if is_config_file(path) {
                    return (Activity::Configuring, job);
                } else if is_code_file(path) {
                    return (Activity::Coding, job);
                }
            }

            (Activity::Writing, job)
        }
        "Bash" => {
            if let Some(cmd) = command {
                let job = Some(cmd.split_whitespace().next().unwrap_or("bash").to_string());

                if is_install_command(cmd) {
                    (Activity::Installing, job)
                } else if is_build_command(cmd) {
                    (Activity::Building, job)
                } else if is_test_command(cmd) {
                    (Activity::Testing, job)
                } else if is_deploy_command(cmd) {
                    (Activity::Deploying, job)
                } else if is_file_navigation_command(cmd) {
                    (Activity::Navigating, job)
                } else {
                    (Activity::Executing, job)
                }
            } else {
                (Activity::Executing, None)
            }
        }
        "Read" => {
            let job = file_path.map(|f| trim_filename(f, 20));
            (Activity::Reading, job)
        }
        "Grep" => {
            let job = pattern.map(|p| {
                if p.len() > 20 {
                    format!("{}...", &p[..17])
                } else {
                    p.to_string()
                }
            });
            (Activity::Searching, job)
        }
        _ => (Activity::Idle, None),
    }
}

fn trim_filename(name: &str, max_len: usize) -> String {
    // Get just the filename without path
    let name = std::path::Path::new(name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(name);

    if name.len() <= max_len {
        return name.to_string();
    }

    // If filename is too long, truncate it but keep extension
    if let Some(dot_pos) = name.rfind('.') {
        let ext = &name[dot_pos..];
        let base = &name[..dot_pos];
        let keep_len = max_len.saturating_sub(ext.len() + 3); // -3 for "..."

        if keep_len > 0 {
            format!("{}...{}", &base[..keep_len.min(base.len())], ext)
        } else {
            name[..max_len.min(name.len())].to_string()
        }
    } else {
        name[..max_len.min(name.len())].to_string()
    }
}

fn is_install_command(cmd: &str) -> bool {
    cmd.contains(" install") || cmd.contains(" add")
}

fn is_build_command(cmd: &str) -> bool {
    cmd.contains(" build") || cmd.contains(" compile") || cmd.contains("make ")
}

fn is_test_command(cmd: &str) -> bool {
    cmd.contains("test") || cmd.contains("spec")
}

fn is_file_navigation_command(cmd: &str) -> bool {
    let first_word = cmd.split_whitespace().next().unwrap_or("");
    matches!(
        first_word,
        "ls" | "cd" | "pwd" | "find" | "tree" | "mkdir" | "rmdir" | "mv" | "cp" | "rm"
    )
}

fn is_deploy_command(cmd: &str) -> bool {
    let cmd_lower = cmd.to_lowercase();
    cmd_lower.contains("deploy")
        || cmd_lower.contains("docker")
        || cmd_lower.contains("kubectl")
        || cmd_lower.contains("k8s")
        || cmd_lower.contains("helm")
        || cmd_lower.contains("terraform")
        || cmd_lower.contains("ansible")
        || cmd_lower.contains("serverless")
        || cmd_lower.contains("sls ")
        || cmd_lower.contains("vercel")
        || cmd_lower.contains("netlify")
        || cmd_lower.contains("heroku")
        || cmd_lower.contains("aws ")
        || cmd_lower.contains("gcloud")
        || cmd_lower.contains("azure")
}

fn is_documentation_file(path: &str) -> bool {
    let lowercase_path = path.to_lowercase();

    // Documentation file patterns
    if lowercase_path.contains("readme")
        || lowercase_path.contains("docs/")
        || lowercase_path.contains("documentation")
        || lowercase_path.contains("guide")
        || lowercase_path.contains("tutorial")
        || lowercase_path.contains("changelog")
        || lowercase_path.contains("license")
        || lowercase_path.contains("contributing")
        || lowercase_path.contains("api-")
    {
        return true;
    }

    // Documentation extensions
    has_extension(path, &["md", "rst", "txt", "adoc", "asciidoc"])
}

fn is_config_file(path: &str) -> bool {
    let lowercase_path = path.to_lowercase();

    // Check filename patterns - comprehensive list of config files
    if lowercase_path.contains("config")
        || lowercase_path.contains("settings")
        || lowercase_path.contains(".env")
        || lowercase_path.contains("dockerfile")
        || lowercase_path.contains("makefile")
        || lowercase_path.contains("package.json")
        || lowercase_path.contains("tsconfig")
        || lowercase_path.contains("webpack")
        || lowercase_path.contains("babel")
        || lowercase_path.contains("eslint")
        || lowercase_path.contains("prettier")
        || lowercase_path.contains("tailwind")
        || lowercase_path.contains("nuxt.config")
        || lowercase_path.contains("vue.config")
        || lowercase_path.contains("vite.config")
        || lowercase_path.contains("vitest.config")
        || lowercase_path.contains("jest.config")
        || lowercase_path.contains("next.config")
        || lowercase_path.contains("remix.config")
        || lowercase_path.contains("svelte.config")
        || lowercase_path.contains("rollup.config")
        || lowercase_path.contains("postcss.config")
        || lowercase_path.contains("gatsby.config")
        || lowercase_path.contains("astro.config")
        || lowercase_path.contains("cargo.toml")
        || lowercase_path.contains("pyproject.toml")
        || lowercase_path.contains("requirements.txt")
        || lowercase_path.contains("pipfile")
        || lowercase_path.contains("poetry.lock")
        || lowercase_path.contains("yarn.lock")
        || lowercase_path.contains("package-lock.json")
        || lowercase_path.contains("pnpm-lock.yaml")
        || lowercase_path.contains("go.mod")
        || lowercase_path.contains("go.sum")
        || lowercase_path.contains("composer.json")
        || lowercase_path.contains("gemfile")
        || lowercase_path.contains("podfile")
        || lowercase_path.contains("build.gradle")
        || lowercase_path.contains("pom.xml")
        || lowercase_path.contains("cmake")
    {
        return true;
    }

    // Check extensions (case-insensitive)
    has_extension(
        path,
        &[
            "json",
            "yaml",
            "yml",
            "toml",
            "ini",
            "conf",
            "cfg",
            "properties",
            "plist",
            "xml",
        ],
    )
}

fn is_code_file(path: &str) -> bool {
    // Programming language extensions - comprehensive list (case-insensitive)
    has_extension(
        path,
        &[
            // Web/JavaScript
            "js", "ts", "jsx", "tsx", "vue", "svelte", "astro", // Systems
            "rs", "c", "cpp", "cc", "cxx", "c++", "h", "hpp", "hxx", "h++", "go", "zig", "v",
            // JVM
            "java", "kt", "kts", "scala", "clj", "cljs", "cljc", "groovy", "gradle",
            // Functional
            "hs", "lhs", "ml", "mli", "elm", "purs", "fs", // Dynamic
            "py", "pyx", "pyi", "rb", "php", "pl", "pm", "lua", "r", "R", "jl",
            // Mobile/Cross-platform
            "swift", "dart", "cs", "vb", "m", "mm", // Scripts
            "sh", "bash", "zsh", "fish", "ps1", // Emerging
            "nim", "crystal", "cr", "ex", "exs", "erl", "hrl", "d", // Data/Query
            "sql", "graphql", "gql", "matlab", // Hardware
            "vhdl",
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_session_id() -> String {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        format!(
            "test_hook_{}_{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        )
    }

    #[test]
    fn test_extract_tool_params() {
        // Test with file_path
        let input = json!({
            "file_path": "/path/to/test.js",
            "command": "npm test",
            "pattern": "function.*test"
        });

        let (file_path, command, pattern) = extract_tool_params(Some(&input));
        assert_eq!(file_path, Some("/path/to/test.js".to_string()));
        assert_eq!(command, Some("npm test".to_string()));
        assert_eq!(pattern, Some("function.*test".to_string()));

        // Test with empty input
        let (file_path, command, pattern) = extract_tool_params(None);
        assert_eq!(file_path, None);
        assert_eq!(command, None);
        assert_eq!(pattern, None);
    }

    #[test]
    fn test_determine_activity() {
        // Edit operations
        let (activity, job) = determine_activity(
            "Edit",
            Some("/very/long/path/to/some/deeply/nested/file.js"),
            None,
            None,
        );
        assert_eq!(activity, Activity::Coding); // .js files should be detected as coding
        assert!(job.is_some());
        let job = job.unwrap();
        assert!(job.len() <= 20); // Should be trimmed
        assert!(job.contains("file.js"));

        // Write operations
        let (activity, job) = determine_activity("Write", Some("README.md"), None, None);
        assert_eq!(activity, Activity::Documenting); // README.md should be detected as documentation
        assert_eq!(job, Some("README.md".to_string()));

        // Bash operations
        let (activity, job) = determine_activity("Bash", None, Some("npm install express"), None);
        assert_eq!(activity, Activity::Installing);
        assert_eq!(job, Some("npm".to_string()));

        let (activity, job) = determine_activity("Bash", None, Some("cargo build --release"), None);
        assert_eq!(activity, Activity::Building);
        assert_eq!(job, Some("cargo".to_string()));

        let (activity, job) = determine_activity("Bash", None, Some("pytest tests/"), None);
        assert_eq!(activity, Activity::Testing);
        assert_eq!(job, Some("pytest".to_string()));

        // Read operations
        let (activity, job) = determine_activity("Read", Some("config.yaml"), None, None);
        assert_eq!(activity, Activity::Reading);
        assert_eq!(job, Some("config.yaml".to_string()));

        // Grep operations
        let (activity, job) = determine_activity("Grep", None, None, Some("function handleClick"));
        assert_eq!(activity, Activity::Searching);
        assert_eq!(job, Some("function handleClick".to_string()));

        // Long pattern should be truncated
        let long_pattern = "this is a very long search pattern that should be truncated";
        let (activity, job) =
            determine_activity("Grep", None, None, Some(long_pattern.to_string().as_ref()));
        assert_eq!(activity, Activity::Searching);
        let job = job.unwrap();
        assert!(job.len() <= 23); // 20 + "..."
        assert!(job.ends_with("..."));

        // Unknown tool
        let (activity, job) = determine_activity("UnknownTool", None, None, None);
        assert_eq!(activity, Activity::Idle);
        assert_eq!(job, None);
    }

    #[test]
    fn test_trim_filename() {
        // Short filename
        assert_eq!(trim_filename("test.js", 20), "test.js");

        // Long filename with extension
        let long_name = "very_long_filename_that_exceeds_limit.js";
        let trimmed = trim_filename(long_name, 20);
        assert!(trimmed.len() <= 20);
        assert!(trimmed.ends_with(".js"));
        assert!(trimmed.contains("..."));

        // Filename with path
        let path = "/path/to/some/file.txt";
        let trimmed = trim_filename(path, 20);
        assert_eq!(trimmed, "file.txt");

        // Very long filename without extension
        let no_ext = "very_long_filename_without_extension_that_goes_on_and_on";
        let trimmed = trim_filename(no_ext, 15);
        assert!(trimmed.len() <= 15);

        // Edge case: max_len smaller than extension
        let short_limit = trim_filename("file.extension", 5);
        assert!(short_limit.len() <= 5);
    }

    #[test]
    fn test_command_classification() {
        // Install commands
        assert!(is_install_command("npm install lodash"));
        assert!(is_install_command("cargo add serde"));
        assert!(is_install_command("pip add requests"));
        assert!(!is_install_command("npm test"));

        // Build commands
        assert!(is_build_command("npm run build"));
        assert!(is_build_command("cargo build"));
        assert!(is_build_command("make compile"));
        assert!(!is_build_command("npm install"));

        // Test commands
        assert!(is_test_command("npm test"));
        assert!(is_test_command("cargo test"));
        assert!(is_test_command("python -m pytest"));
        assert!(is_test_command("jest spec"));
        assert!(!is_test_command("npm build"));

        // File navigation commands
        assert!(is_file_navigation_command("ls -la"));
        assert!(is_file_navigation_command("cd /path"));
        assert!(is_file_navigation_command("mkdir new_dir"));
        assert!(is_file_navigation_command("rm file.txt"));
        assert!(!is_file_navigation_command("npm install"));

        // Config file detection
        assert!(is_config_file("package.json"));
        assert!(is_config_file("config.yaml"));
        assert!(is_config_file("settings.yml"));
        assert!(is_config_file("Cargo.toml"));
        assert!(is_config_file("app.config.js"));
        assert!(is_config_file(".env"));
        assert!(is_config_file("Dockerfile"));
        assert!(!is_config_file("main.js"));
        assert!(!is_config_file("README.md"));

        // Code file detection
        assert!(is_code_file("main.rs"));
        assert!(is_code_file("app.js"));
        assert!(is_code_file("component.tsx"));
        assert!(is_code_file("script.py"));
        assert!(is_code_file("Main.java"));
        assert!(!is_code_file("README.md"));
        assert!(!is_code_file("config.json"));
    }

    #[tokio::test]
    async fn test_handle_tool_hook_edit() {
        let session_id = create_test_session_id();

        // Simulate Edit tool hook
        let hook_input = HookInput {
            session_id: Some(session_id.clone()),
            tool_name: Some("Edit".to_string()),
            tool_input: Some(json!({
                "file_path": "main.js"
            })),
            tool_response: None,
        };

        let _input_json = serde_json::to_string(&hook_input).unwrap();

        // This would normally be called by the hook handler
        // We'll simulate the key parts
        let _state = SessionState::load(&session_id).await.unwrap();

        // Simulate the logic from handle_tool_hook
        let (file_path, command, pattern) = extract_tool_params(hook_input.tool_input.as_ref());
        let (activity, current_job) = determine_activity(
            &hook_input.tool_name.unwrap(),
            file_path.as_deref(),
            command.as_deref(),
            pattern.as_deref(),
        );

        assert_eq!(activity, Activity::Coding); // main.js should be detected as coding
        assert_eq!(current_job, Some("main.js".to_string()));

        // Cleanup
        SessionState::cleanup(&session_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_handle_tool_hook_with_error() {
        let session_id = create_test_session_id();

        // Simulate tool hook with error
        let hook_input = HookInput {
            session_id: Some(session_id.clone()),
            tool_name: Some("Bash".to_string()),
            tool_input: Some(json!({
                "command": "failing_command"
            })),
            tool_response: Some(ToolResponse {
                error: Some(json!("Command failed")),
            }),
        };

        let mut state = SessionState::load(&session_id).await.unwrap();
        let initial_errors = state.error_count;

        // Simulate error handling
        if hook_input.tool_response.as_ref().unwrap().error.is_some() {
            state.increment_errors().await.unwrap();
        }

        assert_eq!(state.error_count, initial_errors + 1);

        // Cleanup
        SessionState::cleanup(&session_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_handle_prompt_submit() {
        let session_id = create_test_session_id();

        // Set up state with errors
        let mut state = SessionState::load(&session_id).await.unwrap();
        state.increment_errors().await.unwrap();
        state.increment_errors().await.unwrap();
        assert_eq!(state.error_count, 2);

        // Simulate prompt submit (should reset errors)
        state.reset_errors().await.unwrap();
        assert_eq!(state.error_count, 0);

        // Cleanup
        SessionState::cleanup(&session_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_handle_session_end() {
        let session_id = create_test_session_id();

        // Create some state
        let state = SessionState::load(&session_id).await.unwrap();
        state.save().await.unwrap();

        // Verify file exists
        let state_path = SessionState::get_state_path(&session_id);
        assert!(state_path.exists());

        // Simulate session end cleanup
        SessionState::cleanup(&session_id).await.unwrap();

        // Verify file is cleaned up
        assert!(!state_path.exists());
    }

    #[test]
    fn test_hook_input_parsing() {
        let json_str = r#"{
            "session_id": "test_123",
            "tool_name": "Edit",
            "tool_input": {
                "file_path": "test.js"
            },
            "tool_response": {
                "error": null
            }
        }"#;

        let hook_input: HookInput = serde_json::from_str(json_str).unwrap();
        assert_eq!(hook_input.session_id, Some("test_123".to_string()));
        assert_eq!(hook_input.tool_name, Some("Edit".to_string()));
        assert!(hook_input.tool_input.is_some());
        assert!(hook_input.tool_response.is_some());
    }

    #[test]
    fn test_bash_command_activity_detection() {
        // Package management
        let (activity, _) = determine_activity("Bash", None, Some("npm install express"), None);
        assert_eq!(activity, Activity::Installing);

        let (activity, _) = determine_activity("Bash", None, Some("pnpm add typescript"), None);
        assert_eq!(activity, Activity::Installing);

        // Build commands
        let (activity, _) = determine_activity("Bash", None, Some("npm run build"), None);
        assert_eq!(activity, Activity::Building);

        let (activity, _) = determine_activity("Bash", None, Some("cargo build --release"), None);
        assert_eq!(activity, Activity::Building);

        // Test commands
        let (activity, _) = determine_activity("Bash", None, Some("npm test"), None);
        assert_eq!(activity, Activity::Testing);

        let (activity, _) = determine_activity("Bash", None, Some("pytest spec/"), None);
        assert_eq!(activity, Activity::Testing);

        // File navigation
        let (activity, _) = determine_activity("Bash", None, Some("ls -la"), None);
        assert_eq!(activity, Activity::Navigating);

        let (activity, _) = determine_activity("Bash", None, Some("cd /path/to/dir"), None);
        assert_eq!(activity, Activity::Navigating);

        // Generic execution
        let (activity, _) = determine_activity("Bash", None, Some("echo hello"), None);
        assert_eq!(activity, Activity::Executing);
    }

    #[test]
    fn test_activity_detection_with_file_types() {
        // Test config file editing
        let (activity, job) = determine_activity("Edit", Some("package.json"), None, None);
        assert_eq!(activity, Activity::Configuring);
        assert_eq!(job, Some("package.json".to_string()));

        // Test code file editing
        let (activity, job) = determine_activity("Edit", Some("main.rs"), None, None);
        assert_eq!(activity, Activity::Coding);
        assert_eq!(job, Some("main.rs".to_string()));

        // Test writing config files
        let (activity, job) = determine_activity("Write", Some("Cargo.toml"), None, None);
        assert_eq!(activity, Activity::Configuring);
        assert_eq!(job, Some("Cargo.toml".to_string()));

        // Test writing code files
        let (activity, job) = determine_activity("Write", Some("component.tsx"), None, None);
        assert_eq!(activity, Activity::Coding);
        assert_eq!(job, Some("component.tsx".to_string()));

        // Test markdown file (should be detected as documenting)
        let (activity, job) = determine_activity("Edit", Some("README.md"), None, None);
        assert_eq!(activity, Activity::Documenting); // README.md should be documentation
        assert_eq!(job, Some("README.md".to_string()));
    }
}
