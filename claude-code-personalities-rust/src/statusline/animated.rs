use anyhow::Result;
use std::time::Duration;
use tokio::time::{sleep, timeout};
use crossterm::{
    cursor,
    execute,
    terminal::{Clear, ClearType},
};
use std::io::{stdout, Write};

use crate::animation::{AnimationEngine, AnimationType, AnimationSpeed};
use crate::config::PersonalityPreferences;
use crate::state::SessionState;
use super::build_statusline;

/// Animated statusline renderer that handles transitions and activity animations
pub struct AnimatedStatuslineRenderer {
    animation_engine: AnimationEngine,
    is_rendering: bool,
}

impl AnimatedStatuslineRenderer {
    pub fn new(preferences: PersonalityPreferences) -> Self {
        Self {
            animation_engine: AnimationEngine::new(preferences),
            is_rendering: false,
        }
    }
    
    /// Render statusline with animation support
    pub async fn render_animated_statusline(
        &mut self,
        state: &mut SessionState,
        model_name: &str,
        prefs: &PersonalityPreferences,
    ) -> Result<String> {
        // Check if we should show a transition animation
        if state.should_show_transition() {
            if let Some((from, to)) = state.get_transition_info() {
                // Play transition animation
                let transition_type = AnimationType::Transition { from, to };
                self.play_transition_animation(transition_type, state, model_name, prefs).await?;
                
                // Mark transition as consumed
                state.consume_transition();
            }
        }
        
        // Check if we should show activity animation
        if prefs.enable_activity_animations && !state.animation_state.is_animating {
            let activity_type = AnimationType::Activity { activity: state.activity.clone() };
            if self.animation_engine.should_animate(&activity_type) {
                self.play_activity_animation(activity_type, state, model_name, prefs).await?;
            }
        }
        
        // If no animations are playing, render static statusline
        if !state.animation_state.is_animating {
            Ok(build_statusline(state, model_name, prefs))
        } else {
            // Render current animated frame
            let base_statusline = build_statusline(state, model_name, prefs);
            Ok(self.animation_engine.render_frame(&base_statusline, true).await
                .unwrap_or_else(|_| base_statusline))
        }
    }
    
    /// Play personality transition animation
    async fn play_transition_animation(
        &mut self,
        animation_type: AnimationType,
        state: &mut SessionState,
        model_name: &str,
        prefs: &PersonalityPreferences,
    ) -> Result<()> {
        if !prefs.enable_transitions {
            return Ok(());
        }
        
        state.set_animation_state(true, Some("transition".to_string()));
        self.is_rendering = true;
        
        // Create animation sequence for transition
        if let Some(sequence) = self.get_transition_sequence(&animation_type, state) {
            // Render transition frames
            for frame in &sequence.frames {
                if !self.is_rendering {
                    break;
                }
                
                // Build statusline with animated personality
                let animated_statusline = self.build_animated_statusline(
                    state,
                    model_name,
                    prefs,
                    &frame.content,
                )?;
                
                // Clear and render frame
                self.render_frame_to_terminal(&animated_statusline).await?;
                
                // Wait for frame duration
                sleep(Duration::from_millis(frame.duration_ms)).await;
            }
        }
        
        state.set_animation_state(false, None);
        self.is_rendering = false;
        Ok(())
    }
    
    /// Play activity-specific animation
    async fn play_activity_animation(
        &mut self,
        animation_type: AnimationType,
        state: &mut SessionState,
        model_name: &str,
        prefs: &PersonalityPreferences,
    ) -> Result<()> {
        if !prefs.enable_activity_animations {
            return Ok(());
        }
        
        state.set_animation_state(true, Some("activity".to_string()));
        self.is_rendering = true;
        
        // Get activity animation sequence
        if let Some(sequence) = self.get_activity_sequence(&animation_type, state) {
            // Play for a limited time (don't loop forever)
            let max_duration = Duration::from_millis(3000); // 3 seconds max
            let animation_future = async {
                let mut frame_index = 0;
                loop {
                    if !self.is_rendering {
                        break;
                    }
                    
                    let frame = &sequence.frames[frame_index % sequence.frames.len()];
                    
                    // Build statusline with animated personality
                    let animated_statusline = self.build_animated_statusline(
                        state,
                        model_name,
                        prefs,
                        &frame.content,
                    )?;
                    
                    // Render frame
                    self.render_frame_to_terminal(&animated_statusline).await?;
                    
                    // Wait for frame duration
                    sleep(Duration::from_millis(frame.duration_ms)).await;
                    
                    frame_index += 1;
                    
                    // If not looping, break after one cycle
                    if !sequence.loops && frame_index >= sequence.frames.len() {
                        break;
                    }
                }
                Ok::<(), anyhow::Error>(())
            };
            
            // Run animation with timeout
            let _ = timeout(max_duration, animation_future).await;
        }
        
        state.set_animation_state(false, None);
        self.is_rendering = false;
        Ok(())
    }
    
    /// Build statusline with animated personality
    fn build_animated_statusline(
        &self,
        state: &SessionState,
        model_name: &str,
        prefs: &PersonalityPreferences,
        animated_personality: &str,
    ) -> Result<String> {
        // Create temporary state with animated personality
        let mut animated_state = state.clone();
        animated_state.personality = animated_personality.to_string();
        
        Ok(build_statusline(&animated_state, model_name, prefs))
    }
    
    /// Render frame to terminal with proper cursor control
    async fn render_frame_to_terminal(&self, content: &str) -> Result<()> {
        let mut stdout = stdout();
        
        // Save current cursor position
        execute!(stdout, cursor::SavePosition)?;
        
        // Clear current line and render content
        execute!(stdout, Clear(ClearType::CurrentLine))?;
        print!("{}", content);
        stdout.flush()?;
        
        // Restore cursor position
        execute!(stdout, cursor::RestorePosition)?;
        
        Ok(())
    }
    
