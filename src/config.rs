// src/config.rs
use serde::Deserialize;
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
pub struct SourceConfig {
    /// Human-friendly name, printed in output.
    pub name: String,

    #[serde(flatten)]
    pub kind: SourceKind,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")] // "file" or "command"
pub enum SourceKind {
    #[serde(rename = "file")]
    File { path: PathBuf },

    #[serde(rename = "command")]
    Command {
        command: String,
        #[serde(default)]
        args: Vec<String>,
    },
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

    #[error("Source `{name}`: command is empty")]
    SourceCommandEmpty { name: String },

    #[error("Source `{name}`: cannot access `{path}`: {source}")]
    SourceIo {
        name: String,
        path: String,
        #[source]
        source: io::Error,
    },

    #[allow(dead_code)]
    #[error("Source `{name}`: failed to spawn command `{command}`: {source}")]
    SourceSpawn {
        name: String,
        command: String,
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
        }

        self.dedup_sources_by_name();
        self.validate_sources()?;

        Ok(())
    }

    /// Deduplicate sources by name, keeping the first occurrence.
    fn dedup_sources_by_name(&mut self) {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        self.sources.retain(|s| seen.insert(s.name.clone()));
    }

    /// Validate that sources are accessible and valid.
    fn validate_sources(&self) -> Result<(), ConfigError> {
        use std::io::ErrorKind;

        for s in &self.sources {
            match &s.kind {
                // Check that the given log file exists and is a regular file.
                SourceKind::File { path } => {
                    let name = s.name.clone();
                    let path_str = path.display().to_string();

                    let meta = match fs::metadata(path) {
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
                                _ => Err(ConfigError::SourceIo {
                                    name,
                                    path: path_str,
                                    source: e,
                                }),
                            }?;
                        }
                    };
                    if !meta.is_file() {
                        return Err(ConfigError::SourceNotAFile {
                            name,
                            path: path_str,
                        });
                    }
                }

                // Check that the command is not empty. (Later we try to spawn it to verify.)
                SourceKind::Command { command, .. } => {
                    if command.trim().is_empty() {
                        return Err(ConfigError::SourceCommandEmpty {
                            name: s.name.clone(),
                        });
                    }
                }
            }
        }

        Ok(())
    }
}
