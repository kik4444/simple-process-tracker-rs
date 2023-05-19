use std::path::PathBuf;

pub mod commands;
pub mod errors;
pub mod process_scanner;
pub mod structures;

pub fn get_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("simple-process-tracker"))
}
