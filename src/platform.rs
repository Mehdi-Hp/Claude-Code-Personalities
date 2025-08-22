use anyhow::{Result, anyhow};
use std::env;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Platform {
    pub os: String,
    pub arch: String,
    pub target: String,
}

impl Platform {
    /// Detect the current platform from the system
    ///
    /// # Errors
    ///
    /// This function will return an error if the current OS or architecture
    /// is not supported, or if platform detection fails
    pub fn detect() -> Result<Self> {
        let os = detect_os()?;
        let arch = detect_arch()?;
        let target = format!("{arch}-{os}");

        Ok(Platform { os, arch, target })
    }

    /// Check if this platform is supported
    #[must_use]
    pub fn is_supported(&self) -> bool {
        matches!(
            (self.os.as_str(), self.arch.as_str()),
            ("apple-darwin" | "linux", "x86_64" | "aarch64")
        )
    }

    /// Get human-readable description
    #[must_use]
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
        "windows" => Err(anyhow!(
            "Windows is not supported - developer doesn't care about Windows development.\n\n\
            This tool uses Unix-specific features (file permissions, ~/.local/bin, /tmp, Unix shells).\n\n\
            Windows users: Use WSL2 (Windows Subsystem for Linux) for full compatibility:\n\
            https://docs.microsoft.com/en-us/windows/wsl/install"
        )),
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

        // Should be in supported target format
        assert!(platform.target.contains('-'));
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
            assert!(
                platform.is_supported(),
                "Platform should be supported: {platform:?}"
            );
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
            (
                Platform {
                    os: "apple-darwin".to_string(),
                    arch: "x86_64".to_string(),
                    target: "x86_64-apple-darwin".to_string(),
                },
                "macOS (Intel)",
            ),
            (
                Platform {
                    os: "apple-darwin".to_string(),
                    arch: "aarch64".to_string(),
                    target: "aarch64-apple-darwin".to_string(),
                },
                "macOS (Apple Silicon)",
            ),
            (
                Platform {
                    os: "linux".to_string(),
                    arch: "x86_64".to_string(),
                    target: "x86_64-linux".to_string(),
                },
                "Linux (x86_64)",
            ),
        ];

        for (platform, expected) in test_cases {
            assert_eq!(platform.description(), expected);
        }
    }

    #[test]
    fn test_target_format() {
        let platform = Platform {
            os: "apple-darwin".to_string(),
            arch: "x86_64".to_string(),
            target: "x86_64-apple-darwin".to_string(),
        };

        assert_eq!(platform.target, "x86_64-apple-darwin");
        assert!(platform.target.starts_with("x86_64"));
        assert!(platform.target.ends_with("apple-darwin"));
    }
}
