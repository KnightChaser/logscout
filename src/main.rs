// src/main.rs
mod config;

use crate::config::{Config, ConfigError, SourceKind};
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
    for src in &cfg.sources {
        print_source(src);
    }

    Ok(())
}

fn print_source(src: &config::SourceConfig) {
    match &src.kind {
        SourceKind::File { path } => {
            println!("    - name: {}", src.name);
            println!("      type: file");
            println!("      path: {}", path.display());
        }
        SourceKind::Command { command, args } => {
            println!("    - name: {}", src.name);
            println!("      type: command");
            println!("      command: {}", command);
            println!("      args: {:?}", args);
        }
    }
}
