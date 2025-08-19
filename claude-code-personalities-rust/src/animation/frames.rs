use super::{AnimationFrame, AnimationSequence, AnimationSpeed};
use crate::types::Activity;

/// Pre-defined animation frame sequences for different personality types and activities
pub struct AnimationFrames;

impl AnimationFrames {
    /// Get spinner frames for building/processing activities
    pub fn spinner_frames(speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms();
        AnimationSequence::from_text_frames(
            vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            duration
        ).with_loops()
    }
    
    /// Get progress bar frames for testing activities
    pub fn progress_frames(speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms() * 2; // Slower for progress
        AnimationSequence::from_text_frames(
            vec![
                "[▱▱▱▱▱]",
                "[▰▱▱▱▱]", 
                "[▰▰▱▱▱]",
                "[▰▰▰▱▱]",
                "[▰▰▰▰▱]",
                "[▰▰▰▰▰]"
            ],
            duration
        )
    }
    
    /// Get pulsing dots for thinking activities
    pub fn thinking_dots(speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms() * 3; // Much slower for thinking
        AnimationSequence::from_text_frames(
            vec!["", ".", "..", "...", "..", "."],
            duration
        ).with_loops()
    }
    
    /// Get search scanning animation
    pub fn search_scan(speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms() * 2;
        let frames = vec![
            AnimationFrame::new("(◔_◔)", duration),
            AnimationFrame::new("(◔_◔)", duration),
            AnimationFrame::new("(◕_◔)", duration),
            AnimationFrame::new("(◔_◕)", duration),
            AnimationFrame::new("(◔_◔)", duration),
        ];
        AnimationSequence::new(frames).with_loops()
    }
    
    /// Get debugging magnifying glass effect
    pub fn debug_magnify(speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms() * 4; // Slower for debug
        let frames = vec![
            AnimationFrame::new("( ಠ_ಠ)", duration),
            AnimationFrame::new("( ಠ_ರೃ)", duration),
            AnimationFrame::new("( ಠ_ಠ)", duration),
            AnimationFrame::new("( ರೃ_ಠ)", duration),
        ];
        AnimationSequence::new(frames).with_loops()
    }
    
    /// Get shake effect for error states
    pub fn shake_effect(personality: &str, speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms() / 2; // Fast shaking
        let frames = vec![
            AnimationFrame::new(personality, duration).with_offset(-1, 0),
            AnimationFrame::new(personality, duration).with_offset(1, 0),
            AnimationFrame::new(personality, duration).with_offset(0, 0),
            AnimationFrame::new(personality, duration).with_offset(1, 0),
            AnimationFrame::new(personality, duration).with_offset(-1, 0),
            AnimationFrame::new(personality, duration).with_offset(0, 0),
        ];
        AnimationSequence::new(frames)
    }
    
    /// Get celebration sparkle effect
    pub fn celebration_sparkles(speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms();
        let frames = vec![
            AnimationFrame::new("✨(ﾉ◕ヮ◕)ﾉ*:･ﾟ✧", duration),
            AnimationFrame::new("⭐(ﾉ◕ヮ◕)ﾉ*:･ﾟ⭐", duration),
            AnimationFrame::new("✨(ﾉ◕ヮ◕)ﾉ*:･ﾟ✨", duration),
            AnimationFrame::new("⭐(ﾉ◕ヮ◕)ﾉ*:･ﾟ⭐", duration),
            AnimationFrame::new("✨(ﾉ◕ヮ◕)ﾉ*:･ﾟ✨", duration),
        ];
        AnimationSequence::new(frames)
    }
    
    /// Get activity-specific animation sequence
    pub fn for_activity(activity: &Activity, speed: AnimationSpeed) -> Option<AnimationSequence> {
        match activity {
            Activity::Building | Activity::Installing => Some(Self::spinner_frames(speed)),
            Activity::Testing => Some(Self::progress_frames(speed)),
            Activity::Thinking => Some(Self::thinking_dots(speed)),
            Activity::Searching => Some(Self::search_scan(speed)),
            Activity::Debugging => Some(Self::debug_magnify(speed)),
            _ => None, // No specific animation for other activities
        }
    }
    
