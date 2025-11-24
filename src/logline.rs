// src/logline.rs
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct LogLine {
    /// Logical source name ("nginx-access", "tmp-notify", etc.)
    pub source: String,

    /// Raw text of the line
    pub line: String,

    /// When we read it
    pub timestamp: SystemTime,
}
