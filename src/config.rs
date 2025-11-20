// src/config.rs
use serde::Deserialize;
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct Config {
    /// Whether to follow files like `tail -F`.
    pub follow: bool,

    /// Lines must match at least one of these (if not empty).
    #[serde(default)]
    pub include: Vec<String>,

    /// Lines must NOT match any of these.
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Log sources to read.
    pub sources: Vec<SourceConfig>,
}

#[derive(Debug, Deserialize)]
pub struct SourceConfig {
    /// Human-friendly name, printed in output.
    pub name: String,

    /// Path to the log file.
    pub path: PathBuf,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file `{path}`: {source}")]
    Io {
        #[source]
        source: std::io::Error,
        path: String,
    },

    #[error("Failed to parse YAML in `{path}`: {source}")]
    Parse {
        #[source]
        source: serde_yaml::Error,
        path: String,
    },

    #[error("Invalid configuration: {0}")]
    Invalid(String),

    #[error("Source `{name}`: file not found at `{path}`")]
    SourceFileNotFound { name: String, path: String },

    #[error("Source `{name}`: `{path}` is not a regular file")]
    SourceNotAFile { name: String, path: String },

    #[error("Source `{name}`: cannot access `{path}`: {source}")]
    SourceIo {
        name: String,
        path: String,
        #[source]
        source: io::Error,
    },
}

impl Config {
    /// Load and validate configuration from a YAML file.
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let path_str = path.display().to_string();

        let contents = fs::read_to_string(path).map_err(|e| ConfigError::Io {
            source: e,
            path: path_str.clone(),
        })?;

        let mut cfg: Config = serde_yaml::from_str(&contents).map_err(|e| ConfigError::Parse {
            source: e,
            path: path_str.clone(),
        })?;

        cfg.validate()?;
        Ok(cfg)
    }

    fn validate(&mut self) -> Result<(), ConfigError> {
        if self.sources.is_empty() {
            return Err(ConfigError::Invalid(
                "At least one log source must be specified.".into(),
            ));
        }

        // basic sanity checks
        for s in &self.sources {
            // If the name is empty, it's not very useful.
            if s.name.trim().is_empty() {
                return Err(ConfigError::Invalid("Source name cannot be empty.".into()));
            }

            // NOTE:
            // If the path is empty, we can't read anything.
            // Filesystem paths are not guaranteed to be valid UTF-8,
            // so we check the OsStr (OS-native string) directly.
            if s.path.as_os_str().is_empty() {
                return Err(ConfigError::Invalid(format!(
                    "Source `{}` has an empty path.",
                    s.name
                )));
            }
        }

        self.dedup_sources_by_name();
        self.validate_source_paths()?;

        Ok(())
    }

    /// Deduplicate sources by name, keeping the first occurrence.
    fn dedup_sources_by_name(&mut self) {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        self.sources.retain(|s| seen.insert(s.name.clone()));
    }

    /// Validate that each source path exists and is a regular file.
    fn validate_source_paths(&self) -> Result<(), ConfigError> {
        use std::io::ErrorKind;

        for s in &self.sources {
            let name = s.name.clone();
            let path_str = s.path.display().to_string();

            let meta = match fs::metadata(&s.path) {
                Ok(m) => m,
                Err(e) => {
                    return match e.kind() {
                        ErrorKind::NotFound => Err(ConfigError::SourceFileNotFound {
                            name,
                            path: path_str,
                        }),
                        ErrorKind::PermissionDenied => Err(ConfigError::SourceIo {
                            name,
                            path: path_str,
                            source: e,
                        }),
                        // Other kinds of errors (e.g., I/O errors)
                        _ => Err(ConfigError::SourceIo {
                            name,
                            path: path_str,
                            source: e,
                        }),
                    };
                }
            };

            if !meta.is_file() {
                return Err(ConfigError::SourceNotAFile {
                    name,
                    path: path_str,
                });
            }
        }

        Ok(())
    }
}
