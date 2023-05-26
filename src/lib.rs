use std::path::PathBuf;

use interprocess::local_socket::NameTypeSupport;

pub mod client_utils;
pub mod commands;
pub mod process_scanner;
pub mod server_utils;
pub mod structures;

pub const ACTIVE_ICON: &str = "✅";
pub const PAUSED_ICON: &str = "❌";
pub const MIN_POLL_INTERVAL: u64 = 10;
pub const MIN_DURATION_UPDATE_INTERVAL: u64 = 1;
pub const MIN_AUTOSAVE_INTERVAL: u64 = 60;

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

pub fn string_to_duration(input: &str) -> Result<u64, String> {
    let parts: Vec<String> = input.split(':').map(|s| s.to_string()).collect();

    if parts.len() != 3 {
        return Err("invalid duration input".into());
    }

    let (hours, minutes, seconds) =
        move || -> Result<(u64, u64, u64), Box<dyn std::error::Error>> {
            Ok((
                parts[0].parse::<u64>()?,
                parts[1].parse::<u64>()?,
                parts[2].parse::<u64>()?,
            ))
        }()
        .map_err(|e| format!("invalid duration {input} -> {e}"))?;

    Ok(hours * 3600 + minutes * 60 + seconds)
}

pub fn duration_to_string(input: u64) -> String {
    let hours = input / 3600;
    let minutes = input % 3600 / 60;
    let seconds = input % 3600 % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

pub fn parse_range(input: &str) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
    let mut output = vec![];

    let ranges: Vec<String> = input.split(',').map(|s| s.to_string()).collect();

    for range in ranges {
        if range.contains('-') {
            let parts: Vec<String> = range.split('-').map(|s| s.to_string()).collect();

            if parts.len() != 2 {
                return Err(format!("invalid range {range}").into());
            }

            let (left, right) = (parts[0].parse::<usize>()?, parts[1].parse::<usize>()?);

            for i in left..=right {
                output.push(i);
            }
        } else {
            output.push(range.parse::<usize>()?)
        }
    }

    Ok(output)
}

pub fn parse_datetime(input: &str) -> Result<chrono::NaiveDateTime, String> {
    chrono::NaiveDateTime::parse_from_str(input, "%Y/%m/%d %H:%M:%S")
        .map_err(|e| format!("invalid date time {input} -> {e}"))
}
