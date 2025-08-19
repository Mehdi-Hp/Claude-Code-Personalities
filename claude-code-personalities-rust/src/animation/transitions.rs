use super::{AnimationFrame, AnimationSequence, AnimationSpeed};

/// Personality transition animations and effects
pub struct PersonalityTransitions;

impl PersonalityTransitions {
    /// Create a smooth transition between two personalities
    pub fn smooth_transition(from: &str, to: &str, speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.transition_duration_ms() / 8; // 8 frames for smooth transition
        
        let from_face = Self::extract_face(from);
        let to_face = Self::extract_face(to);
        
        // Create smooth morphing frames
        let frames = Self::create_smooth_morph(&from_face, &to_face, duration);
        
        AnimationSequence::new(frames)
    }
    
    /// Create a fade transition effect
    pub fn fade_transition(from: &str, to: &str, speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms() * 2;
        
        let frames = vec![
            AnimationFrame::new(from, duration),
            AnimationFrame::new("", duration), // Blank frame for fade effect
            AnimationFrame::new(to, duration),
        ];
        
        AnimationSequence::new(frames)
    }
    
    /// Create a slide transition (personality slides in from the side)
    pub fn slide_transition(from: &str, to: &str, speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms();
        let to_face = Self::extract_face(to);
        
        let frames = vec![
            // Slide out old personality
            AnimationFrame::new(from, duration),
            AnimationFrame::new(from, duration).with_offset(1, 0),
            AnimationFrame::new(from, duration).with_offset(2, 0),
            AnimationFrame::new("", duration), // Clear
            // Slide in new personality
            AnimationFrame::new(&to_face, duration).with_offset(-2, 0),
            AnimationFrame::new(&to_face, duration).with_offset(-1, 0),
            AnimationFrame::new(to, duration),
        ];
        
        AnimationSequence::new(frames)
    }
    
    /// Create a bounce transition effect
    pub fn bounce_transition(from: &str, to: &str, speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms();
        
        let frames = vec![
            AnimationFrame::new(from, duration),
            AnimationFrame::new(from, duration).with_offset(0, -1), // Up
            AnimationFrame::new("", duration), // Blank
            AnimationFrame::new(to, duration).with_offset(0, 1), // Down
            AnimationFrame::new(to, duration),
        ];
        
        AnimationSequence::new(frames)
    }
    
    /// Create a spin transition effect
    pub fn spin_transition(from: &str, to: &str, speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms();
        let from_face = Self::extract_face(from);
        let to_face = Self::extract_face(to);
        
        // Create spinning effect with character rotation
        let spin_frames = vec![
            from_face.clone(),
            Self::rotate_face(&from_face, 90),
            Self::rotate_face(&from_face, 180),
            Self::rotate_face(&from_face, 270),
            to_face.clone(),
        ];
        
        let frames: Vec<AnimationFrame> = spin_frames.into_iter()
            .map(|face| AnimationFrame::new(face, duration))
            .collect();
            
        AnimationSequence::new(frames)
    }
    
    /// Extract personality face from full personality string
    fn extract_face(personality: &str) -> String {
        personality.split_whitespace()
            .next()
            .unwrap_or(personality)
            .to_string()
    }
    
    /// Create smooth morphing frames between two faces
    fn create_smooth_morph(from: &str, to: &str, frame_duration: u64) -> Vec<AnimationFrame> {
        let mut frames = Vec::new();
        
        // Start with original
        frames.push(AnimationFrame::new(from, frame_duration));
        
        // Create intermediate morphing states
        for i in 1..=6 {
            let progress = i as f32 / 7.0;
            let morphed = Self::morph_characters(from, to, progress);
            frames.push(AnimationFrame::new(morphed, frame_duration));
        }
        
        // End with target
        frames.push(AnimationFrame::new(to, frame_duration));
        
        frames
    }
    
