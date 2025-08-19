pub mod engine;
pub mod transitions;
pub mod activities;
pub mod frames;

pub use engine::*;
pub use transitions::*;
pub use activities::*;
pub use frames::*;

use crate::types::Activity;
use serde::{Deserialize, Serialize};

/// Types of animations that can be played
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationType {
    /// Personality transition from one to another
    Transition { from: String, to: String },
    /// Activity-specific animation
    Activity { activity: Activity },
    /// Error state animation based on error level
    Error { level: u32 },
    /// Celebration animation for successful operations
    Celebration,
    /// Idle state animation
    Idle,
}

/// Animation speed settings
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AnimationSpeed {
    Slow,
    Normal,
    Fast,
}

impl Default for AnimationSpeed {
    fn default() -> Self {
        AnimationSpeed::Normal
    }
}

impl AnimationSpeed {
    /// Get frame duration in milliseconds
    pub fn frame_duration_ms(&self) -> u64 {
        match self {
            AnimationSpeed::Slow => 150,
            AnimationSpeed::Normal => 100,
            AnimationSpeed::Fast => 50,
        }
    }
    
    /// Get transition duration in milliseconds
    pub fn transition_duration_ms(&self) -> u64 {
        match self {
            AnimationSpeed::Slow => 400,
            AnimationSpeed::Normal => 250,
            AnimationSpeed::Fast => 150,
        }
    }
}

/// Single frame of animation
#[derive(Debug, Clone)]
pub struct AnimationFrame {
    /// The content to display
    pub content: String,
    /// Duration to display this frame in milliseconds
    pub duration_ms: u64,
    /// Optional cursor offset (for shake effects)
    pub offset_x: i16,
    pub offset_y: i16,
}

impl AnimationFrame {
    pub fn new(content: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            content: content.into(),
            duration_ms,
            offset_x: 0,
            offset_y: 0,
        }
    }
    
    pub fn with_offset(mut self, x: i16, y: i16) -> Self {
        self.offset_x = x;
        self.offset_y = y;
        self
    }
}

/// Complete animation sequence
#[derive(Debug, Clone)]
pub struct AnimationSequence {
    /// Animation frames in order
    pub frames: Vec<AnimationFrame>,
    /// Whether the animation should loop
    pub loops: bool,
    /// Total duration of one complete sequence
    pub total_duration_ms: u64,
}

impl AnimationSequence {
    pub fn new(frames: Vec<AnimationFrame>) -> Self {
        let total_duration_ms = frames.iter().map(|f| f.duration_ms).sum();
        Self {
            frames,
            loops: false,
            total_duration_ms,
        }
    }
    
    pub fn with_loops(mut self) -> Self {
        self.loops = true;
        self
    }
    
    /// Create a simple sequence from text with equal frame durations
    pub fn from_text_frames(frames: Vec<&str>, frame_duration: u64) -> Self {
        let frames = frames.into_iter()
            .map(|text| AnimationFrame::new(text, frame_duration))
            .collect();
        Self::new(frames)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_speed() {
        assert_eq!(AnimationSpeed::Normal.frame_duration_ms(), 100);
        assert_eq!(AnimationSpeed::Slow.frame_duration_ms(), 150);
        assert_eq!(AnimationSpeed::Fast.frame_duration_ms(), 50);
        
        assert_eq!(AnimationSpeed::Normal.transition_duration_ms(), 250);
        assert_eq!(AnimationSpeed::Slow.transition_duration_ms(), 400);
        assert_eq!(AnimationSpeed::Fast.transition_duration_ms(), 150);
    }
    
    #[test]
    fn test_animation_frame() {
        let frame = AnimationFrame::new("ʕ•ᴥ•ʔ", 100)
            .with_offset(1, 0);
        
        assert_eq!(frame.content, "ʕ•ᴥ•ʔ");
        assert_eq!(frame.duration_ms, 100);
        assert_eq!(frame.offset_x, 1);
        assert_eq!(frame.offset_y, 0);
    }
    
    #[test]
    fn test_animation_sequence() {
        let sequence = AnimationSequence::from_text_frames(
            vec!["ʕ•ᴥ•ʔ", "ʕ-ᴥ-ʔ", "ʕ˘ᴥ˘ʔ"], 
            100
        ).with_loops();
        
        assert_eq!(sequence.frames.len(), 3);
        assert_eq!(sequence.total_duration_ms, 300);
        assert!(sequence.loops);
        assert_eq!(sequence.frames[0].content, "ʕ•ᴥ•ʔ");
        assert_eq!(sequence.frames[1].content, "ʕ-ᴥ-ʔ");
        assert_eq!(sequence.frames[2].content, "ʕ˘ᴥ˘ʔ");
    }
    
    #[test]
    fn test_animation_type_equality() {
        let transition1 = AnimationType::Transition { 
            from: "ʕ•ᴥ•ʔ Code Wizard".to_string(), 
            to: "(˘ ³˘) Booting Up".to_string() 
        };
        let transition2 = AnimationType::Transition { 
            from: "ʕ•ᴥ•ʔ Code Wizard".to_string(), 
            to: "(˘ ³˘) Booting Up".to_string() 
        };
        
        assert_eq!(transition1, transition2);
        
        let activity = AnimationType::Activity { activity: Activity::Editing };
        let error = AnimationType::Error { level: 3 };
        
        assert_ne!(transition1, activity);
        assert_ne!(activity, error);
    }
}