use super::{AnimationFrame, AnimationSequence, AnimationSpeed};
use crate::types::Activity;

/// Activity-specific animations that show what Claude is currently doing
pub struct ActivityAnimations;

impl ActivityAnimations {
    /// Get animation for a specific activity
    pub fn for_activity(activity: &Activity, speed: AnimationSpeed) -> Option<AnimationSequence> {
        match activity {
            Activity::Editing => Some(Self::editing_animation(speed)),
            Activity::Reading => Some(Self::reading_animation(speed)),
            Activity::Writing => Some(Self::writing_animation(speed)),
            Activity::Searching => Some(Self::searching_animation(speed)),
            Activity::Building => Some(Self::building_animation(speed)),
            Activity::Testing => Some(Self::testing_animation(speed)),
            Activity::Debugging => Some(Self::debugging_animation(speed)),
            Activity::Installing => Some(Self::installing_animation(speed)),
            Activity::Thinking => Some(Self::thinking_animation(speed)),
            Activity::Idle => Some(Self::idle_animation(speed)),
            Activity::Coding => Some(Self::coding_animation(speed)),
            Activity::Navigating => Some(Self::navigating_animation(speed)),
            Activity::Reviewing => Some(Self::reviewing_animation(speed)),
            Activity::Executing => Some(Self::git_animation(speed)), // Git operations are typically executed
            Activity::Configuring => Some(Self::config_animation(speed)),
            Activity::Working => Some(Self::working_animation(speed)),
        }
    }
    
    /// Animation for code editing activities
    fn editing_animation(speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms();
        let frames = vec![
            AnimationFrame::new("(⌐■_■)", duration),
            AnimationFrame::new("(⌐■_□)", duration),
            AnimationFrame::new("(⌐□_■)", duration),
            AnimationFrame::new("(⌐■_■)", duration),
        ];
        AnimationSequence::new(frames).with_loops()
    }
    
    /// Animation for file reading activities
    fn reading_animation(speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms() * 2; // Slower for reading
        let frames = vec![
            AnimationFrame::new("╭༼ ◔◡◔ ༽╮", duration),
            AnimationFrame::new("╭༼ ◕◡◕ ༽╮", duration),
            AnimationFrame::new("╭༼ ●◡● ༽╮", duration),
            AnimationFrame::new("╭༼ ◕◡◕ ༽╮", duration),
        ];
        AnimationSequence::new(frames).with_loops()
    }
    
    /// Animation for writing/creating files
    fn writing_animation(speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms();
        let frames = vec![
            AnimationFrame::new("(• ε •)", duration),
            AnimationFrame::new("(• ε •)✎", duration),
            AnimationFrame::new("(• ε •)✍", duration),
            AnimationFrame::new("(• ε •)✎", duration),
        ];
        AnimationSequence::new(frames).with_loops()
    }
    
    /// Animation for search operations
    fn searching_animation(speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms();
        let frames = vec![
            AnimationFrame::new("(◔_◔)", duration),
            AnimationFrame::new("(◔_◔)ノ", duration),
            AnimationFrame::new("(◕_◔)", duration),
            AnimationFrame::new("(◔_◕)", duration),
            AnimationFrame::new("(◔_◔)", duration),
        ];
        AnimationSequence::new(frames).with_loops()
    }
    
    /// Animation for build/compile operations
    fn building_animation(speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms();
        let frames = vec![
            AnimationFrame::new("ᕦ(ò_óˇ)ᕤ", duration),
            AnimationFrame::new("ᕦ(ó_òˇ)ᕤ", duration),
            AnimationFrame::new("ᕦ(ò_óˇ)ᕤ⚡", duration),
            AnimationFrame::new("ᕦ(ó_òˇ)ᕤ", duration),
        ];
        AnimationSequence::new(frames).with_loops()
    }
    
    /// Animation for test execution
    fn testing_animation(speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms() * 2; // Slower for testing
        let frames = vec![
            AnimationFrame::new("( ദ്ദി ˙ᗜ˙ )", duration),
            AnimationFrame::new("( ദ്ദി ˙ᗜ˙ )✓", duration),
            AnimationFrame::new("( ദ്ദി ˙ᗜ˙ )✓✓", duration),
            AnimationFrame::new("( ദ്ദി ˙ᗜ˙ )✓✓✓", duration),
            AnimationFrame::new("( ദ്ദി ˙ᗜ˙ )", duration),
        ];
        AnimationSequence::new(frames).with_loops()
    }
    