    /// Advanced character morphing between two face strings
    fn morph_characters(from: &str, to: &str, progress: f32) -> String {
        let from_chars: Vec<char> = from.chars().collect();
        let to_chars: Vec<char> = to.chars().collect();
        let max_len = from_chars.len().max(to_chars.len());
        
        let mut result = String::new();
        
        for i in 0..max_len {
            let from_char = from_chars.get(i).copied().unwrap_or(' ');
            let to_char = to_chars.get(i).copied().unwrap_or(' ');
            
            let morphed_char = Self::morph_single_character(from_char, to_char, progress);
            result.push(morphed_char);
        }
        
        result
    }
    
    /// Morph a single character based on progress (0.0 to 1.0)
    fn morph_single_character(from: char, to: char, progress: f32) -> char {
        if from == to {
            return from;
        }
        
        // Define morphing stages for eyes and mouth characters as arrays
        let eye_morph_stages = [
            ['•', '°', '◯', '○', '◌'],
            ['○', '◯', '°', '•', '·'],
            ['◕', '◔', '◑', '◐', '●'],
            ['ಠ', 'ರ', '೧', 'ಥ', '◔'],
        ];
        
        let mouth_morph_stages = [
            ['ᴥ', '﹃', '︶', '‿', '◡'],
            ['ω', '㉨', 'ᴗ', '◇', '◆'],
            ['³', '˘', 'ε', '□', '益'],
        ];
        
        // Check if this is an eye character
        for stage in &eye_morph_stages {
            if stage.contains(&from) && stage.contains(&to) {
                let stage_index = (progress * (stage.len() - 1) as f32).round() as usize;
                return stage[stage_index.min(stage.len() - 1)];
            }
        }
        
        // Check if this is a mouth character  
        for stage in &mouth_morph_stages {
            if stage.contains(&from) && stage.contains(&to) {
                let stage_index = (progress * (stage.len() - 1) as f32).round() as usize;
                return stage[stage_index.min(stage.len() - 1)];
            }
        }
        
        // For other characters, use simple threshold-based switching
        if progress < 0.3 {
            from
        } else if progress < 0.7 {
            Self::get_intermediate_character(from, to)
        } else {
            to
        }
    }
    
    /// Get an intermediate character for morphing
    fn get_intermediate_character(from: char, to: char) -> char {
        match (from, to) {
            // Parentheses transitions
            ('(', '[') | ('[', '(') => '⌐',
            (')', ']') | (']', ')') => '¬',
            
            // Bracket transitions
            ('⌐', '╭') | ('╭', '⌐') => '┌',
            ('¬', '╮') | ('╮', '¬') => '┐',
            
            // Eye transitions
            ('•', '°') | ('°', '•') => '◦',
            ('○', '●') | ('●', '○') => '◯',
            
            // Special character transitions
            ('ಠ', '◕') | ('◕', 'ಠ') => '◔',
            ('ᴥ', '‿') | ('‿', 'ᴥ') => '◡',
            
            // Default: alternate between characters
            _ => if rand::random::<bool>() { from } else { to },
        }
    }
    
    /// Simulate character rotation for spin effect
    fn rotate_face(face: &str, _degrees: u16) -> String {
        // Simple implementation: replace characters with "rotated" equivalents
        face.chars().map(|c| match c {
            '(' => '⌐',
            ')' => '¬', 
            '[' => '╭',
            ']' => '╮',
            '•' => '°',
            '°' => '◯',
            '○' => '●',
            '●' => '○',
            _ => c,
        }).collect()
    }
    
    /// Get transition based on personality change context
    pub fn contextual_transition(from: &str, to: &str, speed: AnimationSpeed, context: TransitionContext) -> AnimationSequence {
        match context {
            TransitionContext::ErrorIncrease => Self::bounce_transition(from, to, speed),
            TransitionContext::ErrorDecrease => Self::fade_transition(from, to, speed),
            TransitionContext::ActivityChange => Self::slide_transition(from, to, speed),
            TransitionContext::TimeOfDay => Self::smooth_transition(from, to, speed),
            TransitionContext::LongSession => Self::spin_transition(from, to, speed),
            TransitionContext::Default => Self::smooth_transition(from, to, speed),
        }
    }
}

