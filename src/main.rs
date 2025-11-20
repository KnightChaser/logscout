// src/main.rs
mod config;

use crate::config::{Config, ConfigError};
use std::env;
use std::path::Path;

fn main() {
    if let Err(err) = run() {
        eprintln!("log-scout: error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), ConfigError> {
    let config_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "config.yaml".to_string());

    let path = Path::new(&config_path);

    let cfg = Config::from_file(path)?;

    println!("Loaded config from `{}`:", path.display());
    println!("  follow: {}", cfg.follow);
    println!("  include: {:?}", cfg.include);
    println!("  exclude: {:?}", cfg.exclude);
    println!("  sources:");
    for s in &cfg.sources {
        println!("    - {} -> {}", s.name, s.path.display());
    }

    Ok(())
}
