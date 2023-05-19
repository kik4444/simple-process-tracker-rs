use std::path::PathBuf;

pub mod commands;
pub mod errors;
pub mod processes;
pub mod structures;

pub fn get_config_file() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("simple-process-tracker").join("config.json"))
}
