// src/main.rs
mod config;
mod filters;
mod logline;
mod reader;

use crate::config::{Config, ConfigError};
use crate::filters::Filters;
use crate::logline::LogLine;
use std::env;
use std::path::Path;
use std::sync::mpsc;

fn main() {
    if let Err(err) = run() {
        eprintln!("[logscout]: error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), ConfigError> {
    let config_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "config.yaml".to_string());

    let path = Path::new(&config_path);

    let cfg = Config::from_file(path)?;

    // Build filters (can fil if regex is invalid)
    let filters = Filters::from_config(&cfg)?;

    // Set up channels
    let (tx, rx) = mpsc::channel::<LogLine>();

    // Spawn reader threads for all source
    let _handles = reader::spawn_readers(&cfg.sources, tx);

    // Consume data
    println!("[logscout] Waiting for log lines...");
    for msg in rx {
        if filters.matches(&msg.line) {
            println!("[{}] {}", msg.source, msg.line);
        }
    }

    Ok(())
}