    /// Animation for debugging activities
    fn debugging_animation(speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms();
        let frames = vec![
            AnimationFrame::new("(つ◉益◉)つ", duration),
            AnimationFrame::new("(つ◔益◔)つ", duration),
            AnimationFrame::new("(つ●益●)つ", duration),
            AnimationFrame::new("(つ◔益◔)つ", duration),
            AnimationFrame::new("(つ◉益◉)つ\u{f002}", duration), // Search icon
        ];
        AnimationSequence::new(frames).with_loops()
    }
    
    /// Animation for package installation
    fn installing_animation(speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms();
        let frames = vec![
            AnimationFrame::new("^⎚-⎚^", duration),
            AnimationFrame::new("^⎚-⎚^ ⬇", duration),
            AnimationFrame::new("^⎚-⎚^ ⬇⬇", duration),
            AnimationFrame::new("^⎚-⎚^ ⬇⬇⬇", duration),
            AnimationFrame::new("^⎚-⎚^ ✓", duration),
            AnimationFrame::new("^⎚-⎚^", duration),
        ];
        AnimationSequence::new(frames).with_loops()
    }
    
    /// Animation for thinking/processing
    fn thinking_animation(speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms() * 3; // Much slower for thinking
        let frames = vec![
            AnimationFrame::new("(⌐■_■)", duration),
            AnimationFrame::new("(⌐■_■).", duration),
            AnimationFrame::new("(⌐■_■)..", duration),
            AnimationFrame::new("(⌐■_■)...", duration),
            AnimationFrame::new("(⌐■_■)..", duration),
            AnimationFrame::new("(⌐■_■).", duration),
        ];
        AnimationSequence::new(frames).with_loops()
    }
    
    /// Animation for idle state
    fn idle_animation(speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms() * 5; // Very slow for idle
        let frames = vec![
            AnimationFrame::new("( ˘ ³˘)", duration),
            AnimationFrame::new("( ˘ ³˘)~", duration),
            AnimationFrame::new("( ˘ ³˘)", duration),
        ];
        AnimationSequence::new(frames).with_loops()
    }
    
    /// Animation for Git operations
    fn git_animation(speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms();
        let frames = vec![
            AnimationFrame::new("┗(▀̿Ĺ̯▀̿ ̿)┓", duration),
            AnimationFrame::new("┗(▀̿Ĺ̯▀̿ ̿)┓⚡", duration),
            AnimationFrame::new("┗(▀̿Ĺ̯▀̿ ̿)┓⚡git", duration),
            AnimationFrame::new("┗(▀̿Ĺ̯▀̿ ̿)┓", duration),
        ];
        AnimationSequence::new(frames).with_loops()
    }
    
    /// Animation for configuration/setup activities
    fn config_animation(speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms();
        let frames = vec![
            AnimationFrame::new("(๑>؂•̀๑)", duration),
            AnimationFrame::new("(๑>؂•̀๑)⚙", duration),
            AnimationFrame::new("(๑>؂•̀๑)⚙⚙", duration),
            AnimationFrame::new("(๑>؂•̀๑)⚙", duration),
        ];
        AnimationSequence::new(frames).with_loops()
    }
    
    /// Animation for generic working state
    fn working_animation(speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms();
        let frames = vec![
            AnimationFrame::new("(⌐■_■)", duration),
            AnimationFrame::new("(⌐■_■)⚡", duration),
            AnimationFrame::new("(⌐■_■)⚙", duration),
            AnimationFrame::new("(⌐■_■)", duration),
        ];
        AnimationSequence::new(frames).with_loops()
    }
    
    /// Animation for coding activities (more technical)
    fn coding_animation(speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms();
        let frames = vec![
            AnimationFrame::new("(▀̿Ĺ̯▀̿ ̿)", duration),
            AnimationFrame::new("(▀̿Ĺ̯▀̿ ̿)\u{f121}", duration), // Code icon
            AnimationFrame::new("(▀̿Ĺ̯▀̿ ̿){}", duration),
            AnimationFrame::new("(▀̿Ĺ̯▀̿ ̿)", duration),
        ];
        AnimationSequence::new(frames).with_loops()
    }
    
