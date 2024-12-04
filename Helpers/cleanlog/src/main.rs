// This program manages log file retention by removing entries older than specified retention periods.
// It processes log files that contain timestamps in the format [YYYY-MM-DD HH:MM:SS] at the start
// of each line. Lines without timestamps are preserved.

// Standard library imports
use std::env; // For reading environment variables
use std::fs::File; // File system operations
use std::io::{self, BufRead, BufReader, Write}; // Input/Output operations
use std::path::{Path, PathBuf}; // Path manipulation utilities
use std::process; // For exiting the program

// External crate imports
use chrono::{Duration, NaiveDateTime, Utc}; // DateTime handling and calculations
use tempfile::NamedTempFile; // Temporary file operations for safe file writing

// Configuration structure to define each log file's settings
#[derive(Debug)]
struct LogConfig {
    relative_path: &'static str, // The path relative to LOG_HOME
    retention_days: u32,         // How many days of logs to keep
}

// Static configuration array - modify this to set up your log files
// Each entry defines a log file path (relative to LOG_HOME) and its retention period
const LOG_CONFIGS: &[LogConfig] = &[
    LogConfig {
        relative_path: "cronup.brew.log",
        retention_days: 7,
    },
    LogConfig {
        relative_path: "cronup.cargo.log",
        retention_days: 3,
    },
    LogConfig {
        relative_path: "cronup.nvim.log",
        retention_days: 1,
    },
    LogConfig {
        relative_path: "cronup.rustup.log",
        retention_days: 5,
    },
    LogConfig {
        relative_path: "snitchprot.log",
        retention_days: 1,
    },
];

/// Gets the LOG_HOME directory from environment variable or returns default
fn get_log_home() -> PathBuf {
    // Try to get LOG_HOME from environment, default to /var/log if not set
    env::var("LOG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/var/log"))
}

/// Attempts to parse a timestamp from a log line
/// Expected format: [YYYY-MM-DD HH:MM:SS]
/// Returns None if the line doesn't match the expected format
fn parse_timestamp(line: &str) -> Option<NaiveDateTime> {
    // Check if the line is long enough to contain a timestamp
    if line.len() < 21 {
        return None;
    }

    // Extract the timestamp portion (excluding the brackets)
    let timestamp_str = &line[1..20]; // Slice containing "YYYY-MM-DD HH:MM:SS"

    // Attempt to parse the timestamp string into a NaiveDateTime
    NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S").ok()
}

/// Processes a single log file according to its retention configuration
/// Takes the full path to the log file and its retention configuration
/// Returns the number of lines removed or an IO error if something goes wrong
fn clean_log_file(full_path: &Path, retention_days: u32) -> io::Result<usize> {
    // Check if the file exists before attempting to process it
    if !full_path.exists() {
        return Ok(0);
    }

    // Open the original file for reading
    let file = File::open(full_path)?;
    let reader = BufReader::new(file);

    // Create a temporary file to write the filtered content
    let mut temp_file = NamedTempFile::new()?;

    // Get current time for comparison
    let current_time = Utc::now().naive_utc();

    // Calculate the cutoff time based on retention period
    let retention_period = Duration::days(retention_days as i64);

    // Counter for removed lines
    let mut lines_removed = 0;

    // Process the file line by line
    for line in reader.lines() {
        let line = line?;

        // Determine if we should keep this line
        // We keep the line if:
        // 1. It doesn't have a valid timestamp (preserve non-log lines)
        // 2. Its timestamp is within the retention period
        let should_keep = match parse_timestamp(&line) {
            Some(timestamp) => {
                // Keep if the difference between current time and timestamp
                // is less than or equal to the retention period
                current_time - timestamp <= retention_period
            }
            None => true, // Keep lines without valid timestamps
        };

        // Write the line to the temporary file if we're keeping it
        if should_keep {
            writeln!(temp_file, "{}", line)?;
        } else {
            lines_removed += 1;
        }
    }

    // Replace the original file with the cleaned version
    // This is an atomic operation on most filesystems
    temp_file.persist(full_path)?;

    Ok(lines_removed)
}

/// Main program entry point
/// Processes all configured log files and exits on any error
fn main() {
    // Get the LOG_HOME directory (defaults to /var/log)
    let log_home = get_log_home();

    // Exit if log_home doesn't exist or isn't a directory
    if !log_home.is_dir() {
        process::exit(1);
    }

    // Process each log file configuration
    for config in LOG_CONFIGS {
        // Construct the full path by joining LOG_HOME with the relative path
        let full_path = log_home.join(config.relative_path);

        // Process the file and exit on error
        if clean_log_file(&full_path, config.retention_days).is_err() {
            process::exit(1);
        }
    }
}
