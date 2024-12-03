// This program automates updates for Homebrew, Cargo, Rustup, and Neovim plugins.
// It checks for network connectivity before running update commands and logs the output with timestamps.

// The `Local` struct from the `chrono` crate is used for handling dates and times.
use chrono::Local;

// Import various modules from the Rust standard library.
use std::{
    // The `env` module is used for interacting with environment variables.
    env,
    // The `Error` trait is used for error handling.
    error::Error,
    // The `OpenOptions` struct is used for configuring how a file is opened.
    fs::OpenOptions,
    // The `BufRead`, `BufReader`, and `Write` traits are used for buffered I/O operations.
    io::{BufRead, BufReader, Write},
    // The `SocketAddr` and `TcpStream` structs are used for network socket operations.
    net::{SocketAddr, TcpStream},
    // The `Command` and `Stdio` structs are used for running external commands and handling their I/O.
    process::{Command, Stdio},
    // The `Duration` struct is used for specifying time intervals.
    time::Duration,
};

// The main function of the program. It returns a `Result` type that can contain an empty tuple `()`
// on success or a boxed error (`Box<dyn Error>`) on failure.
fn main() -> Result<(), Box<dyn Error>> {
    // Retrieve the log directory path from the environment variable `LOG_HOME`.
    // If `LOG_HOME` is not set, default to `"/var/logs"`.
    let log_home = env::var("LOG_HOME").unwrap_or_else(|_| String::from("/var/logs"));

    // Check if the network is available by attempting to connect to a known address.
    if !check_network()? {
        // If the network is not available, log the offline status and exit.
        log_offline(&log_home)?;
        return Ok(());
    }

    // Run and log Homebrew commands for updating and cleaning up packages.
    run_commands_and_log(
        vec![
            // Update Homebrew package list.
            "/opt/homebrew/bin/brew update",
            // Upgrade all installed Homebrew packages.
            "/opt/homebrew/bin/brew upgrade",
            // Remove old versions of packages.
            "/opt/homebrew/bin/brew cleanup",
        ],
        &log_home, // The directory where logs will be stored.
        "brew",    // The name used to identify the log file.
    )?;

    // Run and log Cargo commands for updating Rust packages.
    run_commands_and_log(
        vec![
            // Update all installed Cargo packages.
            "~/.dev/cargo/bin/cargo install-update -a",
        ],
        &log_home,
        "cargo",
    )?;

    // Run and log Rustup commands for updating Rust toolchains.
    run_commands_and_log(
        vec![
            // Update Rust toolchains and components.
            "~/.dev/cargo/bin/rustup update",
        ],
        &log_home,
        "rustup",
    )?;

    // Run and log Neovim commands for updating plugins.

    // Execute Neovim in headless mode to update plugins using the 'Lazy' plugin manager.
    let status = Command::new("/opt/homebrew/bin/nvim") // Path to the Neovim executable.
        .args(&[
            "--headless",  // Run Neovim without a user interface.
            "-V1",         // Set the verbosity level to 1 for logging.
            "+Lazy! sync", // Run the ':Lazy sync' command to update plugins.
            "+qa",         // Quit Neovim after running the command.
        ])
        .stdout(Stdio::piped()) // Capture standard output.
        .stderr(Stdio::piped()) // Capture standard error.
        .spawn()? // Start the process.
        .wait_with_output()?; // Wait for the process to finish and collect the output.

    // Get the current timestamp in the format "YYYY-MM-DD HH:MM:SS".
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");

    // Define the path for the Neovim log file.
    let log_path = format!("{}/cronup.nvim.log", log_home);

    // Open the Neovim log file in append mode, creating it if it doesn't exist.
    let mut nvim_log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    // Write the status of the Neovim plugin update to the log file.
    writeln!(
        nvim_log,
        "[{}] Neovim plugin update {}",
        timestamp,
        if status.status.success() {
            // If the exit status is successful, indicate success.
            "completed successfully"
        } else {
            // If the exit status is not successful, indicate failure.
            "failed"
        }
    )?;

    // Convert the standard output bytes to a UTF-8 string.
    if let Ok(output) = String::from_utf8(status.stdout) {
        // Iterate over each line in the output.
        for line in output.lines() {
            // Check if the line is not empty after trimming whitespace.
            if !line.trim().is_empty() {
                // Write the line to the log file with a timestamp.
                writeln!(nvim_log, "[{}] {}", timestamp, line)?;
            }
        }
    }

    // Convert the standard error bytes to a UTF-8 string.
    if let Ok(error) = String::from_utf8(status.stderr) {
        // Iterate over each line in the error output.
        for line in error.lines() {
            // Check if the line is not empty after trimming whitespace.
            if !line.trim().is_empty() {
                // Write the line to the log file with a timestamp.
                writeln!(nvim_log, "[{}] {}", timestamp, line)?;
            }
        }
    }

    // Return `Ok(())` to indicate the program completed successfully.
    Ok(())
}

