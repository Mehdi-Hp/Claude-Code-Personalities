use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

use crate::types::Activity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub session_id: String,
    pub activity: Activity,
    pub current_job: Option<String>,
    pub personality: String,
    pub previous_personality: Option<String>,
    pub consecutive_actions: u32,
    pub error_count: u32,
    #[serde(skip)]
    pub animation_state: AnimationState,
}

#[derive(Debug, Clone)]
pub struct AnimationState {
    pub is_animating: bool,
    pub animation_type: Option<String>,
    pub should_transition: bool,
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
            animation_state: AnimationState::default(),
        }
    }
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            is_animating: false,
            animation_type: None,
            should_transition: false,
        }
    }
}

impl SessionState {
    pub async fn load(session_id: &str) -> Result<Self> {
        use anyhow::Context;
        
        let path = Self::get_state_path(session_id);
        
        if path.exists() {
            let content = fs::read_to_string(&path).await
                .with_context(|| format!("Failed to read session state from {}", path.display()))?;
            let state: SessionState = serde_json::from_str(&content)
                .with_context(|| format!("Invalid session state format for session {}", session_id))?;
            Ok(state)
        } else {
            let mut state = Self::default();
            state.session_id = session_id.to_string();
            Ok(state)
        }
    }
    
    pub async fn save(&self) -> Result<()> {
        use anyhow::Context;
        
        let path = Self::get_state_path(&self.session_id);
        let content = serde_json::to_string_pretty(self)
            .with_context(|| format!("Failed to serialize session state for session {}", self.session_id))?;
        fs::write(&path, content).await
            .with_context(|| format!("Failed to save session state to {}", path.display()))?;
        Ok(())
    }
    
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
        
        // Check for personality change to trigger transition
        if self.personality != personality {
            self.previous_personality = Some(self.personality.clone());
            self.animation_state.should_transition = true;
        }
        
        self.activity = activity;
        self.current_job = current_job;
        self.personality = personality;
        