    /// Animation for navigating file system
    fn navigating_animation(speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms();
        let frames = vec![
            AnimationFrame::new("ᓚ₍ ^. .^₎", duration),
            AnimationFrame::new("ᓚ₍ ^. .^₎\u{f07b}", duration), // Folder icon
            AnimationFrame::new("ᓚ₍ ^. .^₎\u{f002}", duration), // Search icon
            AnimationFrame::new("ᓚ₍ ^. .^₎", duration),
        ];
        AnimationSequence::new(frames).with_loops()
    }
    
    /// Animation for code review activities
    fn reviewing_animation(speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms() * 2; // Slower for reviewing
        let frames = vec![
            AnimationFrame::new("¯\\_(ツ)_/¯", duration),
            AnimationFrame::new("¯\\_(ツ)_/¯\u{f06e}", duration), // Eye icon
            AnimationFrame::new("¯\\_(ツ)_/¯\u{f00c}", duration), // Check icon
            AnimationFrame::new("¯\\_(ツ)_/¯", duration),
        ];
        AnimationSequence::new(frames).with_loops()
    }
    
    /// Get contextual activity animation based on file being worked on
    pub fn contextual_activity_animation(activity: &Activity, file_path: Option<&str>, speed: AnimationSpeed) -> Option<AnimationSequence> {
        match (activity, file_path) {
            (Activity::Editing, Some(path)) if path.ends_with(".rs") => {
                Some(Self::rust_editing_animation(speed))
            },
            (Activity::Editing, Some(path)) if path.ends_with(".js") || path.ends_with(".ts") => {
                Some(Self::js_editing_animation(speed))
            },
            (Activity::Editing, Some(path)) if path.ends_with(".py") => {
                Some(Self::python_editing_animation(speed))
            },
            (Activity::Testing, Some(path)) if path.contains("test") => {
                Some(Self::enhanced_testing_animation(speed))
            },
            (Activity::Reading, Some(path)) if path.ends_with(".md") => {
                Some(Self::docs_reading_animation(speed))
            },
            _ => Self::for_activity(activity, speed), // Fallback to generic
        }
    }
    
    /// Rust-specific editing animation
    fn rust_editing_animation(speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms();
        let frames = vec![
            AnimationFrame::new("(⌐■_■)rs", duration),
            AnimationFrame::new("(⌐■_□)rs", duration),
            AnimationFrame::new("(⌐□_■)rs", duration),
            AnimationFrame::new("(⌐■_■)rs", duration),
        ];
        AnimationSequence::new(frames).with_loops()
    }
    
    /// JavaScript/TypeScript-specific editing animation
    fn js_editing_animation(speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms();
        let frames = vec![
            AnimationFrame::new("(▀̿Ĺ̯▀̿ ̿)JS", duration),
            AnimationFrame::new("(▀̿Ĺ̯▀̿ ̿)⚡", duration),
            AnimationFrame::new("(▀̿Ĺ̯▀̿ ̿){}", duration),
            AnimationFrame::new("(▀̿Ĺ̯▀̿ ̿)JS", duration),
        ];
        AnimationSequence::new(frames).with_loops()
    }
    
    /// Python-specific editing animation
    fn python_editing_animation(speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms();
        let frames = vec![
            AnimationFrame::new("(⌐■_■)py", duration),
            AnimationFrame::new("(⌐■_□)py", duration),
            AnimationFrame::new("(⌐□_■)py", duration),
            AnimationFrame::new("(⌐■_■)py", duration),
        ];
        AnimationSequence::new(frames).with_loops()
    }
    
    /// Enhanced testing animation for test files
    fn enhanced_testing_animation(speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms();
        let frames = vec![
            AnimationFrame::new("( ദ്ദി ˙ᗜ˙ )test", duration),
            AnimationFrame::new("( ദ്ദി ˙ᗜ˙ )⚡test", duration),
            AnimationFrame::new("( ദ്ദി ˙ᗜ˙ )✓test", duration),
            AnimationFrame::new("( ദ്ദി ˙ᗜ˙ )test", duration),
        ];
        AnimationSequence::new(frames).with_loops()
    }
    
