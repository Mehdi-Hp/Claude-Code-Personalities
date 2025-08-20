use std::fmt;
use colored::Colorize;

/// Custom error types for claude-code-personalities
#[derive(Debug)]
#[allow(dead_code)]
pub enum PersonalityError {
    /// File I/O errors
    IO {
        operation: String,
        path: Option<String>,
        source: std::io::Error,
        suggestion: Option<String>,
    },
    /// JSON parsing errors
    Parsing {
        context: String,
        input_preview: Option<String>,
        source: serde_json::Error,
        suggestion: Option<String>,
    },
    /// Session state management errors
    State {
        session_id: String,
        operation: String,
        suggestion: Option<String>,
    },
    /// System environment errors
    System {
        message: String,
        suggestion: Option<String>,
    },
}

impl PersonalityError {
    #[allow(dead_code)]
    pub fn io(operation: impl Into<String>, path: Option<impl Into<String>>, source: std::io::Error) -> Self {
        Self::IO {
            operation: operation.into(),
            path: path.map(std::convert::Into::into),
            source,
            suggestion: None,
        }
    }
    
    #[allow(dead_code)]
    pub fn parsing(context: impl Into<String>, source: serde_json::Error) -> Self {
        Self::Parsing {
            context: context.into(),
            input_preview: None,
            source,
            suggestion: None,
        }
    }
    
    #[allow(dead_code)]
    pub fn state(session_id: impl Into<String>, operation: impl Into<String>) -> Self {
        Self::State {
            session_id: session_id.into(),
            operation: operation.into(),
            suggestion: None,
        }
    }
    
    #[allow(dead_code)]
    pub fn system(message: impl Into<String>) -> Self {
        Self::System {
            message: message.into(),
            suggestion: None,
        }
    }
}

impl fmt::Display for PersonalityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Nerd Font icons using UTF-8 byte sequences
        const ERROR_ICON: &str = "\u{f057}";  // 
        const LIGHTBULB_ICON: &str = "\u{f0eb}"; // 
        
        match self {
            PersonalityError::IO { operation, path, source, suggestion } => {
                if let Some(path) = path {
                    writeln!(f, "{} {}: Failed to {} {}", 
                        ERROR_ICON.red(),
                        "File Error".red().bold(), 
                        operation,
                        path.blue()
                    )?;
                } else {
                    writeln!(f, "{} {}: Failed to {}", 
                        ERROR_ICON.red(),
                        "File Error".red().bold(), 
                        operation
                    )?;
                }
                writeln!(f, "Cause: {}", source.to_string().dimmed())?;
                
                if let Some(suggestion) = suggestion {
                    writeln!(f, "\n{}", suggestion.yellow())?;
                }
            }
            
            PersonalityError::Parsing { context, input_preview, source, suggestion } => {
                writeln!(f, "{} {}: {}", 
                    ERROR_ICON.red(),
                    "Invalid Input".red().bold(), 
                    context
                )?;
                
                if let Some(preview) = input_preview {
                    writeln!(f, "Received: {}", preview.dimmed())?;
                }
                writeln!(f, "Parse error: {}", source.to_string().dimmed())?;
                
                if let Some(suggestion) = suggestion {
                    writeln!(f, "\n{}", suggestion.yellow())?;
                } else {
                    writeln!(f, "\n{}", "This is likely a Claude Code compatibility issue.".yellow())?;
                }
            }
            
            PersonalityError::State { session_id, operation, suggestion } => {
                writeln!(f, "{} {}: Failed to {} for session {}", 
                    ERROR_ICON.red(),
                    "State Error".red().bold(), 
                    operation,
                    session_id.blue()
                )?;
                
                if let Some(suggestion) = suggestion {
                    writeln!(f, "\n{}", suggestion.yellow())?;
                } else {
                    writeln!(f, "\n{} Try removing /tmp/claude_code_personalities_activity_*.json files", LIGHTBULB_ICON.yellow())?;
                }
            }
            
            PersonalityError::System { message, suggestion } => {
                writeln!(f, "{} {}: {}", 
                    ERROR_ICON.red(),
                    "System Error".red().bold(), 
                    message
                )?;
                
                if let Some(suggestion) = suggestion {
                    writeln!(f, "\n{}", suggestion.yellow())?;
                }
            }
        }
        
        Ok(())
    }
}

impl std::error::Error for PersonalityError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PersonalityError::IO { source, .. } => Some(source),
            PersonalityError::Parsing { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Result type alias for convenience
#[allow(dead_code)]
pub type Result<T> = std::result::Result<T, PersonalityError>;