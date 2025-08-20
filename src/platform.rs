use anyhow::{anyhow, Result};
use std::env;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Platform {
    pub os: String,
    pub arch: String,
    pub target: String,
}

impl Platform {
    /// Detect the current platform from the system
    pub fn detect() -> Result<Self> {
        let os = detect_os()?;
        let arch = detect_arch()?;
        let target = format!("{arch}-{os}");
        
        Ok(Platform { os, arch, target })
    }
    
    /// Get the expected binary name for this platform
    #[allow(dead_code)]
    pub fn binary_name(&self) -> String {
        format!("claude-code-personalities-{}", self.target)
    }
    
    /// Check if this platform is supported
    pub fn is_supported(&self) -> bool {
        matches!(
            (self.os.as_str(), self.arch.as_str()),
            ("apple-darwin" | "linux", "x86_64" | "aarch64")
        )
    }
    
    /// Get human-readable description
    pub fn description(&self) -> String {
        match (self.os.as_str(), self.arch.as_str()) {
            ("apple-darwin", "x86_64") => "macOS (Intel)".to_string(),
            ("apple-darwin", "aarch64") => "macOS (Apple Silicon)".to_string(),
            ("linux", "x86_64") => "Linux (x86_64)".to_string(),
            ("linux", "aarch64") => "Linux (ARM64)".to_string(),
            _ => format!("{} ({})", self.os, self.arch),
        }
    }
}

fn detect_os() -> Result<String> {
    match env::consts::OS {
        "macos" => Ok("apple-darwin".to_string()),
        "linux" => Ok("linux".to_string()),
        "windows" => Err(anyhow!("Windows is not currently supported")),
        other => Err(anyhow!("Unsupported operating system: {}", other)),
    }
}

fn detect_arch() -> Result<String> {
    match env::consts::ARCH {
        "x86_64" => Ok("x86_64".to_string()),
        "aarch64" => Ok("aarch64".to_string()),
        "arm64" => Ok("aarch64".to_string()), // macOS reports arm64 sometimes
        other => Err(anyhow!("Unsupported architecture: {}", other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = Platform::detect().unwrap();
        
        // Should be able to detect current platform
        assert!(!platform.os.is_empty());
        assert!(!platform.arch.is_empty());
        assert!(!platform.target.is_empty());
        
        // Should generate valid binary name
        let binary_name = platform.binary_name();
        assert!(binary_name.starts_with("claude-code-personalities-"));
    }
    
    #[test]
    fn test_supported_platforms() {
        let platforms = vec![
            Platform {
                os: "apple-darwin".to_string(),
                arch: "x86_64".to_string(),
                target: "x86_64-apple-darwin".to_string(),
            },
            Platform {
                os: "apple-darwin".to_string(),
                arch: "aarch64".to_string(),
                target: "aarch64-apple-darwin".to_string(),
            },
            Platform {
                os: "linux".to_string(),
                arch: "x86_64".to_string(),
                target: "x86_64-linux".to_string(),
            },
            Platform {
                os: "linux".to_string(),
                arch: "aarch64".to_string(),
                target: "aarch64-linux".to_string(),
            },
        ];
        
        for platform in platforms {
            assert!(platform.is_supported(), "Platform should be supported: {platform:?}");
        }
    }
    
    #[test]
    fn test_unsupported_platform() {
        let platform = Platform {
            os: "windows".to_string(),
            arch: "x86_64".to_string(),
            target: "x86_64-windows".to_string(),
        };
        
        assert!(!platform.is_supported());
    }
    
    #[test]
    fn test_platform_descriptions() {
        let test_cases = vec![
            (Platform {
                os: "apple-darwin".to_string(),
                arch: "x86_64".to_string(),
                target: "x86_64-apple-darwin".to_string(),
            }, "macOS (Intel)"),
            (Platform {
                os: "apple-darwin".to_string(),
                arch: "aarch64".to_string(),
                target: "aarch64-apple-darwin".to_string(),
            }, "macOS (Apple Silicon)"),
            (Platform {
                os: "linux".to_string(),
                arch: "x86_64".to_string(),
                target: "x86_64-linux".to_string(),
            }, "Linux (x86_64)"),
        ];
        
        for (platform, expected) in test_cases {
            assert_eq!(platform.description(), expected);
        }
    }
    
    #[test]
    fn test_binary_name_generation() {
        let platform = Platform {
            os: "apple-darwin".to_string(),
            arch: "x86_64".to_string(),
            target: "x86_64-apple-darwin".to_string(),
        };
        
        assert_eq!(platform.binary_name(), "claude-code-personalities-x86_64-apple-darwin");
    }
}