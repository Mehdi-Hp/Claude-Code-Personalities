use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;

use crate::types::Activity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoodState {
    pub frustration_level: u8,        // 0-10, increases with errors
    pub momentum: u8,                 // 0-10, consecutive successes
    pub last_error_time: Option<u64>, // Unix timestamp
}

impl Default for MoodState {
    fn default() -> Self {
        Self {
            frustration_level: 0,
            momentum: 0,
            last_error_time: None,
        }
    }
}

impl MoodState {
    /// Update mood based on error occurrence
    pub fn update(&mut self, had_error: bool) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if had_error {
            self.frustration_level = (self.frustration_level + 2).min(10);
            self.momentum = 0;
            self.last_error_time = Some(now);
        } else {
            // Gradual frustration decay over time
            if let Some(last_error) = self.last_error_time {
                let minutes_since_error = (now - last_error) / 60;
                if minutes_since_error > 5 {
                    self.frustration_level = self.frustration_level.saturating_sub(1);
                }
            } else {
                self.frustration_level = self.frustration_level.saturating_sub(1);
            }

            self.momentum = (self.momentum + 1).min(10);
        }
    }

    /// Get personality modifier based on current mood
    pub fn get_personality_modifier(&self) -> PersonalityModifier {
        match (self.frustration_level, self.momentum) {
            (6..=10, _) => PersonalityModifier::Frustrated,
            (_, 8..=10) => PersonalityModifier::InTheZone,
            _ => PersonalityModifier::Normal,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PersonalityModifier {
    Frustrated,
    InTheZone,
    Normal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub session_id: String,
    pub activity: Activity,
    pub current_job: Option<String>,
    pub personality: String,
    pub previous_personality: Option<String>,
    pub consecutive_actions: u32,
    pub error_count: u32,
    #[serde(default)]
    pub recent_activities: Vec<Activity>,
    #[serde(default)]
    pub mood: MoodState,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            session_id: "unknown".to_string(),
            activity: Activity::Idle,
            current_job: None,
            personality: "( ˘ ³˘) Booting Up".to_string(),
            previous_personality: None,
            consecutive_actions: 0,
            error_count: 0,
            recent_activities: Vec::new(),
            mood: MoodState::default(),
        }
    }
}

impl SessionState {
    /// Load session state from disk or create default state.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The state file cannot be read due to permissions or I/O errors
    /// - The state file contains invalid JSON that cannot be parsed
    /// - File system operations fail during read
    pub async fn load(session_id: &str) -> Result<Self> {
        use anyhow::Context;

        let path = Self::get_state_path(session_id);

        if path.exists() {
            let content = fs::read_to_string(&path)
                .await
                .with_context(|| format!("Failed to read session state from {}", path.display()))?;
            let state: SessionState = serde_json::from_str(&content).with_context(|| {
                format!("Invalid session state format for session {session_id}")
            })?;
            Ok(state)
        } else {
            Ok(Self {
                session_id: session_id.to_string(),
                ..Default::default()
            })
        }
    }

    /// Save the current session state to disk.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - JSON serialization fails
    /// - The state file cannot be written due to permissions or I/O errors
    /// - File system operations fail during write
    pub async fn save(&self) -> Result<()> {
        use anyhow::Context;

        let path = Self::get_state_path(&self.session_id);
        let content = serde_json::to_string_pretty(self).with_context(|| {
            format!(
                "Failed to serialize session state for session {}",
                self.session_id
            )
        })?;
        fs::write(&path, content)
            .await
            .with_context(|| format!("Failed to save session state to {}", path.display()))?;
        Ok(())
    }

    /// Update the current activity and personality, then save to disk.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The session state cannot be saved to disk after updating
    /// - File system operations fail during save
    /// - JSON serialization fails
    pub async fn update_activity(
        &mut self,
        activity: Activity,
        current_job: Option<String>,
        personality: String,
    ) -> Result<()> {
        use anyhow::Context;

        // Update consecutive actions
        if self.activity == activity {
            self.consecutive_actions += 1;
        } else {
            self.consecutive_actions = 1;
        }

        // Check for personality change
        if self.personality != personality {
            self.previous_personality = Some(self.personality.clone());
        }

        self.activity = activity;
        self.current_job = current_job;
        self.personality = personality;

        // Update mood for successful activity (no error)
        self.mood.update(false);

        self.save().await.with_context(|| {
            format!(
                "Failed to save updated activity for session {}",
                self.session_id
            )
        })
    }