/// Context for personality transitions to choose appropriate animation
#[derive(Debug, Clone, PartialEq)]
pub enum TransitionContext {
    /// Error count increased (frustration)
    ErrorIncrease,
    /// Error count decreased (relief)
    ErrorDecrease,
    /// Activity/tool changed
    ActivityChange,
    /// Time of day changed
    TimeOfDay,
    /// Long session effects kicking in
    LongSession,
    /// Default/generic transition
    Default,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_face() {
        assert_eq!(
            PersonalityTransitions::extract_face("ʕ•ᴥ•ʔ Code Wizard"),
            "ʕ•ᴥ•ʔ"
        );
        assert_eq!(
            PersonalityTransitions::extract_face("(╯°□°)╯︵ ┻━┻ Table Flipper"),
            "(╯°□°)╯︵"
        );
        assert_eq!(
            PersonalityTransitions::extract_face("ಠ_ಠ"),
            "ಠ_ಠ"
        );
    }
    
    #[test]
    fn test_smooth_transition() {
        let transition = PersonalityTransitions::smooth_transition(
            "ʕ•ᴥ•ʔ Code Wizard", 
            "(˘ ³˘) Booting Up", 
            AnimationSpeed::Normal
        );
        
        assert!(transition.frames.len() > 2); // Should have multiple frames
        assert_eq!(transition.frames[0].content, "ʕ•ᴥ•ʔ");
        assert_eq!(transition.frames[transition.frames.len() - 1].content, "(˘ ³˘)");
        assert!(!transition.loops); // Transitions shouldn't loop
    }
    
    #[test]
    fn test_fade_transition() {
        let transition = PersonalityTransitions::fade_transition(
            "ʕ•ᴥ•ʔ", 
            "(˘ ³˘)", 
            AnimationSpeed::Fast
        );
        
        assert_eq!(transition.frames.len(), 3);
        assert_eq!(transition.frames[0].content, "ʕ•ᴥ•ʔ");
        assert_eq!(transition.frames[1].content, ""); // Blank frame
        assert_eq!(transition.frames[2].content, "(˘ ³˘)");
    }
    
    #[test]
    fn test_slide_transition() {
        let transition = PersonalityTransitions::slide_transition(
            "ʕ•ᴥ•ʔ", 
            "(˘ ³˘)", 
            AnimationSpeed::Normal
        );
        
        assert_eq!(transition.frames.len(), 7);
        
        // Check that some frames have offsets
        let has_offsets = transition.frames.iter().any(|f| f.offset_x != 0);
        assert!(has_offsets);
        
        // First and last frames should be the personalities
        assert_eq!(transition.frames[0].content, "ʕ•ᴥ•ʔ");
        assert_eq!(transition.frames[6].content, "(˘ ³˘)");
    }
    
    #[test]
    fn test_bounce_transition() {
        let transition = PersonalityTransitions::bounce_transition(
            "ʕ•ᴥ•ʔ", 
            "(˘ ³˘)", 
            AnimationSpeed::Normal
        );
        
        assert_eq!(transition.frames.len(), 5);
        
        // Check vertical offsets
        assert_eq!(transition.frames[1].offset_y, -1); // Up
        assert_eq!(transition.frames[3].offset_y, 1);  // Down
        
        // Check blank frame in middle
        assert_eq!(transition.frames[2].content, "");
    }
    
    #[test]
    fn test_spin_transition() {
        let transition = PersonalityTransitions::spin_transition(
            "ʕ•ᴥ•ʔ", 
            "(˘ ³˘)", 
            AnimationSpeed::Normal
        );
        
        assert_eq!(transition.frames.len(), 5);
        
        // First frame should be original face
        assert_eq!(transition.frames[0].content, "ʕ•ᴥ•ʔ");
        
        // Last frame should be target face
        assert_eq!(transition.frames[4].content, "(˘ ³˘)");
        
        // Middle frames should be different (rotated)
        assert_ne!(transition.frames[1].content, transition.frames[0].content);
        assert_ne!(transition.frames[2].content, transition.frames[0].content);
    }
    
