use colored::Colorize;
use std::fmt;

/// Custom error types for claude-code-personalities
#[derive(Debug)]
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
    #[allow(dead_code)]
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

impl fmt::Display for PersonalityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Nerd Font icons using UTF-8 byte sequences
        const ERROR_ICON: &str = "\u{f057}"; // 
        const LIGHTBULB_ICON: &str = "\u{f0eb}"; // 

        match self {
            PersonalityError::IO {
                operation,
                path,
                source,
                suggestion,
            } => {
                if let Some(path) = path {
                    writeln!(
                        f,
                        "{} {}: Failed to {} {}",
                        ERROR_ICON.red(),
                        "File Error".red().bold(),
                        operation,
                        path.blue()
                    )?;
                } else {
                    writeln!(
                        f,
                        "{} {}: Failed to {}",
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

            PersonalityError::Parsing {
                context,
                input_preview,
                source,
                suggestion,
            } => {
                writeln!(
                    f,
                    "{} {}: {}",
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
                    writeln!(
                        f,
                        "\n{}",
                        "This is likely a Claude Code compatibility issue.".yellow()
                    )?;
                }
            }

            PersonalityError::State {
                session_id,
                operation,
                suggestion,
            } => {
                writeln!(
                    f,
                    "{} {}: Failed to {} for session {}",
                    ERROR_ICON.red(),
                    "State Error".red().bold(),
                    operation,
                    session_id.blue()
                )?;

                if let Some(suggestion) = suggestion {
                    writeln!(f, "\n{}", suggestion.yellow())?;
                } else {
                    writeln!(
                        f,
                        "\n{} Try removing /tmp/claude_code_personalities_activity_*.json files",
                        LIGHTBULB_ICON.yellow()
                    )?;
                }
            }

            PersonalityError::System {
                message,
                suggestion,
            } => {
                writeln!(
                    f,
                    "{} {}: {}",
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