    /// Increment the error count and save to disk.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The session state cannot be saved to disk after incrementing
    /// - File system operations fail during save
    /// - JSON serialization fails
    pub async fn increment_errors(&mut self) -> Result<()> {
        use anyhow::Context;

        self.error_count += 1;
        self.mood.update(true); // Update mood for error
        self.save().await.with_context(|| {
            format!(
                "Failed to save incremented error count for session {}",
                self.session_id
            )
        })
    }

    /// Reset the error count to zero and save to disk.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The session state cannot be saved to disk after resetting
    /// - File system operations fail during save
    /// - JSON serialization fails
    pub async fn reset_errors(&mut self) -> Result<()> {
        use anyhow::Context;

        self.error_count = 0;
        self.save().await.with_context(|| {
            format!(
                "Failed to save reset error count for session {}",
                self.session_id
            )
        })
    }

    /// Clean up session state files for the given session ID.
    ///
    /// This function removes both the state file and error count file.
    /// Missing files are ignored and will not cause an error.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - This function currently does not return errors as file removal failures are ignored
    pub async fn cleanup(session_id: &str) -> Result<()> {
        let state_path = Self::get_state_path(session_id);
        let error_path = Self::get_error_path(session_id);

        // Ignore errors if files don't exist
        let _ = fs::remove_file(&state_path).await;
        let _ = fs::remove_file(&error_path).await;

        Ok(())
    }

    #[must_use]
    pub fn get_state_path(session_id: &str) -> PathBuf {
        PathBuf::from(format!(
            "/tmp/claude_code_personalities_activity_{session_id}.json"
        ))
    }