    #[test]
    fn test_morph_single_character() {
        // Test eye morphing
        let result = PersonalityTransitions::morph_single_character('•', '°', 0.5);
        assert_ne!(result, '•');
        assert_ne!(result, '°');
        
        // Test progress boundaries
        let result = PersonalityTransitions::morph_single_character('•', '°', 0.0);
        assert_eq!(result, '•');
        
        let result = PersonalityTransitions::morph_single_character('•', '°', 1.0);
        assert_eq!(result, '°');
        
        // Test same character
        let result = PersonalityTransitions::morph_single_character('a', 'a', 0.5);
        assert_eq!(result, 'a');
    }
    
    #[test]
    fn test_contextual_transition() {
        let from = "ʕ•ᴥ•ʔ";
        let to = "(╯°□°)╯";
        let speed = AnimationSpeed::Normal;
        
        // Test error increase uses bounce
        let transition = PersonalityTransitions::contextual_transition(
            from, to, speed, TransitionContext::ErrorIncrease
        );
        // Bounce has 5 frames and vertical offsets
        assert_eq!(transition.frames.len(), 5);
        assert!(transition.frames.iter().any(|f| f.offset_y != 0));
        
        // Test error decrease uses fade
        let transition = PersonalityTransitions::contextual_transition(
            from, to, speed, TransitionContext::ErrorDecrease
        );
        // Fade has 3 frames with blank middle
        assert_eq!(transition.frames.len(), 3);
        assert_eq!(transition.frames[1].content, "");
        
        // Test activity change uses slide
        let transition = PersonalityTransitions::contextual_transition(
            from, to, speed, TransitionContext::ActivityChange
        );
        // Slide has 7 frames with horizontal offsets
        assert_eq!(transition.frames.len(), 7);
        assert!(transition.frames.iter().any(|f| f.offset_x != 0));
        
        // Test default uses smooth
        let transition = PersonalityTransitions::contextual_transition(
            from, to, speed, TransitionContext::Default
        );
        // Smooth has 8 frames
        assert_eq!(transition.frames.len(), 8);
    }
    
    #[test]
    fn test_create_smooth_morph() {
        let frames = PersonalityTransitions::create_smooth_morph("ʕ•ᴥ•ʔ", "(˘ ³˘)", 100);
        
        assert_eq!(frames.len(), 8); // Start + 6 intermediate + end
        assert_eq!(frames[0].content, "ʕ•ᴥ•ʔ");
        assert_eq!(frames[7].content, "(˘ ³˘)");
        
        // Middle frames should be different from start/end
        assert_ne!(frames[3].content, frames[0].content);
        assert_ne!(frames[3].content, frames[7].content);
    }
    
    #[test]
    fn test_rotate_face() {
        let rotated = PersonalityTransitions::rotate_face("(•_•)", 90);
        
        // Should change some characters
        assert_ne!(rotated, "(•_•)");
        
        // Should maintain same length
        assert_eq!(rotated.len(), "(•_•)".len());
        
        // Test with different face
        let rotated2 = PersonalityTransitions::rotate_face("[○_○]", 180);
        assert_ne!(rotated2, "[○_○]");
    }
    
    #[test]
    fn test_get_intermediate_character() {
        // Test known transitions
        let result = PersonalityTransitions::get_intermediate_character('(', '[');
        assert_eq!(result, '⌐');
        
        let result = PersonalityTransitions::get_intermediate_character('•', '°');
        assert_eq!(result, '◦');
        
        // Test same character
        let result = PersonalityTransitions::get_intermediate_character('a', 'a');
        // Should be one of the two (since they're the same)
        assert!(result == 'a');
    }
    
    #[test]
    fn test_transition_context_equality() {
        assert_eq!(TransitionContext::ErrorIncrease, TransitionContext::ErrorIncrease);
        assert_ne!(TransitionContext::ErrorIncrease, TransitionContext::ErrorDecrease);
        assert_eq!(TransitionContext::Default, TransitionContext::Default);
    }
}