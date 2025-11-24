// src/filters.rs
use crate::config::{Config, ConfigError};
use regex::Regex;

#[derive(Debug)]
pub struct Filters {
    include: Vec<Regex>,
    exclude: Vec<Regex>,
}

impl Filters {
    /// Build Filters from Config.[include|exclude].
    /// Every pattern must be a valid regex; otherwise we throw ConfigError
    pub fn from_config(cfg: &Config) -> Result<Self, ConfigError> {
        let mut include = Vec::new();
        let mut exclude = Vec::new();

        for pattern in &cfg.include {
            let re = Regex::new(pattern).map_err(|e| ConfigError::InvalidRegex {
                kind: "include",
                pattern: pattern.clone(),
                source: e,
            })?;
            include.push(re);
        }

        for pattern in &cfg.exclude {
            let re = Regex::new(pattern).map_err(|e| ConfigError::InvalidRegex {
                kind: "exclude",
                pattern: pattern.clone(),
                source: e,
            })?;
            exclude.push(re);
        }

        Ok(Self { include, exclude })
    }

    /// Return true if the line **passes** the filters.
    ///
    /// Rules:
    /// - If it matches any exclude regex -> reject (false).
    /// - Else, if include list is empty -> accept everything (true).
    /// - Else, accept only if it matches at least one include regex.
    pub fn matches(&self, line: &str) -> bool {
        // 1. Exclude has highest priority
        if self.exclude.iter().any(|re| re.is_match(line)) {
            return false;
        }

        // 2. If no include filters, accept everything (except excluded)
        if self.include.is_empty() {
            return true;
        }

        // 3. Require at least one include to match
        self.include.iter().any(|re| re.is_match(line))
    }
}