    /// Get personality transition frames (morphing effect)
    pub fn personality_transition(from: &str, to: &str, speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.transition_duration_ms() / 5; // 5 frames for transition
        
        // Extract the face part from personality strings
        let from_face = Self::extract_face(from);
        let to_face = Self::extract_face(to);
        
        // Create morphing frames
        let morph_frames = Self::create_morph_sequence(&from_face, &to_face);
        
        let frames: Vec<AnimationFrame> = morph_frames.into_iter()
            .map(|face| AnimationFrame::new(face, duration))
            .collect();
            
        AnimationSequence::new(frames)
    }
    
    /// Extract the face part from a personality string
    fn extract_face(personality: &str) -> String {
        // Simple extraction - take everything before the first space (the face part)
        personality.split_whitespace()
            .next()
            .unwrap_or(personality)
            .to_string()
    }
    
    /// Create morphing sequence between two faces
    fn create_morph_sequence(from: &str, to: &str) -> Vec<String> {
        // For now, create a simple intermediate transition
        // In the future, this could be more sophisticated with character-by-character morphing
        vec![
            from.to_string(),
            Self::create_intermediate_face(from, to, 0.25),
            Self::create_intermediate_face(from, to, 0.5),
            Self::create_intermediate_face(from, to, 0.75),
            to.to_string(),
        ]
    }
    
    /// Create intermediate face for morphing (simple implementation)
    fn create_intermediate_face(from: &str, _to: &str, progress: f32) -> String {
        // Simple implementation: gradually close eyes during transition
        if progress > 0.2 && progress < 0.8 {
            // Replace eyes with closed/transitional states
            from.chars()
                .map(|c| match c {
                    '•' | '◕' | '○' | '°' | 'ಠ' | '◔' | '◉' => if progress < 0.5 { '-' } else { '˘' },
                    _ => c,
                })
                .collect()
        } else {
            from.to_string()
        }
    }
    
