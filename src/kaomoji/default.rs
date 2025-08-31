//! Default kaomoji personalities
//!
//! These are fallback kaomojis used when no specific personality matches,
//! or for basic tool operations and initial states.

use super::Kaomoji;

// Initial state
pub const BOOTING_UP: Kaomoji = Kaomoji::new("( ˘ ³˘)", "Chillin");

// Basic tool operations
pub const CODE_WIZARD: Kaomoji = Kaomoji::new("ლ(╹◡╹ლ)", "Cowder");
pub const CODE_WIZARD_ALT: Kaomoji = Kaomoji::new("ლ(╹◡╹ლ)", "Cowder");
pub const GENTLE_REFACTORER: Kaomoji = Kaomoji::new("(• ε •)", "Gentle Refactorer");
pub const CODE_JANITOR: Kaomoji = Kaomoji::new("(ง'̀-'́)ง", "Code Janitor");
pub const CASUAL_CODE_REVIEWER: Kaomoji = Kaomoji::new("¯\\_(ツ)_/¯", "Casual Code Reviewer");