    /// Documentation reading animation
    fn docs_reading_animation(speed: AnimationSpeed) -> AnimationSequence {
        let duration = speed.frame_duration_ms() * 2;
        let frames = vec![
            AnimationFrame::new("φ(．．)\u{f15c}", duration), // Book icon
            AnimationFrame::new("φ(．．)\u{f15b}", duration), // File icon
            AnimationFrame::new("φ(．．)\u{f044}", duration), // Edit icon
            AnimationFrame::new("φ(．．)\u{f15c}", duration), // Book icon
        ];
        AnimationSequence::new(frames).with_loops()
    }
    
    /// Create a combined activity animation (multiple activities happening)
    pub fn combined_activity_animation(activities: &[Activity], speed: AnimationSpeed) -> Option<AnimationSequence> {
        if activities.is_empty() {
            return None;
        }
        
        if activities.len() == 1 {
            return Self::for_activity(&activities[0], speed);
        }
        
        // Create a sequence that shows multiple activities
        let duration = speed.frame_duration_ms();
        let mut frames = Vec::new();
        
        for activity in activities {
            if let Some(activity_seq) = Self::for_activity(activity, speed) {
                // Take first frame from each activity animation
                if let Some(frame) = activity_seq.frames.first() {
                    frames.push(AnimationFrame::new(&frame.content, duration));
                }
            }
        }
        
        if frames.is_empty() {
            None
        } else {
            Some(AnimationSequence::new(frames).with_loops())
        }
    }
    
    /// Get activity intensity level (for animation speed adjustment)
    pub fn get_activity_intensity(activity: &Activity) -> f32 {
        match activity {
            Activity::Working => 1.1,      // Slightly intense
            Activity::Building => 1.5,     // High intensity  
            Activity::Testing => 1.3,      // Medium-high intensity
            Activity::Debugging => 1.2,    // Medium intensity
            Activity::Coding => 1.1,       // Slightly intense
            Activity::Reviewing => 0.8,    // Relaxed
            Activity::Navigating => 0.9,   // Slightly relaxed
            Activity::Executing => 1.3,    // Medium-high intensity
            Activity::Configuring => 0.9,  // Slightly relaxed
            Activity::Editing => 1.0,      // Normal intensity
            Activity::Writing => 0.9,      // Slightly relaxed
            Activity::Installing => 0.8,   // Relaxed
            Activity::Reading => 0.7,      // Low intensity
            Activity::Searching => 0.8,    // Relaxed
            Activity::Thinking => 0.5,     // Very relaxed
            Activity::Idle => 0.3,         // Minimal intensity
        }
    }
    