    /// Get error level animation
    pub fn error_animation(level: u32, personality: &str, speed: AnimationSpeed) -> AnimationSequence {
        match level {
            0 => {
                // No errors - calm state
                let frames = vec![AnimationFrame::new(personality, speed.frame_duration_ms() * 10)];
                AnimationSequence::new(frames)
            },
            1..=2 => {
                // Light frustration - slight movement
                let duration = speed.frame_duration_ms() * 3;
                let frames = vec![
                    AnimationFrame::new(personality, duration),
                    AnimationFrame::new(personality, duration).with_offset(0, 0),
                ];
                AnimationSequence::new(frames).with_loops()
            },
            3..=4 => {
                // Medium frustration - gentle shake
                Self::shake_effect(personality, speed)
            },
            _ => {
                // High frustration - intense shake
                let duration = speed.frame_duration_ms() / 3; // Very fast shaking
                let frames = vec![
                    AnimationFrame::new(personality, duration).with_offset(-2, 0),
                    AnimationFrame::new(personality, duration).with_offset(2, 0),
                    AnimationFrame::new(personality, duration).with_offset(0, -1),
                    AnimationFrame::new(personality, duration).with_offset(0, 1),
                    AnimationFrame::new(personality, duration).with_offset(-1, -1),
                    AnimationFrame::new(personality, duration).with_offset(1, 1),
                    AnimationFrame::new(personality, duration).with_offset(0, 0),
                ];
                AnimationSequence::new(frames).with_loops()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_frames() {
        let sequence = AnimationFrames::spinner_frames(AnimationSpeed::Normal);
        assert_eq!(sequence.frames.len(), 10);
        assert!(sequence.loops);
        assert_eq!(sequence.frames[0].content, "⠋");
        assert_eq!(sequence.frames[9].content, "⠏");
    }
    
    #[test]
    fn test_progress_frames() {
        let sequence = AnimationFrames::progress_frames(AnimationSpeed::Fast);
        assert_eq!(sequence.frames.len(), 6);
        assert!(!sequence.loops);
        assert_eq!(sequence.frames[0].content, "[▱▱▱▱▱]");
        assert_eq!(sequence.frames[5].content, "[▰▰▰▰▰]");
    }
    
    #[test]
    fn test_thinking_dots() {
        let sequence = AnimationFrames::thinking_dots(AnimationSpeed::Normal);
        assert_eq!(sequence.frames.len(), 6);
        assert!(sequence.loops);
        assert_eq!(sequence.frames[0].content, "");
        assert_eq!(sequence.frames[3].content, "...");
    }
    
    #[test]
    fn test_extract_face() {
        assert_eq!(
            AnimationFrames::extract_face("ʕ•ᴥ•ʔ Code Wizard"),
            "ʕ•ᴥ•ʔ"
        );
        assert_eq!(
            AnimationFrames::extract_face("(╯°□°)╯︵ ┻━┻ Table Flipper"),
            "(╯°□°)╯︵"
        );
        assert_eq!(
            AnimationFrames::extract_face("ಠ_ಠ"),
            "ಠ_ಠ"
        );
    }
    
    #[test]
    fn test_create_morph_sequence() {
        let sequence = AnimationFrames::create_morph_sequence("ʕ•ᴥ•ʔ", "(˘ ³˘)");
        assert_eq!(sequence.len(), 5);
        assert_eq!(sequence[0], "ʕ•ᴥ•ʔ");
        assert_eq!(sequence[4], "(˘ ³˘)");
        // Middle frames should have transitional characters
        assert_ne!(sequence[2], sequence[0]);
        assert_ne!(sequence[2], sequence[4]);
    }
    
    #[test]
    fn test_shake_effect() {
        let sequence = AnimationFrames::shake_effect("(╯°□°)╯", AnimationSpeed::Fast);
        assert_eq!(sequence.frames.len(), 6);
        assert!(!sequence.loops);
        
        // Check that frames have different offsets
        assert_eq!(sequence.frames[0].offset_x, -1);
        assert_eq!(sequence.frames[1].offset_x, 1);
        assert_eq!(sequence.frames[2].offset_x, 0);
    }
    
    #[test]
    fn test_for_activity() {
        assert!(AnimationFrames::for_activity(&Activity::Building, AnimationSpeed::Normal).is_some());
        assert!(AnimationFrames::for_activity(&Activity::Testing, AnimationSpeed::Normal).is_some());
        assert!(AnimationFrames::for_activity(&Activity::Thinking, AnimationSpeed::Normal).is_some());
        assert!(AnimationFrames::for_activity(&Activity::Searching, AnimationSpeed::Normal).is_some());
        assert!(AnimationFrames::for_activity(&Activity::Debugging, AnimationSpeed::Normal).is_some());
        
        // Activities without specific animations
        assert!(AnimationFrames::for_activity(&Activity::Reading, AnimationSpeed::Normal).is_none());
        assert!(AnimationFrames::for_activity(&Activity::Writing, AnimationSpeed::Normal).is_none());
        assert!(AnimationFrames::for_activity(&Activity::Idle, AnimationSpeed::Normal).is_none());
    }
    
    #[test]
    fn test_error_animation() {
        // No errors - should be calm
        let seq0 = AnimationFrames::error_animation(0, "(˘ ³˘)", AnimationSpeed::Normal);
        assert_eq!(seq0.frames.len(), 1);
        assert!(!seq0.loops);
        
        // Low errors - should have some movement
        let seq1 = AnimationFrames::error_animation(1, "(¬_¬)", AnimationSpeed::Normal);
        assert!(seq1.loops);
        
        // Medium errors - should shake
        let seq3 = AnimationFrames::error_animation(3, "(ಠ_ಠ)", AnimationSpeed::Normal);
        assert!(seq3.frames.iter().any(|f| f.offset_x != 0 || f.offset_y != 0));
        
        // High errors - should shake intensely
        let seq5 = AnimationFrames::error_animation(5, "(╯°□°)╯", AnimationSpeed::Normal);
        assert!(seq5.loops);
        assert!(seq5.frames.iter().any(|f| f.offset_x.abs() >= 2 || f.offset_y.abs() >= 1));
    }
}