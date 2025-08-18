use std::fmt::{Display, Formatter, Result as FmtResult};
use serde::{Deserialize, Serialize};

/// Activity types that Claude can be performing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
        write!(f, "{}", s)
    }
}

impl Activity {
    /// Convert from string (case-insensitive)
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
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
        }
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

impl HookType {
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pre-tool" => Some(HookType::PreTool),
            "post-tool" => Some(HookType::PostTool),
            "prompt-submit" => Some(HookType::PromptSubmit),
            "session-end" => Some(HookType::SessionEnd),
            _ => None,
        }
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
        write!(f, "{}", s)
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

impl ModelType {
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Self {
        let lower = s.to_lowercase();
        if lower.contains("opus") {
            ModelType::Opus
        } else if lower.contains("sonnet") {
            ModelType::Sonnet
        } else if lower.contains("haiku") {
            ModelType::Haiku
        } else {
            ModelType::Other(s.to_string())
        }
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
        write!(f, "{}", s)
    }
}