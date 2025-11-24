# logscout

A simple, lightweight log monitoring and filtering tool written in Rust. `logscout` reads logs from multiple sources (files or commands), filters them based on regular expressions, and prints the results to your console.

This project was created as a practice exercise to learn the Rust programming language.

## Features

- **Multiple Data Sources**: Read from static files or capture standard output from commands.
- **Flexible Filtering**:
  - **Include**: Only show lines matching specific patterns.
  - **Exclude**: Hide lines matching specific patterns (takes precedence).
- **Regex Support**: Use regular expressions for powerful pattern matching.
- **Aggregation**: Interleaves logs from multiple sources into a single output stream.
- **Statistics**: Displays a summary of processed, included, and excluded lines upon exit.

## Installation

Ensure you have Rust and Cargo installed. You can build the project using:

```bash
cargo build --release
```

The binary will be available in `target/release/logscout`.

## Usage

Run `logscout` by providing a configuration file.

```bash
# Run with a specific config file
./target/release/logscout my_config.yaml
```

To stop the application, press `Ctrl+C`. `logscout` will handle the signal and print a summary of the session statistics before exiting.

## Configuration

`logscout` uses a YAML configuration file to define sources and filter rules.

### Configuration Options

- **follow** (boolean): Intended to enable `tail -f` style following.
- **include** (list of strings): A list of regex patterns. If provided, only lines matching at least one of these patterns will be displayed. If empty, all lines (not excluded) are shown.
- **exclude** (list of strings): A list of regex patterns. Lines matching any of these patterns will be hidden, even if they match an include pattern.
- **sources** (list of objects): A list of log sources to monitor.

### Source Options

Each source must have a `name` and a `type` ("file" or "command").

- **File Source**:
  - `type`: "file"
  - `path`: Absolute or relative path to the log file.

- **Command Source**:
  - `type`: "command"
  - `command`: The executable to run.
  - `args`: A list of arguments to pass to the command.

### Example Configuration

```yaml
# config.yaml
follow: true

# Show lines containing "ERROR", "CRITICAL", etc.
include:
  - "ERROR"
  - "CRITICAL"
  - "CREATE"
  - "DELETE"

# Hide lines containing "healthcheck", "metrics", etc.
exclude:
  - "healthcheck"
  - "metrics"
  - ".swp"

sources:
  - name: "auditd"
    type: "file"
    path: "/var/log/audit/audit.log"

  - name: "tmp-inotify"
    type: "command"
    command: "inotifywait"
    args: ["-m", "-r", "/tmp"]
```