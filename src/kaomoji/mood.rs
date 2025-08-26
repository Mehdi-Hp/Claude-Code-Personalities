//! Mood-based kaomoji personalities
//!
//! These kaomojis represent Claude's emotional state based on error counts,
//! momentum, and overall session performance.

use super::Kaomoji;

/// High frustration state - multiple consecutive errors
pub const FRUSTRATED_HIGH: Kaomoji = Kaomoji::new("(╯°□°)╯︵ ┻━┻", "Table Flipper");

/// Mid-level frustration - some errors encountered
pub const FRUSTRATED_MID: Kaomoji = Kaomoji::new("(ノಠ益ಠ)ノ", "Error Warrior");

/// Highly focused state - good momentum
pub const HYPERFOCUSED: Kaomoji = Kaomoji::new("┌༼◉ل͟◉༽┐", "Hyperfocused Coder");

/// Extreme focus state - very high consecutive actions
pub const CODE_BERSERKER: Kaomoji = Kaomoji::new("【╯°□°】╯︵ ┻━┻", "Code Berserker");