    fn get_error_path(session_id: &str) -> PathBuf {
        PathBuf::from(format!(
            "/tmp/claude_code_personalities_errors_{session_id}.count"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create unique test session IDs
    fn create_test_session_id() -> String {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        format!(
            "test_session_{}_{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        )
    }

    #[tokio::test]
    async fn test_default_state() {
        let state = SessionState::default();
        assert_eq!(state.session_id, "unknown");
        assert_eq!(state.activity, Activity::Idle);
        assert_eq!(state.personality, "( ˘ ³˘) Booting Up");
        assert!(state.previous_personality.is_none());
        assert_eq!(state.consecutive_actions, 0);
        assert_eq!(state.error_count, 0);
        assert!(state.current_job.is_none());
    }

    #[tokio::test]
    async fn test_load_nonexistent_state() {
        let session_id = create_test_session_id();
        let state = SessionState::load(&session_id).await.unwrap();

        // Should create default state with correct session_id
        assert_eq!(state.session_id, session_id);
        assert_eq!(state.activity, Activity::Idle);
        assert_eq!(state.personality, "( ˘ ³˘) Booting Up");
        assert_eq!(state.consecutive_actions, 0);
        assert_eq!(state.error_count, 0);
    }

    #[tokio::test]
    async fn test_save_and_load_state() {
        let session_id = create_test_session_id();

        // Create and save a state
        let state = SessionState {
            session_id: session_id.clone(),
            activity: Activity::Editing,
            current_job: Some("test.js".to_string()),
            personality: "Code Wizard".to_string(),
            previous_personality: None,
            consecutive_actions: 5,
            error_count: 2,
            recent_activities: Vec::new(),
            mood: MoodState::default(),
        };

        state.save().await.unwrap();

        // Load it back
        let loaded_state = SessionState::load(&session_id).await.unwrap();

        assert_eq!(loaded_state.session_id, session_id);
        assert_eq!(loaded_state.activity, Activity::Editing);
        assert_eq!(loaded_state.current_job, Some("test.js".to_string()));
        assert_eq!(loaded_state.personality, "Code Wizard");
        assert_eq!(loaded_state.consecutive_actions, 5);
        assert_eq!(loaded_state.error_count, 2);

        // Cleanup
        SessionState::cleanup(&session_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_update_activity() {
        let session_id = create_test_session_id();
        let mut state = SessionState::load(&session_id).await.unwrap();

        // First update
        state
            .update_activity(
                Activity::Editing,
                Some("main.js".to_string()),
                "JS Master".to_string(),
            )
            .await
            .unwrap();

        assert_eq!(state.activity, Activity::Editing);
        assert_eq!(state.current_job, Some("main.js".to_string()));
        assert_eq!(state.personality, "JS Master");
        assert_eq!(state.consecutive_actions, 1);

        // Same activity should increment consecutive
        state
            .update_activity(
                Activity::Editing,
                Some("utils.js".to_string()),
                "JS Master".to_string(),
            )
            .await
            .unwrap();

        assert_eq!(state.consecutive_actions, 2);

        // Different activity should reset consecutive
        state
            .update_activity(
                Activity::Reading,
                Some("README.md".to_string()),
                "Documentation Writer".to_string(),
            )
            .await
            .unwrap();

        assert_eq!(state.activity, Activity::Reading);
        assert_eq!(state.consecutive_actions, 1);

        // Cleanup
        SessionState::cleanup(&session_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_error_counting() {
        let session_id = create_test_session_id();
        let mut state = SessionState::load(&session_id).await.unwrap();

        assert_eq!(state.error_count, 0);

        // Increment errors
        state.increment_errors().await.unwrap();
        assert_eq!(state.error_count, 1);

        state.increment_errors().await.unwrap();
        assert_eq!(state.error_count, 2);

        // Reset errors
        state.reset_errors().await.unwrap();
        assert_eq!(state.error_count, 0);

        // Cleanup
        SessionState::cleanup(&session_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_state_persistence() {
        let session_id = create_test_session_id();

        // Create and modify state
        {
            let mut state = SessionState::load(&session_id).await.unwrap();
            state
                .update_activity(Activity::Testing, None, "Test Engineer".to_string())
                .await
                .unwrap();
            state.increment_errors().await.unwrap();
            state.increment_errors().await.unwrap();
            // State is automatically saved by update_activity and increment_errors
        }

        // Load from a fresh instance
        {
            let state = SessionState::load(&session_id).await.unwrap();
            assert_eq!(state.activity, Activity::Testing);
            assert_eq!(state.personality, "Test Engineer");
            assert_eq!(state.error_count, 2);
            assert_eq!(state.consecutive_actions, 1);
        }

        // Cleanup
        SessionState::cleanup(&session_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_cleanup() {
        let session_id = create_test_session_id();

        // Create state file
        let state = SessionState::load(&session_id).await.unwrap();
        state.save().await.unwrap();

        // Verify file exists
        let state_path = SessionState::get_state_path(&session_id);
        assert!(state_path.exists());

        // Cleanup
        SessionState::cleanup(&session_id).await.unwrap();

        // Verify file is gone
        assert!(!state_path.exists());
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let session_id = create_test_session_id();

        // Create initial state to avoid race condition with file creation
        let initial_state = SessionState::load(&session_id).await.unwrap();
        initial_state.save().await.unwrap();

        // Simulate concurrent updates with delay to reduce race conditions
        let handles: Vec<_> = (0..3)
            .map(|i| {
                let session_id = session_id.clone();
                tokio::spawn(async move {
                    // Add small delay to reduce concurrent file access
                    tokio::time::sleep(std::time::Duration::from_millis(i * 10)).await;

                    let mut state = SessionState::load(&session_id).await.unwrap();
                    state
                        .update_activity(
                            Activity::parse_activity(&format!("activity_{i}")),
                            Some(format!("file_{i}.js")),
                            format!("Personality {i}"),
                        )
                        .await
                        .unwrap();
                    state.increment_errors().await.unwrap();
                })
            })
            .collect();

        // Wait for all to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Load final state
        let final_state = SessionState::load(&session_id).await.unwrap();

        // Should have some activity and errors
        assert!(final_state.error_count > 0);
        assert_ne!(final_state.activity, Activity::Idle);

        // Cleanup
        SessionState::cleanup(&session_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_invalid_json_handling() {
        let session_id = create_test_session_id();
        let state_path = SessionState::get_state_path(&session_id);

        // Write invalid JSON
        fs::write(&state_path, "invalid json").await.unwrap();

        // Should return error for invalid JSON
        let result = SessionState::load(&session_id).await;
        assert!(result.is_err());

        // Cleanup
        if state_path.exists() {
            fs::remove_file(&state_path).await.unwrap();
        }
    }
}
