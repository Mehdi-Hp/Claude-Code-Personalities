//! Model-specific Nerd Font icons
//!
//! These icons differentiate between different Claude models and AI systems.
//! Currently using different crown/star variations for different model tiers.

/// Claude Opus - Custom icon (premium/powerful model)
pub const ICON_OPUS: &str = "\u{f4f5}"; // 

/// Claude Sonnet - Diamond icon (balanced model)  
pub const ICON_SONNET: &str = "\u{f219}"; // 

/// Claude Haiku - Leaf icon (fast/lightweight model)
pub const ICON_HAIKU: &str = "\u{f06c}"; // 

/// Default Claude icon - North star (fallback for unknown models)
pub const ICON_CLAUDE_DEFAULT: &str = "\u{f3f5}"; // 
