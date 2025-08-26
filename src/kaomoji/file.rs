//! File-type specific kaomoji personalities
//!
//! These kaomojis are triggered by specific file types and extensions,
//! providing specialized personalities for different kinds of development work.

use super::Kaomoji;

// Security and Analysis
pub const SECURITY_ANALYST: Kaomoji = Kaomoji::new("ಠ_ಠ", "Security Analyst");
pub const PERFORMANCE_TUNER: Kaomoji = Kaomoji::new("★⌒ヽ( ͡° ε ͡°)", "Performance Tuner");

// Documentation and Content
pub const DOCUMENTATION_WRITER: Kaomoji = Kaomoji::new("φ(．．)", "Documentation Writer");

// Frontend and UI
pub const UI_DEVELOPER: Kaomoji = Kaomoji::new("(✿◠ᴗ◠)", "UI Developer");
pub const STYLE_ARTIST: Kaomoji = Kaomoji::new("♥‿♥", "Style Artist");
pub const MARKUP_WIZARD: Kaomoji = Kaomoji::new("<(￣︶￣)>", "Markup Wizard");

// Programming Languages
pub const JS_MASTER: Kaomoji = Kaomoji::new("(▀̿Ĺ̯▀̿ ̿)", "JS Master");

// Configuration and Settings
pub const CONFIG_HELPER: Kaomoji = Kaomoji::new("(๑>؂•̀๑)", "Config Helper");
