//! Tool-specific kaomoji personalities
//!
//! These kaomojis are triggered by specific tools and commands,
//! providing context-aware personalities for different development activities.

use super::Kaomoji;

// Version Control
pub const GIT_MANAGER: Kaomoji = Kaomoji::new("┗(▀̿Ĺ̯▀̿ ̿)┓", "Git Manager");
pub const CODE_HISTORIAN: Kaomoji = Kaomoji::new("(╯︵╰,)", "Code Historian");

// Testing and Quality
pub const TEST_TASKMASTER: Kaomoji = Kaomoji::new("( ദ്ദി ˙ᗜ˙ )", "Test Taskmaster");
pub const BUG_HUNTER: Kaomoji = Kaomoji::new("(つ◉益◉)つ", "Bug Hunter");
pub const QUALITY_AUDITOR: Kaomoji = Kaomoji::new("৻( •̀ ᗜ •́ ৻)", "Quality Auditor");

// Development and Build
pub const COMPILATION_WARRIOR: Kaomoji = Kaomoji::new("ᕦ(ò_óˇ)ᕤ", "Compilation Warrior");
pub const DEPENDENCY_WRANGLER: Kaomoji = Kaomoji::new("^⎚-⎚^", "Dependency Wrangler");

// System Operations
pub const DEPLOYMENT_GUARD: Kaomoji = Kaomoji::new("( ͡ _ ͡°)ﾉ⚲", "Deployment Guard");
pub const TASK_ASSASSIN: Kaomoji = Kaomoji::new("(╬ ಠ益ಠ)", "Task Assassin");
pub const NETWORK_SENTINEL: Kaomoji = Kaomoji::new("(╭ರ_ಠ)", "Network Sentinel");
pub const SYSTEM_DETECTIVE: Kaomoji = Kaomoji::new("(◉_◉)", "System Detective");
pub const SYSTEM_ADMIN: Kaomoji = Kaomoji::new("( ͡ಠ ʖ̯ ͡ಠ)", "System Admin");
pub const PERMISSION_POLICE: Kaomoji = Kaomoji::new("(╯‵□′)╯", "Permission Police");

// File and Text Operations
pub const FILE_EXPLORER: Kaomoji = Kaomoji::new("ᓚ₍ ^. .^₎", "File Explorer");
pub const STRING_SURGEON: Kaomoji = Kaomoji::new("(˘▾˘~)", "String Surgeon");
pub const COMPRESSION_CHEF: Kaomoji = Kaomoji::new("(っ˘ڡ˘ς)", "Compression Chef");

// Specialized Tools
pub const DATABASE_EXPERT: Kaomoji = Kaomoji::new("⚆_⚆", "Database Expert");
pub const EDITOR_USER: Kaomoji = Kaomoji::new("( . .)φ", "Editor User");
pub const ENVIRONMENT_ENCHANTER: Kaomoji = Kaomoji::new("(∗´ര ᎑ ര`∗)", "Environment Enchanter");
pub const CONTAINER_CAPTAIN: Kaomoji = Kaomoji::new("(づ｡◕‿‿◕｡)づ", "Container Captain");

// Research and Reading
pub const SEARCH_MAESTRO: Kaomoji = Kaomoji::new("⋋| ◉ ͟ʖ ◉ |⋌", "Search Maestro");
pub const RESEARCH_KING: Kaomoji = Kaomoji::new("╭༼ ººل͟ºº ༽╮", "Research King");
