use std::fmt;
use colored::*;

/// Custom error types for claude-code-personalities
#[derive(Debug)]
pub enum PersonalityError {
    /// Configuration-related errors
    Configuration {
        message: String,
        suggestion: Option<String>,
    },
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
    /// Interactive prompt errors
    Prompt {
        message: String,
        suggestion: Option<String>,
    },
}

impl PersonalityError {
    pub fn config(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
            suggestion: None,
        }
    }
    
    pub fn config_with_suggestion(message: impl Into<String>, suggestion: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
            suggestion: Some(suggestion.into()),
        }
    }
    
    pub fn io(operation: impl Into<String>, path: Option<impl Into<String>>, source: std::io::Error) -> Self {
        Self::IO {
            operation: operation.into(),
            path: path.map(|p| p.into()),
            source,
            suggestion: None,
        }
    }
    
    pub fn io_with_suggestion(
        operation: impl Into<String>, 
        path: Option<impl Into<String>>, 
        source: std::io::Error,
        suggestion: impl Into<String>
    ) -> Self {
        Self::IO {
            operation: operation.into(),
            path: path.map(|p| p.into()),
            source,
            suggestion: Some(suggestion.into()),
        }
    }
    
    pub fn parsing(context: impl Into<String>, source: serde_json::Error) -> Self {
        Self::Parsing {
            context: context.into(),
            input_preview: None,
            source,
            suggestion: None,
        }
    }
    
    pub fn parsing_with_preview(
        context: impl Into<String>, 
        source: serde_json::Error,
        input: &str
    ) -> Self {
        let preview = if input.len() > 100 {
            format!("{}...", &input[..100])
        } else {
            input.to_string()
        };
        
        Self::Parsing {
            context: context.into(),
            input_preview: Some(preview),
            source,
            suggestion: None,
        }
    }
    
    pub fn state(session_id: impl Into<String>, operation: impl Into<String>) -> Self {
        Self::State {
            session_id: session_id.into(),
            operation: operation.into(),
            suggestion: None,
        }
    }
    
    pub fn system(message: impl Into<String>) -> Self {
        Self::System {
            message: message.into(),
            suggestion: None,
        }
    }
    
    pub fn system_with_suggestion(message: impl Into<String>, suggestion: impl Into<String>) -> Self {
        Self::System {
            message: message.into(),
            suggestion: Some(suggestion.into()),
        }
    }
    
    pub fn prompt(message: impl Into<String>) -> Self {
        Self::Prompt {
            message: message.into(),
            suggestion: Some("The interactive prompt was interrupted. Try running the command again.".to_string()),
        }
    }
    
    /// Add a suggestion to any error type
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        match &mut self {
            PersonalityError::Configuration { suggestion: s, .. } |
            PersonalityError::IO { suggestion: s, .. } |
            PersonalityError::Parsing { suggestion: s, .. } |
            PersonalityError::State { suggestion: s, .. } |
            PersonalityError::System { suggestion: s, .. } |
            PersonalityError::Prompt { suggestion: s, .. } => {
                *s = Some(suggestion.into());
            }
        }
        self
    }
}

impl fmt::Display for PersonalityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PersonalityError::Configuration { message, suggestion } => {
                writeln!(f, "{} {}: {}", 
                    "❌".red(),
                    "Configuration Error".red().bold(), 
                    message
                )?;
                
                if let Some(suggestion) = suggestion {
                    writeln!(f, "\n{}", suggestion.yellow())?;
                }
            }
            
            PersonalityError::IO { operation, path, source, suggestion } => {
                if let Some(path) = path {
                    writeln!(f, "{} {}: Failed to {} {}", 
                        "❌".red(),
                        "File Error".red().bold(), 
                        operation,
                        path.blue()
                    )?;
                } else {
                    writeln!(f, "{} {}: Failed to {}", 
                        "❌".red(),
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
                    "❌".red(),
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
                    "❌".red(),
                    "State Error".red().bold(), 
                    operation,
                    session_id.blue()
                )?;
                
                if let Some(suggestion) = suggestion {
                    writeln!(f, "\n{}", suggestion.yellow())?;
                } else {
                    writeln!(f, "\n{} Try removing /tmp/claude_activity_*.json files", "💡".yellow())?;
                }
            }
            
            PersonalityError::System { message, suggestion } => {
                writeln!(f, "{} {}: {}", 
                    "❌".red(),
                    "System Error".red().bold(), 
                    message
                )?;
                
                if let Some(suggestion) = suggestion {
                    writeln!(f, "\n{}", suggestion.yellow())?;
                }
            }
            
            PersonalityError::Prompt { message, suggestion } => {
                writeln!(f, "{} {}: {}", 
                    "❌".red(),
                    "Prompt Error".red().bold(), 
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
pub type Result<T> = std::result::Result<T, PersonalityError>;

/// Helper trait to convert anyhow errors to PersonalityError with context
pub trait PersonalityContext<T> {
    fn config_context(self, message: impl Into<String>) -> Result<T>;
    fn io_context(self, operation: impl Into<String>, path: Option<impl Into<String>>) -> Result<T>;
    fn parsing_context(self, context: impl Into<String>) -> Result<T>;
    fn state_context(self, session_id: impl Into<String>, operation: impl Into<String>) -> Result<T>;
    fn system_context(self, message: impl Into<String>) -> Result<T>;
}

impl<T> PersonalityContext<T> for std::result::Result<T, anyhow::Error> {
    fn config_context(self, message: impl Into<String>) -> Result<T> {
        self.map_err(|_| PersonalityError::config(message))
    }
    
    fn io_context(self, operation: impl Into<String>, path: Option<impl Into<String>>) -> Result<T> {
        self.map_err(|e| {
            if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                PersonalityError::io(operation, path, io_err.kind().into())
            } else {
                PersonalityError::config(format!("IO operation failed: {}", e))
            }
        })
    }
    
    fn parsing_context(self, context: impl Into<String>) -> Result<T> {
        self.map_err(|e| {
            match e.downcast::<serde_json::Error>() {
                Ok(json_err) => PersonalityError::parsing(context, json_err),
                Err(original_err) => PersonalityError::config(format!("Parsing failed: {}", original_err))
            }
        })
    }
    
    fn state_context(self, session_id: impl Into<String>, operation: impl Into<String>) -> Result<T> {
        self.map_err(|_| PersonalityError::state(session_id, operation))
    }
    
    fn system_context(self, message: impl Into<String>) -> Result<T> {
        self.map_err(|_| PersonalityError::system(message))
    }
}