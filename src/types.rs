use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};
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
    /// Refactoring code (mass changes, renames)
    Refactoring,
    /// Writing documentation
    Documenting,
    /// Deploying applications
    Deploying,
}

impl Display for Activity {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let s = match self {
            Activity::Editing => "Editing",
            Activity::Coding => "Coding",
            Activity::Configuring => "Configuring",
            Activity::Navigating => "Navigating",
            Activity::Writing => "Writing",
            Activity::Executing => "Executing",
            Activity::Reading => "Reading",
            Activity::Searching => "Searching",
            Activity::Debugging => "Debugging",
            Activity::Testing => "Testing",
            Activity::Reviewing => "Reviewing",
            Activity::Thinking => "Thinking",
            Activity::Building => "Building",
            Activity::Installing => "Installing",
            Activity::Idle => "Idle",
            Activity::Working => "Working",
            Activity::Refactoring => "Refactoring",
            Activity::Documenting => "Documenting",
            Activity::Deploying => "Deploying",
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
            "refactoring" => Activity::Refactoring,
            "documenting" => Activity::Documenting,
            "deploying" => Activity::Deploying,
            _ => Activity::Working,
        })
    }
}

impl Activity {
    /// Convert from string (case-insensitive) - convenience method for tests
    #[cfg(test)]
    #[must_use]
    pub fn parse_activity(s: &str) -> Self {
        s.parse().unwrap_or(Activity::Working)
    }
}
