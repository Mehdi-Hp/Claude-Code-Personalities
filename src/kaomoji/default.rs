//! Default kaomoji personalities
//!
//! These are fallback kaomojis used when no specific personality matches,
//! or for basic tool operations and initial states.

use super::Kaomoji;

// Initial state
pub const BOOTING_UP: Kaomoji = Kaomoji::new("( ˘ ³˘)", "Booting Up");

// Basic tool operations
pub const CODE_WIZARD: Kaomoji = Kaomoji::new("ʕ•ᴥ•ʔ", "Code Wizard");
pub const CODE_WIZARD_ALT: Kaomoji = Kaomoji::new("(⌐■_■)", "Code Wizard");
pub const GENTLE_REFACTORER: Kaomoji = Kaomoji::new("(• ε •)", "Gentle Refactorer");
pub const CODE_JANITOR: Kaomoji = Kaomoji::new("(ง'̀-'́)ง", "Code Janitor");
pub const CASUAL_CODE_REVIEWER: Kaomoji = Kaomoji::new("¯\\_(ツ)_/¯", "Casual Code Reviewer");
