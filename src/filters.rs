// src/filters.rs
use crate::config::{Config, ConfigError};
use regex::Regex;

#[derive(Debug)]
pub struct Filters {
    include: Vec<Regex>,
    exclude: Vec<Regex>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterDecision {
    /// Line matched an exclude regex and then dropped, count as excluded.
    Excluded,

    /// Line passed and matched an include regex.
    Included,

    /// Line passed without any include regex match (include list empty).
    Passed,

    /// Line was dropped because it didn't match any include regex.
    DroppedNoIncludeMatch,
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

    /// Classify a line according to include/exclude rules.
    ///
    /// Rules:
    /// - If it matches any exclude regex -> Excluded
    /// - Else if include list is empty -> Passed
    /// - Else if it matches any include regex -> Included
    /// - Else -> DroppedNoIncludeMatch
    pub fn classify(&self, line: &str) -> FilterDecision {
        // Check excludes first
        if self.exclude.iter().any(|re| re.is_match(line)) {
            return FilterDecision::Excluded;
        }

        // Then check includes. If empty, pass all.
        if self.include.is_empty() {
            return FilterDecision::Passed;
        }

        if self.include.iter().any(|re| re.is_match(line)) {
            FilterDecision::Included
        } else {
            FilterDecision::DroppedNoIncludeMatch
        }
    }

    /// Convenience wrapper if you only care about "should this be printed?"
    pub fn matches(&self, line: &str) -> bool {
        matches!(
            self.classify(line),
            FilterDecision::Included | FilterDecision::Passed
        )
    }
}