    /// Adjust animation speed based on activity intensity
    pub fn adjust_speed_for_intensity(base_speed: AnimationSpeed, activity: &Activity) -> AnimationSpeed {
        let intensity = Self::get_activity_intensity(activity);
        
        match base_speed {
            AnimationSpeed::Slow => {
                if intensity > 1.5 { AnimationSpeed::Normal }
                else if intensity > 1.0 { AnimationSpeed::Slow }
                else { AnimationSpeed::Slow }
            },
            AnimationSpeed::Normal => {
                if intensity > 1.5 { AnimationSpeed::Fast }
                else if intensity > 1.2 { AnimationSpeed::Normal }
                else if intensity < 0.7 { AnimationSpeed::Slow }
                else { AnimationSpeed::Normal }
            },
            AnimationSpeed::Fast => {
                if intensity > 1.5 { AnimationSpeed::Fast }
                else if intensity < 0.7 { AnimationSpeed::Normal }
                else { AnimationSpeed::Fast }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_for_activity() {
        // Test that all activities return some animation
        let activities = [
            Activity::Editing,
            Activity::Coding,
            Activity::Configuring,
            Activity::Navigating,
            Activity::Writing,
            Activity::Executing,
            Activity::Reading,
            Activity::Searching,
            Activity::Debugging,
            Activity::Testing,
            Activity::Reviewing,
            Activity::Thinking,
            Activity::Building,
            Activity::Installing,
            Activity::Idle,
            Activity::Working,
        ];
        
        for activity in &activities {
            let animation = ActivityAnimations::for_activity(activity, AnimationSpeed::Normal);
            assert!(animation.is_some(), "Activity {:?} should have animation", activity);
            
            let seq = animation.unwrap();
            assert!(!seq.frames.is_empty(), "Activity {:?} should have frames", activity);
            assert!(seq.loops, "Activity animations should loop");
        }
    }
    
    #[test]
    fn test_editing_animation() {
        let animation = ActivityAnimations::editing_animation(AnimationSpeed::Normal);
        
        assert_eq!(animation.frames.len(), 4);
        assert!(animation.loops);
        assert_eq!(animation.frames[0].content, "(⌐■_■)");
        assert_eq!(animation.frames[1].content, "(⌐■_□)");
        assert_eq!(animation.frames[2].content, "(⌐□_■)");
        assert_eq!(animation.frames[3].content, "(⌐■_■)");
    }
    
    #[test] 
    fn test_thinking_animation() {
        let animation = ActivityAnimations::thinking_animation(AnimationSpeed::Fast);
        
        assert_eq!(animation.frames.len(), 6);
        assert!(animation.loops);
        
        // Should have progressive dots
        assert_eq!(animation.frames[0].content, "(⌐■_■)");
        assert_eq!(animation.frames[1].content, "(⌐■_■).");
        assert_eq!(animation.frames[2].content, "(⌐■_■)..");
        assert_eq!(animation.frames[3].content, "(⌐■_■)...");
        
        // Check that thinking has longer duration
        let thinking_duration = animation.frames[0].duration_ms;
        let normal_duration = AnimationSpeed::Fast.frame_duration_ms();
        assert!(thinking_duration > normal_duration);
    }
    
    #[test]
    fn test_error_animation() {
        let animation = ActivityAnimations::error_animation(AnimationSpeed::Normal);
        
        assert_eq!(animation.frames.len(), 6);
        assert!(animation.loops);
        
        // Check for shake effects (offsets)
        let has_offsets = animation.frames.iter().any(|f| f.offset_x != 0);
        assert!(has_offsets);
        
        // Error animation should be faster than normal
        let error_duration = animation.frames[0].duration_ms;
        let normal_duration = AnimationSpeed::Normal.frame_duration_ms();
        assert!(error_duration < normal_duration);
    }
    
    #[test]
    fn test_contextual_activity_animation() {
        // Test Rust file editing
        let animation = ActivityAnimations::contextual_activity_animation(
            &Activity::Editing, 
            Some("main.rs"), 
            AnimationSpeed::Normal
        );
        assert!(animation.is_some());
        let frames = animation.unwrap().frames;
        assert!(frames[0].content.contains("rs")); // Should have Rust indicator
        
        // Test JavaScript file editing
        let animation = ActivityAnimations::contextual_activity_animation(
            &Activity::Editing,
            Some("script.js"),
            AnimationSpeed::Normal
        );
        assert!(animation.is_some());
        let frames = animation.unwrap().frames;
        assert!(frames[0].content.contains("JS") || frames[2].content.contains("{}"));
        
        // Test Python file editing
        let animation = ActivityAnimations::contextual_activity_animation(
            &Activity::Editing,
            Some("main.py"),
            AnimationSpeed::Normal
        );
        assert!(animation.is_some());
        let frames = animation.unwrap().frames;
        assert!(frames[0].content.contains("py")); // Should have Python indicator
        
        // Test documentation reading
        let animation = ActivityAnimations::contextual_activity_animation(
            &Activity::Reading,
            Some("README.md"),
            AnimationSpeed::Normal
        );
        assert!(animation.is_some());
        let frames = animation.unwrap().frames;
        assert!(frames[0].content.contains('\u{f15c}') || frames[1].content.contains('\u{f15b}')); // Should have Nerd Font icons
        
        // Test fallback to generic
        let animation = ActivityAnimations::contextual_activity_animation(
            &Activity::Editing,
            Some("unknown.xyz"),
            AnimationSpeed::Normal
        );
        assert!(animation.is_some());
        let frames = animation.unwrap().frames;
        assert_eq!(frames[0].content, "(⌐■_■)"); // Should be generic editing
    }
    
    #[test]
    fn test_combined_activity_animation() {
        // Test empty activities
        let animation = ActivityAnimations::combined_activity_animation(&[], AnimationSpeed::Normal);
        assert!(animation.is_none());
        
        // Test single activity
        let animation = ActivityAnimations::combined_activity_animation(
            &[Activity::Editing],
            AnimationSpeed::Normal
        );
        assert!(animation.is_some());
        assert_eq!(animation.unwrap().frames[0].content, "(⌐■_■)");
        
        // Test multiple activities
        let animation = ActivityAnimations::combined_activity_animation(
            &[Activity::Editing, Activity::Testing, Activity::Building],
            AnimationSpeed::Normal
        );
        assert!(animation.is_some());
        let seq = animation.unwrap();
        assert_eq!(seq.frames.len(), 3); // One frame per activity
        assert!(seq.loops);
    }
    
    #[test]
    fn test_get_activity_intensity() {
        assert_eq!(ActivityAnimations::get_activity_intensity(&Activity::Error), 2.0);
        assert_eq!(ActivityAnimations::get_activity_intensity(&Activity::Building), 1.5);
        assert_eq!(ActivityAnimations::get_activity_intensity(&Activity::Editing), 1.0);
        assert_eq!(ActivityAnimations::get_activity_intensity(&Activity::Reading), 0.7);
        assert_eq!(ActivityAnimations::get_activity_intensity(&Activity::Idle), 0.3);
    }
    
    #[test]
    fn test_adjust_speed_for_intensity() {
        // High intensity activities should speed up
        let adjusted = ActivityAnimations::adjust_speed_for_intensity(
            AnimationSpeed::Normal, 
            &Activity::Error
        );
        assert_eq!(adjusted, AnimationSpeed::Fast);
        
        // Low intensity activities should slow down
        let adjusted = ActivityAnimations::adjust_speed_for_intensity(
            AnimationSpeed::Normal,
            &Activity::Reading
        );
        assert_eq!(adjusted, AnimationSpeed::Slow);
        
        // Normal intensity should stay the same
        let adjusted = ActivityAnimations::adjust_speed_for_intensity(
            AnimationSpeed::Normal,
            &Activity::Editing
        );
        assert_eq!(adjusted, AnimationSpeed::Normal);
        
        // Fast base speed with high intensity should stay fast
        let adjusted = ActivityAnimations::adjust_speed_for_intensity(
            AnimationSpeed::Fast,
            &Activity::Error
        );
        assert_eq!(adjusted, AnimationSpeed::Fast);
        
        // Slow base speed with high intensity should speed up
        let adjusted = ActivityAnimations::adjust_speed_for_intensity(
            AnimationSpeed::Slow,
            &Activity::Building
        );
        assert_eq!(adjusted, AnimationSpeed::Normal);
    }
    
    #[test]
    fn test_rust_editing_animation() {
        let animation = ActivityAnimations::rust_editing_animation(AnimationSpeed::Normal);
        
        assert_eq!(animation.frames.len(), 4);
        assert!(animation.loops);
        
        // All frames should contain the Rust crab emoji
        for frame in &animation.frames {
            assert!(frame.content.contains("rs"));
        }
    }
    
    #[test]
    fn test_testing_animation() {
        let animation = ActivityAnimations::testing_animation(AnimationSpeed::Normal);
        
        assert_eq!(animation.frames.len(), 5);
        assert!(animation.loops);
        
        // Should have progressive checkmarks
        assert_eq!(animation.frames[0].content, "( ദ്ദി ˙ᗜ˙ )");
        assert_eq!(animation.frames[1].content, "( ദ്ദി ˙ᗜ˙ )✓");
        assert_eq!(animation.frames[2].content, "( ദ്ദി ˙ᗜ˙ )✓✓");
        assert_eq!(animation.frames[3].content, "( ദ്ദി ˙ᗜ˙ )✓✓✓");
        assert_eq!(animation.frames[4].content, "( ദ്ദി ˙ᗜ˙ )");
        
        // Testing should have longer duration
        let testing_duration = animation.frames[0].duration_ms;
        let normal_duration = AnimationSpeed::Normal.frame_duration_ms();
        assert!(testing_duration > normal_duration);
    }
    
    #[test]
    fn test_installing_animation() {
        let animation = ActivityAnimations::installing_animation(AnimationSpeed::Normal);
        
        assert_eq!(animation.frames.len(), 6);
        assert!(animation.loops);
        
        // Should show progressive download arrows and success
        assert!(animation.frames[1].content.contains("⬇"));
        assert!(animation.frames[2].content.contains("⬇⬇"));
        assert!(animation.frames[3].content.contains("⬇⬇⬇"));
        assert!(animation.frames[4].content.contains("✓"));
    }
}