// Function to check if the network is available.
// It tries to establish a TCP connection to a known reliable DNS server.
fn check_network() -> Result<bool, Box<dyn Error>> {
    // Define the socket address for the DNS server at 9.9.9.9 on port 53.
    let address: SocketAddr = "9.9.9.9:53".parse()?;

    // Set a timeout duration of 5 seconds for the connection attempt.
    let timeout = Duration::from_secs(5);

    // Attempt to establish a TCP connection to the specified address with the timeout.
    // The `is_ok()` method returns `true` if the connection was successful.
    Ok(TcpStream::connect_timeout(&address, timeout).is_ok())
}

// Function to log that the system is offline and updates were aborted.
fn log_offline(log_home: &str) -> Result<(), Box<dyn Error>> {
    // Get the current timestamp.
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");

    // Define the path for the offline log file.
    let offline_log_path = format!("{}/cronup.offline.log", log_home);

    // Open the offline log file in append mode, creating it if it doesn't exist.
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(offline_log_path)?;

    // Write the offline status message to the log file with a timestamp.
    writeln!(file, "[{}] System offline - updates aborted.", timestamp)?;

    // Return `Ok(())` to indicate the function completed successfully.
    Ok(())
}

// Function to run a list of shell commands and log their output.
// It accepts a vector of command strings, the log directory, and a name for the log file.
fn run_commands_and_log(
    commands: Vec<&str>, // Vector of command strings to execute.
    log_home: &str,      // Directory where the log file will be stored.
    name: &str,          // Name used to identify the log file.
) -> Result<(), Box<dyn Error>> {
    // Define the path for the log file using the provided name.
    let log_path = format!("{}/cronup.{}.log", log_home, name);

    // Join the list of commands into a single string separated by '&&'.
    // This ensures that the next command runs only if the previous one succeeds.
    let shell_cmd = commands.join(" && ");

    // Execute the combined shell command using `/bin/bash -c`.
    let output = Command::new("/bin/bash")
        .arg("-c") // Specify that the next argument is a command.
        .arg(shell_cmd) // The shell command to execute.
        .stdout(Stdio::piped()) // Capture standard output.
        .stderr(Stdio::piped()) // Capture standard error.
        .output()?; // Execute the command and wait for it to finish.

    // Get the current timestamp.
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");

    // Open the log file in append mode, creating it if it doesn't exist.
    let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;

    // Create a buffered reader for the standard output.
    let stdout = BufReader::new(&output.stdout[..]);

    // Iterate over each line in the standard output.
    for line in stdout.lines() {
        // Handle any errors that may occur while reading lines.
        let line = line?;
        // Check if the line is not empty after trimming whitespace.
        if !line.trim().is_empty() {
            // Write the line to the log file with a timestamp.
            writeln!(log_file, "[{}] {}", timestamp, line)?;
        }
    }

    // Create a buffered reader for the standard error.
    let stderr = BufReader::new(&output.stderr[..]);

    // Iterate over each line in the standard error.
    for line in stderr.lines() {
        // Handle any errors that may occur while reading lines.
        let line = line?;
        // Check if the line is not empty after trimming whitespace.
        if !line.trim().is_empty() {
            // Write the line to the log file with a timestamp.
            writeln!(log_file, "[{}] {}", timestamp, line)?;
        }
    }

    // Return `Ok(())` to indicate the function completed successfully.
    Ok(())
}
