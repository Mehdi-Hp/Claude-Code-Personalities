use std::fmt::{Display, Formatter, Result as FmtResult};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Activity types that Claude can be performing
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Activity {
    /// General editing of files
    Editing,
    /// Editing code files specifically
    Coding,
    /// Editing configuration files
    Configuring,
    /// File system navigation
    Navigating,
    /// Writing new files
    Writing,
    /// Executing commands
    Executing,
    /// Reading files
    Reading,
    /// Searching through files
    Searching,
    /// Debugging code
    Debugging,
    /// Running tests
    Testing,
    /// Reviewing code
    Reviewing,
    /// Processing or thinking
    Thinking,
    /// Building/compiling
    Building,
    /// Installing packages
    Installing,
    /// Idle state
    Idle,
    /// Generic working state
    Working,
}

impl Display for Activity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let s = match self {
            Activity::Editing => "editing",
            Activity::Coding => "coding",
            Activity::Configuring => "configuring",
            Activity::Navigating => "navigating",
            Activity::Writing => "writing",
            Activity::Executing => "executing",
            Activity::Reading => "reading",
            Activity::Searching => "searching",
            Activity::Debugging => "debugging",
            Activity::Testing => "testing",
            Activity::Reviewing => "reviewing",
            Activity::Thinking => "thinking",
            Activity::Building => "building",
            Activity::Installing => "installing",
            Activity::Idle => "idle",
            Activity::Working => "working",
        };
        write!(f, "{s}")
    }
}

impl FromStr for Activity {
    type Err = ();
    
    /// Convert from string (case-insensitive)
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "editing" => Activity::Editing,
            "coding" => Activity::Coding,
            "configuring" => Activity::Configuring,
            "navigating" => Activity::Navigating,
            "writing" => Activity::Writing,
            "executing" => Activity::Executing,
            "reading" => Activity::Reading,
            "searching" => Activity::Searching,
            "debugging" => Activity::Debugging,
            "testing" => Activity::Testing,
            "reviewing" => Activity::Reviewing,
            "thinking" => Activity::Thinking,
            "building" => Activity::Building,
            "installing" => Activity::Installing,
            "idle" => Activity::Idle,
            _ => Activity::Working,
        })
    }
}

impl Activity {
    /// Convert from string (case-insensitive) - convenience method
    #[allow(dead_code)]
    pub fn parse_activity(s: &str) -> Self {
        s.parse().unwrap_or(Activity::Working)
    }
}

/// Hook types for different events
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum HookType {
    PreTool,
    PostTool,
    PromptSubmit,
    SessionEnd,
}

impl FromStr for HookType {
    type Err = ();
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pre-tool" => Ok(HookType::PreTool),
            "post-tool" => Ok(HookType::PostTool),
            "prompt-submit" => Ok(HookType::PromptSubmit),
            "session-end" => Ok(HookType::SessionEnd),
            _ => Err(()),
        }
    }
}

impl HookType {
    /// Try to parse a HookType from a string
    #[allow(dead_code)]
    pub fn parse_hook_type(s: &str) -> Option<Self> {
        s.parse().ok()
    }
}

impl Display for HookType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let s = match self {
            HookType::PreTool => "pre-tool",
            HookType::PostTool => "post-tool",
            HookType::PromptSubmit => "prompt-submit",
            HookType::SessionEnd => "session-end",
        };
        write!(f, "{s}")
    }
}

/// Claude model types
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ModelType {
    Opus,
    Sonnet,
    Haiku,
    Other(String),
}

impl FromStr for ModelType {
    type Err = ();
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_lowercase();
        Ok(if lower.contains("opus") {
            ModelType::Opus
        } else if lower.contains("sonnet") {
            ModelType::Sonnet
        } else if lower.contains("haiku") {
            ModelType::Haiku
        } else {
            ModelType::Other(s.to_string())
        })
    }
}

impl ModelType {
    /// Parse a ModelType from a string - convenience method
    #[allow(dead_code)]
    pub fn parse_model(s: &str) -> Self {
        s.parse().unwrap_or_else(|_| ModelType::Other(s.to_string()))
    }
    
    #[allow(dead_code)]
    pub fn color(&self) -> &'static str {
        match self {
            ModelType::Opus => "magenta",
            ModelType::Sonnet => "cyan",
            ModelType::Haiku => "green",
            ModelType::Other(_) => "white",
        }
    }
}

impl Display for ModelType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let s = match self {
            ModelType::Opus => "Opus",
            ModelType::Sonnet => "Sonnet", 
            ModelType::Haiku => "Haiku",
            ModelType::Other(name) => name,
        };
        write!(f, "{s}")
    }
}