    /// Get transition animation sequence
    fn get_transition_sequence(
        &self,
        animation_type: &AnimationType,
        state: &SessionState,
    ) -> Option<crate::animation::AnimationSequence> {
        use crate::animation::transitions::PersonalityTransitions;
        
        if let AnimationType::Transition { from, to } = animation_type {
            Some(PersonalityTransitions::smooth_transition(
                from,
                to,
                AnimationSpeed::Normal,
            ))
        } else {
            None
        }
    }
    
    /// Get activity animation sequence
    fn get_activity_sequence(
        &self,
        animation_type: &AnimationType,
        state: &SessionState,
    ) -> Option<crate::animation::AnimationSequence> {
        use crate::animation::activities::ActivityAnimations;
        
        if let AnimationType::Activity { activity } = animation_type {
            // Use contextual animation if we have current job info
            let current_file = state.current_job.as_deref();
            ActivityAnimations::contextual_activity_animation(
                activity,
                current_file,
                AnimationSpeed::Normal,
            )
        } else {
            None
        }
    }
    
    /// Stop any currently running animation
    pub fn stop_animation(&mut self) {
        self.is_rendering = false;
        self.animation_engine.stop_animation();
    }
    
    /// Check if animation is currently running
    pub fn is_animating(&self) -> bool {
        self.is_rendering || self.animation_engine.is_playing()
    }
    
    /// Update animation preferences
    pub fn update_preferences(&mut self, preferences: PersonalityPreferences) {
        self.animation_engine.update_preferences(preferences);
    }
}

impl Default for AnimatedStatuslineRenderer {
    fn default() -> Self {
        Self::new(PersonalityPreferences::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Activity;

    fn create_test_state() -> SessionState {
        SessionState {
            session_id: "test".to_string(),
            activity: Activity::Editing,
            current_job: Some("test.js".to_string()),
            personality: "ʕ•ᴥ•ʔ Code Wizard".to_string(),
            previous_personality: Some("( ˘ ³˘) Booting Up".to_string()),
            consecutive_actions: 1,
            error_count: 0,
            animation_state: Default::default(),
        }
    }

    #[test]
    fn test_animated_statusline_creation() {
        let prefs = PersonalityPreferences::default();
        let renderer = AnimatedStatuslineRenderer::new(prefs);
        
        assert!(!renderer.is_animating());
        assert!(!renderer.is_rendering);
    }

    #[test]
    fn test_build_animated_statusline() {
        let prefs = PersonalityPreferences::default();
        let renderer = AnimatedStatuslineRenderer::new(prefs);
        let state = create_test_state();
        
        let result = renderer.build_animated_statusline(
            &state,
            "Opus",
            &prefs,
            "(⌐■_■) Testing"
        );
        
        assert!(result.is_ok());
        let statusline = result.unwrap();
        assert!(statusline.contains("(⌐■_■) Testing"));
    }

    #[tokio::test]
    async fn test_render_without_animation() {
        let prefs = PersonalityPreferences::default();
        let mut renderer = AnimatedStatuslineRenderer::new(prefs);
        let mut state = create_test_state();
        
        // Clear transition flag
        state.consume_transition();
        
        let result = renderer.render_animated_statusline(
            &mut state,
            "Opus",
            &PersonalityPreferences::default()
        ).await;
        
        assert!(result.is_ok());
        let statusline = result.unwrap();
        assert!(statusline.contains("ʕ•ᴥ•ʔ Code Wizard"));
    }

    #[test]
    fn test_stop_animation() {
        let prefs = PersonalityPreferences::default();
        let mut renderer = AnimatedStatuslineRenderer::new(prefs);
        
        // Simulate running animation
        renderer.is_rendering = true;
        
        renderer.stop_animation();
        
        assert!(!renderer.is_animating());
        assert!(!renderer.is_rendering);
    }

    #[test]
    fn test_get_transition_sequence() {
        let prefs = PersonalityPreferences::default();
        let renderer = AnimatedStatuslineRenderer::new(prefs);
        let state = create_test_state();
        
        let transition = AnimationType::Transition {
            from: "ʕ•ᴥ•ʔ Code Wizard".to_string(),
            to: "(˘ ³˘) Booting Up".to_string(),
        };
        
        let sequence = renderer.get_transition_sequence(&transition, &state);
        assert!(sequence.is_some());
        
        let seq = sequence.unwrap();
        assert!(!seq.frames.is_empty());
    }

    #[test]
    fn test_get_activity_sequence() {
        let prefs = PersonalityPreferences::default();
        let renderer = AnimatedStatuslineRenderer::new(prefs);
        let state = create_test_state();
        
        let activity = AnimationType::Activity {
            activity: Activity::Editing,
        };
        
        let sequence = renderer.get_activity_sequence(&activity, &state);
        assert!(sequence.is_some());
        
        let seq = sequence.unwrap();
        assert!(!seq.frames.is_empty());
        assert!(seq.loops); // Activity animations should loop
    }

    #[test]
    fn test_update_preferences() {
        let prefs = PersonalityPreferences::default();
        let mut renderer = AnimatedStatuslineRenderer::new(prefs);
        
        let mut new_prefs = PersonalityPreferences::default();
        new_prefs.enable_animations = false;
        
        renderer.update_preferences(new_prefs);
        
        // Can't directly test internal state, but method should not panic
        assert!(!renderer.is_animating());
    }
}