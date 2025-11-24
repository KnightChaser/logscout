// src/main.rs
mod config;
mod filters;
mod logline;
mod reader;
mod stats;

use crate::config::{Config, ConfigError};
use crate::filters::Filters;
use crate::logline::LogLine;
use crate::stats::Stats;
use std::env;
use std::path::Path;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};

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

    // Shared shutdown flag (Ctrl+C)
    let shutdown = Arc::new(AtomicBool::new(false));
    {
        let shutdown_flag = shutdown.clone();
        ctrlc::set_handler(move || {
            // Only print on first [Ctrl]+[C]
            let first = !shutdown_flag.swap(true, Ordering::SeqCst);
            if first {
                println!("\n[logscout] Shutdown signal received, terminating...");
            }
        })
        .expect("[logscout] Error setting Ctrl-C handler");
    }

    // Set up channels
    let (tx, rx) = mpsc::channel::<LogLine>();

    // Spawn reader threads for all source with shutdown flag
    let _handles = reader::spawn_readers(&cfg.sources, tx, shutdown.clone());

    // Stats (atomic counters)
    let stats = Arc::new(Stats::new());

    // Consume data
    println!("[logscout] Waiting for log lines...");
    for msg in rx {
        if shutdown.load(Ordering::SeqCst) {
            break;
        }

        stats.inc_total();

        match filters.classify(&msg.line) {
            filters::FilterDecision::Excluded => {
                stats.inc_excluded();
                // Silently ignore excluded lines
            }

            filters::FilterDecision::Included => {
                stats.inc_included();
                println!("[{}] {}", msg.source, msg.line);
            }

            filters::FilterDecision::Passed => {
                stats.inc_included();
                println!("[{}] {}", msg.source, msg.line);
            }

            filters::FilterDecision::DroppedNoIncludeMatch => {
                // Do nothing
            }
        }
    }

    // After loop, print the summary
    let (total, included, excluded) = stats.snapshot();
    println!("\n[logscout] Summary:");
    println!("  Total lines processed: {}", total);
    println!("  Included lines: {}", included);
    println!("  Excluded lines: {}", excluded);

    Ok(())
}