        self.save().await
            .with_context(|| format!("Failed to save updated activity for session {}", self.session_id))
    }
    
    pub async fn increment_errors(&mut self) -> Result<()> {
        use anyhow::Context;
        
        self.error_count += 1;
        self.save().await
            .with_context(|| format!("Failed to save incremented error count for session {}", self.session_id))
    }
    
    pub async fn reset_errors(&mut self) -> Result<()> {
        use anyhow::Context;
        
        self.error_count = 0;
        self.save().await
            .with_context(|| format!("Failed to save reset error count for session {}", self.session_id))
    }
    
    /// Check if personality transition is needed
    pub fn should_show_transition(&self) -> bool {
        self.animation_state.should_transition && self.previous_personality.is_some()
    }
    
    /// Get personality transition info
    pub fn get_transition_info(&self) -> Option<(String, String)> {
        if let Some(prev) = &self.previous_personality {
            Some((prev.clone(), self.personality.clone()))
        } else {
            None
        }
    }
    
    /// Mark transition as consumed
    pub fn consume_transition(&mut self) {
        self.animation_state.should_transition = false;
    }
    
    /// Set animation state
    pub fn set_animation_state(&mut self, is_animating: bool, animation_type: Option<String>) {
        self.animation_state.is_animating = is_animating;
        self.animation_state.animation_type = animation_type;
    }
    
    pub async fn cleanup(session_id: &str) -> Result<()> {
        let state_path = Self::get_state_path(session_id);
        let error_path = Self::get_error_path(session_id);
        
        // Ignore errors if files don't exist
        let _ = fs::remove_file(&state_path).await;
        let _ = fs::remove_file(&error_path).await;
        
        Ok(())
    }
    
    pub fn get_state_path(session_id: &str) -> PathBuf {
        PathBuf::from(format!("/tmp/claude_activity_{}.json", session_id))
    }
    
    fn get_error_path(session_id: &str) -> PathBuf {
        PathBuf::from(format!("/tmp/claude_errors_{}.count", session_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create unique test session IDs
    fn create_test_session_id() -> String {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        format!("test_session_{}_{}", std::process::id(), COUNTER.fetch_add(1, Ordering::SeqCst))
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
        assert!(!state.animation_state.is_animating);
        assert!(!state.animation_state.should_transition);
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
            animation_state: AnimationState::default(),
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
        state.update_activity(
            Activity::Editing,
            Some("main.js".to_string()),
            "JS Master".to_string(),
        ).await.unwrap();
        
        assert_eq!(state.activity, Activity::Editing);
        assert_eq!(state.current_job, Some("main.js".to_string()));
        assert_eq!(state.personality, "JS Master");
        assert_eq!(state.consecutive_actions, 1);
        
        // Same activity should increment consecutive
        state.update_activity(
            Activity::Editing,
            Some("utils.js".to_string()),
            "JS Master".to_string(),
        ).await.unwrap();
        
        assert_eq!(state.consecutive_actions, 2);
        
        // Different activity should reset consecutive
        state.update_activity(
            Activity::Reading,
            Some("README.md".to_string()),
            "Documentation Writer".to_string(),
        ).await.unwrap();
        
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
            state.update_activity(
                Activity::Testing,
                None,
                "Test Engineer".to_string(),
            ).await.unwrap();
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
        let handles: Vec<_> = (0..3).map(|i| {
            let session_id = session_id.clone();
            tokio::spawn(async move {
                // Add small delay to reduce concurrent file access
                tokio::time::sleep(std::time::Duration::from_millis(i * 10)).await;
                
                let mut state = SessionState::load(&session_id).await.unwrap();
                state.update_activity(
                    Activity::from_str(&format!("activity_{}", i)),
                    Some(format!("file_{}.js", i)),
                    format!("Personality {}", i),
                ).await.unwrap();
                state.increment_errors().await.unwrap();
            })
        }).collect();
        
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
    
    #[tokio::test]
    async fn test_personality_transition_tracking() {
        let session_id = create_test_session_id();
        let mut state = SessionState::load(&session_id).await.unwrap();
        
        // Initial state should not have transition
        assert!(!state.should_show_transition());
        assert!(state.get_transition_info().is_none());
        
        // Update with same personality - no transition
        state.update_activity(
            Activity::Editing,
            Some("test.js".to_string()),
            state.personality.clone(),
        ).await.unwrap();
        assert!(!state.should_show_transition());
        
        // Update with different personality - should trigger transition
        let new_personality = "ʕ•ᴥ•ʔ Code Wizard".to_string();
        state.update_activity(
            Activity::Coding,
            Some("main.rs".to_string()),
            new_personality.clone(),
        ).await.unwrap();
        
        assert!(state.should_show_transition());
        let transition_info = state.get_transition_info();
        assert!(transition_info.is_some());
        let (from, to) = transition_info.unwrap();
        assert_eq!(from, "( ˘ ³˘) Booting Up");
        assert_eq!(to, new_personality);
        
        // Consume transition
        state.consume_transition();
        assert!(!state.should_show_transition());
        
        // Cleanup
        SessionState::cleanup(&session_id).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_animation_state_management() {
        let session_id = create_test_session_id();
        let mut state = SessionState::load(&session_id).await.unwrap();
        
        // Initial animation state
        assert!(!state.animation_state.is_animating);
        assert!(state.animation_state.animation_type.is_none());
        
        // Set animation state
        state.set_animation_state(true, Some("transition".to_string()));
        assert!(state.animation_state.is_animating);
        assert_eq!(state.animation_state.animation_type, Some("transition".to_string()));
        
        // Clear animation state
        state.set_animation_state(false, None);
        assert!(!state.animation_state.is_animating);
        assert!(state.animation_state.animation_type.is_none());
        
        // Cleanup
        SessionState::cleanup(&session_id).await.unwrap();
    }
}