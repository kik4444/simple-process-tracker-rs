use std::path::PathBuf;

use interprocess::local_socket::NameTypeSupport;

pub mod client;
pub mod commands;
pub mod errors;
pub mod process_scanner;
pub mod server;
pub mod structures;

pub const ACTIVE_ICON: &str = "⏵";
pub const PAUSED_ICON: &str = "⏸";

pub fn get_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("simple-process-tracker"))
}

pub fn get_socket_name() -> &'static str {
    use NameTypeSupport::*;

    match NameTypeSupport::query() {
        OnlyPaths => "/tmp/simple-process-tracker.sock",
        OnlyNamespaced | Both => "@simple-process-tracker.sock",
    }
}

pub fn string_to_duration(input: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let parts: Vec<String> = input.split(':').map(|s| s.to_string()).collect();

    if parts.len() != 3 {
        return Err("invalid duration input".into());
    }

    let duration = parts[0].parse::<u64>()? * 3600
        + parts[1].parse::<u64>()? * 60
        + parts[2].parse::<u64>()?;

    Ok(duration)
}

pub fn duration_to_string(input: u64) -> String {
    let hours = input / 3600;
    let minutes = input % 3600 / 60;
    let seconds = input % 3600 % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}
