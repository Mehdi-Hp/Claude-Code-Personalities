use std::time::{Duration, Instant};
use tokio::time::sleep;
use crossterm::{
    cursor,
    execute,
    terminal::{Clear, ClearType},
};
use std::io::{stdout, Write};

use super::{AnimationFrame, AnimationSequence, AnimationType};
use crate::config::preferences::PersonalityPreferences;
use crate::state::SessionState;

/// Animation engine that handles frame rendering and timing
pub struct AnimationEngine {
    /// Current animation being played
    current_animation: Option<AnimationSequence>,
    /// Start time of current animation
    start_time: Option<Instant>,
    /// Current frame index
    current_frame: usize,
    /// Whether animation is currently playing
    is_playing: bool,
    /// Preferences for animation settings
    preferences: PersonalityPreferences,
}

impl AnimationEngine {
    pub fn new(preferences: PersonalityPreferences) -> Self {
        Self {
            current_animation: None,
            start_time: None,
            current_frame: 0,
            is_playing: false,
            preferences,
        }
    }
    
    /// Start playing an animation
    pub async fn play_animation(&mut self, animation_type: AnimationType, state: &SessionState) {
        // Check if animations are enabled
        if !self.preferences.enable_animations {
            return;
        }
        
        // Get appropriate animation sequence
        let sequence = match self.get_animation_sequence(&animation_type, state) {
            Some(seq) => seq,
            None => return, // No animation available
        };
        
        self.current_animation = Some(sequence);
        self.start_time = Some(Instant::now());
        self.current_frame = 0;
        self.is_playing = true;
        
        // Start animation loop
        self.animation_loop().await;
    }
    
    /// Stop current animation
    pub fn stop_animation(&mut self) {
        self.is_playing = false;
        self.current_animation = None;
        self.start_time = None;
        self.current_frame = 0;
    }
    
