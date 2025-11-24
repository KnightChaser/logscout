// src/reader.rs
use crate::config::{SourceConfig, SourceKind};
use crate::logline::LogLine;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::JoinHandle;
use std::time::SystemTime;

/// Spawn one reader thread per source.
/// Returns the join handles
pub fn spawn_readers(sources: &[SourceConfig], tx: Sender<LogLine>) -> Vec<JoinHandle<()>> {
    let mut handles = Vec::new();

    for src in sources {
        let name = src.name.clone();
        let kind = src.kind.clone();
        let tx_clone = tx.clone(); // Multiple threads need their own sender

        let handle = match kind {
            SourceKind::File { path } => spawn_file_reader(name, path, tx_clone),
            SourceKind::Command { command, args } => {
                spawn_command_reader(name, command, args, tx_clone)
            }
        };

        handles.push(handle);
    }

    handles
}

/// Spawn a thread to read lines from a file
fn spawn_file_reader(
    name: String,
    path: std::path::PathBuf,
    tx: Sender<LogLine>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        // This can still fail at runtime (file removed/permissions changed)
        let file = match File::open(&path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!(
                    "[logscout] soruce `{}`: failed to open file `{}`: {}",
                    name,
                    path.display(),
                    e
                );
                return;
            }
        };

        let reader = BufReader::new(file);

        for line_result in reader.lines() {
            let line = match line_result {
                Ok(l) => l,
                Err(e) => {
                    eprintln!(
                        "[logscout] source `{}`: error reading line from file `{}`: {}",
                        name,
                        path.display(),
                        e
                    );
                    break;
                }
            };

            let msg = LogLine {
                source: name.clone(),
                line: line,
                timestamp: SystemTime::now(),
            };

            if tx.send(msg).is_err() {
                break; // Receiver has been dropped
            }
        }
    })
}

fn spawn_command_reader(
    name: String,
    command: String,
    args: Vec<String>,
    tx: Sender<LogLine>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        // Execute the command and capture its stdout
        let mut child = match Command::new(&command)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "[logscout] source `{}`: failed to spawn command `{}`: {}",
                    name, command, e
                );
                return;
            }
        };

        let stdout = match child.stdout.take() {
            Some(s) => s,
            None => {
                eprintln!(
                    "[logscout] source `{}`: failed to capture stdout of command `{}`",
                    name, command
                );
                return;
            }
        };

        let reader = BufReader::new(stdout);

        for line_result in reader.lines() {
            let line = match line_result {
                Ok(l) => l,
                Err(e) => {
                    eprintln!(
                        "[logscout] source `{}`: error reading line from command `{}`: {}",
                        name, command, e
                    );
                    break;
                }
            };

            let msg = LogLine {
                source: name.clone(),
                line: line,
                timestamp: SystemTime::now(),
            };

            if tx.send(msg).is_err() {
                break; // Receiver has been dropped
            }
        }

        // Wait for the child to exit; ignore status for now
        let _ = child.wait();
    })
}
