// src/config.rs
use serde::Deserialize;
use std::{fs, path::Path, path::PathBuf};
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

            // If the path is empty, we can't read anything.
            if s.path.as_os_str().is_empty() {
                return Err(ConfigError::Invalid(format!(
                    "Source `{}` has an empty path.",
                    s.name
                )));
            }
        }

        self.dedup_sources_by_name();

        Ok(())
    }

    fn dedup_sources_by_name(&mut self) {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        self.sources.retain(|s| seen.insert(s.name.clone()));
    }
}