    /// Check if an animation is currently playing
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }
    
    /// Render a single frame with optional animation
    pub async fn render_frame(&self, content: &str, with_animation: bool) -> Result<String, Box<dyn std::error::Error>> {
        if !with_animation || !self.preferences.enable_animations {
            return Ok(content.to_string());
        }
        
        if let Some(animation) = &self.current_animation {
            if let Some(frame) = animation.frames.get(self.current_frame) {
                // Apply any frame transformations (like offset)
                return Ok(self.apply_frame_effects(&frame.content, frame));
            }
        }
        
        Ok(content.to_string())
    }
    
    /// Main animation loop
    async fn animation_loop(&mut self) {
        let animation = match &self.current_animation {
            Some(anim) => anim.clone(),
            None => return,
        };
        
        loop {
            if !self.is_playing {
                break;
            }
            
            // Get current frame
            let frame = match animation.frames.get(self.current_frame) {
                Some(frame) => frame,
                None => {
                    if animation.loops {
                        self.current_frame = 0;
                        continue;
                    } else {
                        break; // Animation finished
                    }
                }
            };
            
            // Render the frame
            if let Err(_) = self.render_animation_frame(frame).await {
                break; // Error rendering, stop animation
            }
            
            // Wait for frame duration
            sleep(Duration::from_millis(frame.duration_ms)).await;
            
            // Advance to next frame
            self.current_frame += 1;
        }
        
        self.stop_animation();
    }
    
    /// Render a single animation frame to the terminal
    async fn render_animation_frame(&self, frame: &AnimationFrame) -> Result<(), Box<dyn std::error::Error>> {
        let mut stdout = stdout();
        
        // Clear current line
        execute!(stdout, Clear(ClearType::CurrentLine))?;
        
        // Apply cursor offset if specified
        if frame.offset_x != 0 {
            if frame.offset_x > 0 {
                execute!(stdout, cursor::MoveRight(frame.offset_x as u16))?;
            } else {
                execute!(stdout, cursor::MoveLeft((-frame.offset_x) as u16))?;
            }
        }
        
        if frame.offset_y != 0 {
            if frame.offset_y > 0 {
                execute!(stdout, cursor::MoveDown(frame.offset_y as u16))?;
            } else {
                execute!(stdout, cursor::MoveUp((-frame.offset_y) as u16))?;
            }
        }
        
        // Print frame content
        print!("{}", frame.content);
        stdout.flush()?;
        
        // Reset cursor position if we moved it
        if frame.offset_x != 0 || frame.offset_y != 0 {
            if frame.offset_x != 0 {
                if frame.offset_x > 0 {
                    execute!(stdout, cursor::MoveLeft(frame.offset_x as u16))?;
                } else {
                    execute!(stdout, cursor::MoveRight((-frame.offset_x) as u16))?;
                }
            }
            
            if frame.offset_y != 0 {
                if frame.offset_y > 0 {
                    execute!(stdout, cursor::MoveUp(frame.offset_y as u16))?;
                } else {
                    execute!(stdout, cursor::MoveDown((-frame.offset_y) as u16))?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Apply visual effects to frame content
    fn apply_frame_effects(&self, content: &str, frame: &AnimationFrame) -> String {
        let mut result = content.to_string();
        
        // Apply offset effects by padding with spaces
        if frame.offset_x > 0 {
            result = format!("{}{}", " ".repeat(frame.offset_x as usize), result);
        }
        
        result
    }
    
    /// Get appropriate animation sequence based on type and state
    fn get_animation_sequence(&self, animation_type: &AnimationType, state: &SessionState) -> Option<AnimationSequence> {
        use super::AnimationFrames;
        
        match animation_type {
            AnimationType::Transition { from, to } => {
                if self.preferences.enable_transitions {
                    Some(AnimationFrames::personality_transition(from, to, self.preferences.animation_speed))
                } else {
                    None
                }
            },
            AnimationType::Activity { activity } => {
                if self.preferences.enable_activity_animations {
                    AnimationFrames::for_activity(activity, self.preferences.animation_speed)
                } else {
                    None
                }
            },
            AnimationType::Error { level } => {
                // Error animations are always enabled for important feedback
                Some(AnimationFrames::error_animation(*level, &state.personality, self.preferences.animation_speed))
            },
            AnimationType::Celebration => {
                Some(AnimationFrames::celebration_sparkles(self.preferences.animation_speed))
            },
            AnimationType::Idle => {
                // Create a gentle idle animation
                Some(AnimationFrames::thinking_dots(self.preferences.animation_speed))
            },
        }
    }
    
    /// Update preferences (for runtime configuration changes)
    pub fn update_preferences(&mut self, preferences: PersonalityPreferences) {
        self.preferences = preferences;
    }
    
    /// Create a simple static frame for non-animated display
    pub fn create_static_frame(content: &str) -> String {
        content.to_string()
    }
    
    /// Check if a specific animation type should be played based on preferences
    pub fn should_animate(&self, animation_type: &AnimationType) -> bool {
        if !self.preferences.enable_animations {
            return false;
        }
        
        match animation_type {
            AnimationType::Transition { .. } => self.preferences.enable_transitions,
            AnimationType::Activity { .. } => self.preferences.enable_activity_animations,
            AnimationType::Error { .. } => true, // Always show error animations
            AnimationType::Celebration | AnimationType::Idle => true,
        }
    }
}

impl Default for AnimationEngine {
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
            activity: Activity::Idle,
            current_job: None,
            personality: "ʕ•ᴥ•ʔ Code Wizard".to_string(),
            consecutive_actions: 0,
            error_count: 0,
        }
    }

    #[test]
    fn test_animation_engine_creation() {
        let prefs = PersonalityPreferences::default();
        let engine = AnimationEngine::new(prefs.clone());
        
        assert!(!engine.is_playing());
        assert!(engine.current_animation.is_none());
        assert_eq!(engine.preferences.enable_animations, prefs.enable_animations);
    }
    
    #[test]
    fn test_should_animate() {
        let mut prefs = PersonalityPreferences::default();
        prefs.enable_animations = true;
        prefs.enable_transitions = true;
        prefs.enable_activity_animations = false;
        
        let engine = AnimationEngine::new(prefs);
        
        // Should animate transitions
        let transition = AnimationType::Transition { 
            from: "ʕ•ᴥ•ʔ".to_string(), 
            to: "(˘ ³˘)".to_string() 
        };
        assert!(engine.should_animate(&transition));
        
        // Should not animate activities (disabled)
        let activity = AnimationType::Activity { activity: Activity::Editing };
        assert!(!engine.should_animate(&activity));
        
        // Should always animate errors
        let error = AnimationType::Error { level: 3 };
        assert!(engine.should_animate(&error));
        
        // Should animate celebration and idle
        assert!(engine.should_animate(&AnimationType::Celebration));
        assert!(engine.should_animate(&AnimationType::Idle));
    }
    
    #[test]
    fn test_should_not_animate_when_disabled() {
        let mut prefs = PersonalityPreferences::default();
        prefs.enable_animations = false;
        
        let engine = AnimationEngine::new(prefs);
        
        // Should not animate anything when animations are disabled
        let transition = AnimationType::Transition { 
            from: "ʕ•ᴥ•ʔ".to_string(), 
            to: "(˘ ³˘)".to_string() 
        };
        assert!(!engine.should_animate(&transition));
        
        let activity = AnimationType::Activity { activity: Activity::Editing };
        assert!(!engine.should_animate(&activity));
        
        // Except errors should still show (for important feedback)
        let error = AnimationType::Error { level: 3 };
        assert!(!engine.should_animate(&error));
    }
    
    #[tokio::test]
    async fn test_render_frame_without_animation() {
        let prefs = PersonalityPreferences::default();
        let engine = AnimationEngine::new(prefs);
        
        let result = engine.render_frame("ʕ•ᴥ•ʔ Code Wizard", false).await.unwrap();
        assert_eq!(result, "ʕ•ᴥ•ʔ Code Wizard");
    }
    
    #[test]
    fn test_stop_animation() {
        let prefs = PersonalityPreferences::default();
        let mut engine = AnimationEngine::new(prefs);
        
        // Simulate animation state
        engine.is_playing = true;
        engine.current_frame = 5;
        
        engine.stop_animation();
        
        assert!(!engine.is_playing());
        assert!(engine.current_animation.is_none());
        assert_eq!(engine.current_frame, 0);
    }
    
    #[test]
    fn test_apply_frame_effects() {
        let prefs = PersonalityPreferences::default();
        let engine = AnimationEngine::new(prefs);
        
        let frame = AnimationFrame::new("test", 100).with_offset(3, 0);
        let result = engine.apply_frame_effects("content", &frame);
        
        assert_eq!(result, "   content");
    }
    
    #[test]
    fn test_update_preferences() {
        let prefs = PersonalityPreferences::default();
        let mut engine = AnimationEngine::new(prefs);
        
        let mut new_prefs = PersonalityPreferences::default();
        new_prefs.enable_animations = false;
        
        engine.update_preferences(new_prefs.clone());
        
        assert_eq!(engine.preferences.enable_animations, new_prefs.enable_animations);
    }
    
    #[test]
    fn test_create_static_frame() {
        let result = AnimationEngine::create_static_frame("Static content");
        assert_eq!(result, "Static content");
    }
    
    #[test]
    fn test_get_animation_sequence() {
        let prefs = PersonalityPreferences::default();
        let engine = AnimationEngine::new(prefs);
        let state = create_test_state();
        
        // Test transition animation
        let transition = AnimationType::Transition { 
            from: "ʕ•ᴥ•ʔ Code Wizard".to_string(), 
            to: "(˘ ³˘) Booting Up".to_string() 
        };
        let seq = engine.get_animation_sequence(&transition, &state);
        assert!(seq.is_some());
        assert!(seq.unwrap().frames.len() > 0);
        
        // Test activity animation
        let activity = AnimationType::Activity { activity: Activity::Building };
        let seq = engine.get_animation_sequence(&activity, &state);
        assert!(seq.is_some());
        
        // Test error animation
        let error = AnimationType::Error { level: 3 };
        let seq = engine.get_animation_sequence(&error, &state);
        assert!(seq.is_some());
        
        // Test celebration animation
        let seq = engine.get_animation_sequence(&AnimationType::Celebration, &state);
        assert!(seq.is_some());
        
        // Test idle animation
        let seq = engine.get_animation_sequence(&AnimationType::Idle, &state);
        assert!(seq.is_some());
    }
}