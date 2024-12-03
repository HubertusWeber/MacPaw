// This program monitors the connection state of a Proton VPN and automatically manages Little Snitch firewall profiles
// When the VPN connects, it disables Little Snitch, and when VPN disconnects, it enables a specific "VPN Off" profile

// Standard library imports
use std::env; // For reading environment variables
use std::error::Error; // Provides the Error trait for error handling
use std::io::Write; // Provides writing capabilities for files
use std::path::PathBuf;
use std::process::Command; // Allows executing system commands
use std::time::{SystemTime, UNIX_EPOCH}; // For working with system time and timestamps // For path manipulation

// External crate imports
use chrono::Local; // For formatted date/time handling
                   // Core Foundation imports (macOS specific framework)
use core_foundation::base::TCFType; // Trait for Core Foundation types
use core_foundation::date::{CFDate, CFDateRef}; // For working with CF dates
use core_foundation::string::{CFString, CFStringRef}; // For CF string handling
use core_foundation_sys::base::CFGetTypeID; // For type checking CF objects
use core_foundation_sys::date::CFDateGetTypeID; // For date type identification
                                                // Core Foundation preferences for storing/retrieving application settings
use core_foundation_sys::preferences::{
    CFPreferencesAppSynchronize, // For saving preferences
    CFPreferencesCopyAppValue,   // For reading preferences
    CFPreferencesSetAppValue,    // For writing preferences
};
use core_foundation_sys::string::CFStringGetTypeID; // For string type identification

// Constants
const APP_ID: &str = "gg.hw.snitchprot"; // Unique identifier for the app's preferences

// Function to get the log file path using environment variable
fn get_log_path() -> PathBuf {
    // Get LOG_HOME environment variable, defaulting to ~/.cache if not set
    let log_home = env::var("LOG_HOME").unwrap_or_else(|_| String::from("/var/logs"));

    // Create a PathBuf and append our log filename
    let mut path = PathBuf::from(log_home);
    path.push("snitchprot.log");
    path
}

// Helper function to get current timestamp in formatted string
fn get_timestamp() -> String {
    Local::now().format("[%Y-%m-%d %H:%M:%S]").to_string()
}

// Function to write a message to the log file with timestamp
fn log_message(message: &str) -> std::io::Result<()> {
    let timestamp = get_timestamp();
    // Get log path dynamically
    let log_path = get_log_path();
    // Ensure the parent directory exists
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // Open file in append mode, create if doesn't exist
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;
    writeln!(file, "{} {}", timestamp, message)
}

// Function to retrieve a preference value from macOS preferences system
fn get_preference(key: &str) -> Option<String> {
    unsafe {
        // Required for Core Foundation API calls
        // Convert the key to a Core Foundation string
        let key = CFString::new(key);
        // Attempt to retrieve the preference value
        let value = CFPreferencesCopyAppValue(
            key.as_concrete_TypeRef(),
            CFString::new(APP_ID).as_concrete_TypeRef(),
        );

        if !value.is_null() {
            // Check what type of value we got back
            let type_id = CFGetTypeID(value);

            if type_id == CFStringGetTypeID() {
                // Handle string values
                let cf_string = CFString::wrap_under_get_rule(value as CFStringRef);
                Some(cf_string.to_string())
            } else if type_id == CFDateGetTypeID() {
                // Handle date values - convert to Unix timestamp
                let cf_date = CFDate::wrap_under_get_rule(value as CFDateRef);
                let time = cf_date.abs_time();
                // Add offset to convert from Core Foundation reference date to Unix epoch
                Some(((time + 978307200.0) as u64).to_string())
            } else {
                None
            }
        } else {
            None
        }
    }
}

// Function to save a preference value to macOS preferences system
fn set_preference(key: &str, value: &str) {
    unsafe {
        // Required for Core Foundation API calls
        let key = CFString::new(key);
        let value = CFString::new(value);
        // Set the preference value
        CFPreferencesSetAppValue(
            key.as_concrete_TypeRef(),
            value.as_CFTypeRef(),
            CFString::new(APP_ID).as_concrete_TypeRef(),
        );
        // Ensure changes are saved to disk
        CFPreferencesAppSynchronize(CFString::new(APP_ID).as_concrete_TypeRef());
    }
}

// Main function where the program logic happens
fn main() -> Result<(), Box<dyn Error>> {
    // Run system command to check VPN status
    let output = Command::new("sudo")
        .args(&["/usr/sbin/scutil", "--nc", "list"])
        .output()?;
    let output_str = String::from_utf8_lossy(&output.stdout);

    // Check if Proton VPN is connected by looking for "proton" and "Connected" in output
    let vpn_connected = output_str
        .lines()
        .any(|line| line.to_lowercase().contains("proton") && line.contains("Connected"));

    // Set current state based on VPN connection status
    let current_state = if vpn_connected {
        "connected"
    } else {
        "disconnected"
    };

    // Get the previous state from preferences
    let previous_state = get_preference("previous_state").unwrap_or_default();

    // Check if we need to force refresh (if last refresh was more than 60 seconds ago)
    let force_refresh = match get_preference("last_refresh_time") {
        Some(last_refresh_time_str) => {
            let last_refresh_time: u64 = last_refresh_time_str.parse()?;
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() - last_refresh_time >= 60
        }
        None => true,
    };

    // If state changed or force refresh is needed
    if current_state != previous_state || force_refresh {
        if current_state != previous_state {
            // Log the state change
            log_message(&format!(
                "VPN state changed from '{}' to '{}'",
                previous_state, current_state
            ))?;

            if current_state == "connected" {
                // If VPN connected, disable Little Snitch
                log_message("Disabling Little Snitch profile...")?;
                Command::new("sudo")
                    .args(&[
                        "/Applications/Little Snitch.app/Contents/Components/littlesnitch",
                        "profile",
                        "-d",
                    ])
                    .output()?;
                log_message("Little Snitch profile disabled")?;
            } else {
                // If VPN disconnected, enable "VPN Off" profile
                log_message("Enabling 'VPN Off' profile...")?;
                Command::new("sudo")
                    .args(&[
                        "/Applications/Little Snitch.app/Contents/Components/littlesnitch",
                        "profile",
                        "-a",
                        "VPN Off",
                    ])
                    .output()?;
                log_message("Little Snitch profile 'VPN Off' enabled")?;
            }
        } else {
            // If force refresh, perform same actions but without logging
            if current_state == "connected" {
                Command::new("sudo")
                    .args(&[
                        "/Applications/Little Snitch.app/Contents/Components/littlesnitch",
                        "profile",
                        "-d",
                    ])
                    .output()?;
            } else {
                Command::new("sudo")
                    .args(&[
                        "/Applications/Little Snitch.app/Contents/Components/littlesnitch",
                        "profile",
                        "-a",
                        "VPN Off",
                    ])
                    .output()?;
            }
        }

        // Update preferences with current state and refresh time
        set_preference("previous_state", current_state);
        set_preference(
            "last_refresh_time",
            &SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_secs()
                .to_string(),
        );
    }

    Ok(())
}

