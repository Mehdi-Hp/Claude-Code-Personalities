use std::io::Write;
use std::process::{Command, Stdio};

/// Helper to create a command with test config that uses defaults
fn cargo_run_with_test_config() -> Command {
    let mut cmd = Command::new("cargo");
    // Point to non-existent file so defaults are used
    cmd.env(
        "CLAUDE_PERSONALITIES_CONFIG",
        "/tmp/nonexistent_test_config.json",
    );
    cmd
}

#[tokio::test]
async fn test_statusline_end_to_end() {
    // Use a unique session ID each time to avoid cached state
    let unique_id = format!(
        "integration_test_{}",
        std::time::UNIX_EPOCH.elapsed().unwrap().as_nanos()
    );
    let input_json = format!(
        r#"{{
        "session_id": "{}",
        "model": {{
            "display_name": "Opus"
        }},
        "workspace": {{
            "current_dir": "/test/project",
            "project_dir": "/test/project"
        }}
    }}"#,
        unique_id
    );

    // Build the binary first
    let output = cargo_run_with_test_config()
        .args(["build", "--release"])
        .output()
        .expect("Failed to build binary");

    assert!(
        output.status.success(),
        "Failed to build binary: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Test statusline mode
    let mut child = cargo_run_with_test_config()
        .args(["run", "--", "--statusline"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start process");

    // Write input to stdin
    if let Some(stdin) = child.stdin.take() {
        let mut stdin = stdin;
        stdin
            .write_all(input_json.as_bytes())
            .expect("Failed to write to stdin");
        stdin.flush().expect("Failed to flush stdin");
    }

    // Get output
    let output = child.wait_with_output().expect("Failed to read stdout");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should succeed
    assert!(
        output.status.success(),
        "Process failed with stderr: {stderr}"
    );

    // Should contain a valid statusline
    assert!(!stdout.is_empty(), "No statusline output");
    assert!(
        stdout.contains("Chillin"),
        "Should contain default personality"
    );
    assert!(stdout.contains("Opus"), "Should contain model name");

    println!("Integration test statusline output: {}", stdout.trim());
}

#[tokio::test]
async fn test_hook_mode() {
    let hook_input = r#"{
        "session_id": "hook_test",
        "tool_name": "Edit",
        "tool_input": {
            "file_path": "test.js"
        },
        "tool_response": null
    }"#;

    // Test hook mode
    let mut child = cargo_run_with_test_config()
        .args(["run", "--", "--hook", "pre-tool"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start process");

    // Write input to stdin
    if let Some(stdin) = child.stdin.take() {
        let mut stdin = stdin;
        stdin
            .write_all(hook_input.as_bytes())
            .expect("Failed to write to stdin");
        stdin.flush().expect("Failed to flush stdin");
    }

    // Get output
    let output = child.wait_with_output().expect("Failed to read stdout");
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should succeed (hook mode doesn't usually produce stdout)
    assert!(
        output.status.success(),
        "Hook process failed with stderr: {stderr}"
    );

    // Now test that the state was created
    let statusline_input = r#"{
        "session_id": "hook_test",
        "model": {
            "display_name": "Sonnet"
        }
    }"#;

    let mut child = cargo_run_with_test_config()
        .args(["run", "--", "--statusline"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start process");

    // Write input to stdin
    if let Some(stdin) = child.stdin.take() {
        let mut stdin = stdin;
        stdin
            .write_all(statusline_input.as_bytes())
            .expect("Failed to write to stdin");
        stdin.flush().expect("Failed to flush stdin");
    }

    // Get output
    let output = child.wait_with_output().expect("Failed to read stdout");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should succeed
    assert!(
        output.status.success(),
        "Statusline process failed with stderr: {stderr}"
    );

    // Should show the personality that was set by the hook
    assert!(!stdout.is_empty(), "No statusline output after hook");
    assert!(
        stdout.contains("JS Master") || stdout.contains("Cowder"),
        "Should contain JS or Code personality for .js file. Got: {stdout}"
    );

    println!(
        "Integration test hook -> statusline output: {}",
        stdout.trim()
    );
}

#[test]
fn test_cli_help() {
    let output = cargo_run_with_test_config()
        .args(["run", "--", "help"])
        .output()
        .expect("Failed to run help command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain basic CLI structure
    assert!(
        stdout.contains("Claude Code Personalities")
            || stdout.contains("claude-code-personalities")
    );
}

#[test]
fn test_invalid_json_input() {
    let invalid_json = "not json at all";

    let mut child = cargo_run_with_test_config()
        .args(["run", "--", "--statusline"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start process");

    // Write invalid input to stdin
    if let Some(stdin) = child.stdin.take() {
        let mut stdin = stdin;
        stdin
            .write_all(invalid_json.as_bytes())
            .expect("Failed to write to stdin");
        stdin.flush().expect("Failed to flush stdin");
    }

    // Get output
    let output = child.wait_with_output().expect("Failed to read stdout");

    // Should fail gracefully
    assert!(!output.status.success(), "Should fail with invalid JSON");
}

#[tokio::test]
async fn test_personality_progression() {
    // Test that personalities change based on different tool usage
    let session_id = "personality_test";

    // 1. Start with JS file editing
    let js_edit = format!(
        r#"{{
        "session_id": "{session_id}",
        "tool_name": "Edit",
        "tool_input": {{"file_path": "app.js"}}
    }}"#
    );

    let mut child = cargo_run_with_test_config()
        .args(["run", "--", "--hook", "pre-tool"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start process");

    if let Some(stdin) = child.stdin.take() {
        let mut stdin = stdin;
        stdin
            .write_all(js_edit.as_bytes())
            .expect("Failed to write to stdin");
        stdin.flush().expect("Failed to flush stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");
    assert!(output.status.success());

    // 2. Check statusline shows JS personality
    let statusline_input = format!(
        r#"{{
        "session_id": "{session_id}",
        "model": {{"display_name": "Claude"}}
    }}"#
    );

    let mut child = cargo_run_with_test_config()
        .args(["run", "--", "--statusline"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start process");

    if let Some(stdin) = child.stdin.take() {
        let mut stdin = stdin;
        stdin
            .write_all(statusline_input.as_bytes())
            .expect("Failed to write to stdin");
        stdin.flush().expect("Failed to flush stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");
    let stdout1 = String::from_utf8_lossy(&output.stdout);

    // 3. Now edit a markdown file
    let md_edit = format!(
        r#"{{
        "session_id": "{session_id}",
        "tool_name": "Edit",
        "tool_input": {{"file_path": "README.md"}}
    }}"#
    );

    let mut child = cargo_run_with_test_config()
        .args(["run", "--", "--hook", "pre-tool"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start process");

    if let Some(stdin) = child.stdin.take() {
        let mut stdin = stdin;
        stdin
            .write_all(md_edit.as_bytes())
            .expect("Failed to write to stdin");
        stdin.flush().expect("Failed to flush stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");
    assert!(output.status.success());

    // 4. Check statusline shows documentation personality
    let mut child = cargo_run_with_test_config()
        .args(["run", "--", "--statusline"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start process");

    if let Some(stdin) = child.stdin.take() {
        let mut stdin = stdin;
        stdin
            .write_all(statusline_input.as_bytes())
            .expect("Failed to write to stdin");
        stdin.flush().expect("Failed to flush stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");
    let stdout2 = String::from_utf8_lossy(&output.stdout);

    // Verify personality changed from JS-related to Documentation
    println!("JS file statusline: {}", stdout1.trim());
    println!("MD file statusline: {}", stdout2.trim());

    assert!(
        stdout2.contains("Documentation Writer"),
        "Should show Documentation Writer for .md file. Got: {stdout2}"
    );
}
